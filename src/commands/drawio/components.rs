use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use csv::WriterBuilder;
use flate2::read::ZlibDecoder;
use html_escape::decode_html_entities;
use roxmltree::Document;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::fs::File;
use std::io::{BufWriter, Read};
use std::path::{Path, PathBuf};

/// A structure to represent a rectangle with septic properties
#[derive(Debug, Clone)]
struct RectData {
    properties: HashMap<String, String>,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// Decode HTML entities and trim whitespace
#[inline]
fn clean_attribute_value(value: &str) -> String {
    decode_html_entities(value.trim()).to_string()
}

/// Decompress base64+zlib encoded diagram data
fn decompress_diagram(data: &str) -> Result<String, String> {
    let decoded = general_purpose::STANDARD
        .decode(data)
        .map_err(|e| format!("Base64 decoding error: {}", e))?;
    let mut decoder = ZlibDecoder::new(&decoded[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| format!("Zlib decompression error: {}", e))?;
    String::from_utf8(decompressed).map_err(|e| format!("UTF-8 conversion error: {}", e))
}

/// Public API to extract components and write CSV
pub fn extract_components(input: &Path, output: Option<&Path>) -> Result<(usize, PathBuf)> {
    let output_path = output.map(PathBuf::from).unwrap_or_else(|| {
        let base = input.with_extension("").to_string_lossy().into_owned();
        PathBuf::from(format!("{}_coords.csv", base))
    });
    let xml_content = process_drawio_file(input)?;
    let rectangles = parse_xml_and_extract_rectangles(&xml_content)?;
    write_rectangles_to_csv(&output_path, &rectangles)?;
    Ok((rectangles.len(), output_path))
}

/// Read full file content
fn read_file_content(input: &Path) -> Result<String> {
    fs::read_to_string(input).with_context(|| format!("Failed to read file: {}", input.display()))
}

/// Process .drawio files and decompress if needed
fn process_drawio_file(input: &Path) -> Result<String> {
    let content = read_file_content(input)?;
    if let Ok(doc) = Document::parse(&content) {
        for node in doc
            .descendants()
            .filter(|n| n.has_tag_name("diagram") && n.text().is_some())
        {
            if let Some(text) = node.text() {
                if let Ok(decompressed) = decompress_diagram(text) {
                    return Ok(decompressed);
                }
                if let Ok(decoded) = general_purpose::STANDARD.decode(text) {
                    if let Ok(s) = String::from_utf8(decoded) {
                        return Ok(s);
                    }
                }
            }
        }
    }
    Ok(content)
}

/// Parse XML and extract rectangle data accounting for group offsets
fn parse_xml_and_extract_rectangles(xml: &str) -> Result<Vec<RectData>> {
    const PREFIX: &str = "septic_";
    let doc = Document::parse(xml).context("Error parsing XML")?;
    // Build id->geometry offset and id->parent maps for all mxCell nodes
    let mut id_to_geom: HashMap<String, (i32, i32)> = HashMap::new();
    let mut id_to_parent: HashMap<String, String> = HashMap::new();
    for cell in doc.descendants().filter(|n| n.has_tag_name("mxCell")) {
        if let Some(id) = cell.attribute("id") {
            if let Some(geom) = cell.children().find(|n| n.has_tag_name("mxGeometry")) {
                let x = geom
                    .attribute("x")
                    .unwrap_or("0")
                    .parse::<f32>()
                    .unwrap_or(0.0)
                    .round() as i32;
                let y = geom
                    .attribute("y")
                    .unwrap_or("0")
                    .parse::<f32>()
                    .unwrap_or(0.0)
                    .round() as i32;
                id_to_geom.insert(id.to_string(), (x, y));
                if let Some(parent_id) = cell.attribute("parent") {
                    id_to_parent.insert(id.to_string(), parent_id.to_string());
                }
            }
        }
    }
    let mut rects = Vec::new();
    // Extract each object with septic_ props
    for obj in doc.descendants().filter(|n| n.has_tag_name("object")) {
        // Check for any septic_ attributes
        let mut props = HashMap::new();
        for attr in obj.attributes() {
            // Use strip_prefix to remove the PREFIX
            if let Some(stripped) = attr.name().to_ascii_lowercase().strip_prefix(PREFIX) {
                props.insert(stripped.to_string(), clean_attribute_value(attr.value()));
            }
        }
        if props.is_empty() {
            continue;
        }
        if let Some((x, y, w, h)) = extract_coordinates(&obj, &id_to_geom, &id_to_parent) {
            rects.push(RectData {
                properties: props,
                x,
                y,
                width: w,
                height: h,
            });
        }
    }
    Ok(rects)
}

/// Extract coordinates, accumulating group offsets via id maps
fn extract_coordinates(
    obj: &roxmltree::Node,
    id_to_geom: &HashMap<String, (i32, i32)>,
    id_to_parent: &HashMap<String, String>,
) -> Option<(i32, i32, i32, i32)> {
    let cell = obj.children().find(|n| n.has_tag_name("mxCell"))?;
    let geom = cell.children().find(|n| n.has_tag_name("mxGeometry"))?;
    let local_x = geom
        .attribute("x")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0)
        .round() as i32;
    let local_y = geom
        .attribute("y")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0)
        .round() as i32;
    let width = geom
        .attribute("width")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0)
        .round() as i32;
    let height = geom
        .attribute("height")
        .unwrap_or("0")
        .parse::<f32>()
        .unwrap_or(0.0)
        .round() as i32;
    let mut abs_x = local_x;
    let mut abs_y = local_y;
    let mut current = cell.attribute("parent");
    while let Some(pid) = current {
        if let Some(&(px, py)) = id_to_geom.get(pid) {
            abs_x += px;
            abs_y += py;
            current = id_to_parent.get(pid).map(|s| s.as_str());
        } else {
            break;
        }
    }
    Some((abs_x, abs_y, width, height))
}

/// Count quoted values in a string like "red" "blue" "green"
#[inline]
fn count_quoted_values(s: &str) -> usize {
    let comma_count = s.chars().filter(|&c| c == '"').count();
    if comma_count > 2 { comma_count / 2 } else { 1 }
}

/// Write rectangles to CSV (optimized with multi-value counts and proper quoting)
fn write_rectangles_to_csv(output: &Path, rects: &[RectData]) -> Result<()> {
    const PRIORITY: [&str; 2] = ["type", "name"];
    const COORDS: [&str; 4] = ["x1", "y1", "x2", "y2"];

    // Handle empty case
    if rects.is_empty() {
        let file = File::create(output)
            .with_context(|| format!("Error creating file {}", output.display()))?;
        let mut wtr = WriterBuilder::new().from_writer(file);
        wtr.write_record(PRIORITY)?;
        wtr.write_record(COORDS)?;
        wtr.flush()?;
        return Ok(());
    }

    // Collect all keys and identify multi-value fields
    let mut all_keys = HashSet::new();
    let mut has_multi_values = false;
    for r in rects {
        for (k, v) in &r.properties {
            all_keys.insert(k.clone());
            // Detect if this component has any multi-values
            if v.contains('\"') && count_quoted_values(v) > 1 {
                has_multi_values = true;
            }
        }
    }
    // Build header in desired order
    let mut header: Vec<String> = Vec::with_capacity(all_keys.len() + COORDS.len() + 1);
    for &p in &PRIORITY {
        if all_keys.contains(p) {
            header.push(p.to_string());
        }
    }
    header.extend(COORDS.iter().map(|&c| c.to_string()));
    let mut rem: Vec<_> = all_keys
        .iter()
        .filter(|k| !PRIORITY.contains(&k.as_str()))
        .cloned()
        .collect();
    rem.sort_unstable();
    header.extend(rem);

    // Add a single _num column if we have any multi-values
    if has_multi_values {
        header.push("_numvalues".to_string());
    }

    // Create CSV writer with proper quoting settings
    let file = File::create(output)
        .with_context(|| format!("Error creating file {}", output.display()))?;
    let buf = BufWriter::with_capacity(1 << 20, file);

    // Keep the original quote style to maintain compatibility with existing code
    let mut wtr = WriterBuilder::new()
        // .quote_style(QuoteStyle::Never)  // Use the same quote style as your original code
        .from_writer(buf);

    wtr.write_record(&header)?;

    // Build the index map and pre-allocate record buffer
    let idx: HashMap<_, _> = header
        .iter()
        .enumerate()
        .map(|(i, v)| (v.as_str(), i))
        .collect();
    let mut record = vec![String::new(); header.len()];

    // Write each rectangle's data
    for r in rects {
        // Clear previous values but maintain allocation
        for cell in &mut record {
            cell.clear();
        }

        // Add coordinates
        if let Some(i) = idx.get("x1") {
            record[*i] = r.x.to_string();
        }
        if let Some(i) = idx.get("y1") {
            record[*i] = r.y.to_string();
        }
        if let Some(i) = idx.get("x2") {
            record[*i] = (r.x + r.width).to_string();
        }
        if let Some(i) = idx.get("y2") {
            record[*i] = (r.y + r.height).to_string();
        }

        // Add property values - CSV library will handle quoting
        for (k, v) in &r.properties {
            if let Some(i) = idx.get(k.as_str()) {
                record[*i] = v.clone();
            }
        }

        // Add count of multi-values to the single _num column if needed
        if has_multi_values {
            if let Some(i) = idx.get("_numvalues") {
                // Count the maximum number of values in any of the multi-value properties
                let max_count = r
                    .properties
                    .values()
                    .filter(|v| v.contains('"'))
                    .map(|v| count_quoted_values(v))
                    .max()
                    .unwrap_or(0);
                record[*i] = max_count.to_string();
            }
        }

        wtr.write_record(&record)?;
    }

    wtr.flush()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::Path;
    use std::process::Command;

    #[test]
    fn test_integration_components() {
        // Define paths
        let test_dir = Path::new("tests/testdata");
        let input_file = test_dir.join("test.drawio");
        let expected_file = test_dir.join("test_drawio.csv");
        let output_file = test_dir.join("output_test_drawio.csv");

        // Ensure test directory and input files exist
        for file in [test_dir, &input_file, &expected_file] {
            assert!(file.exists(), "Required resource not found: {:?}", file);
        }

        // Remove the output file if it exists from a previous test run
        let _ = fs::remove_file(&output_file);

        // Build command that runs the full CLI
        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "drawio",
                "components",
                "--input",
                input_file.to_str().unwrap(),
                "--output",
                output_file.to_str().unwrap(),
            ])
            .output()
            .expect("Failed to execute process");

        // Check if command executed successfully
        assert!(
            output.status.success(),
            "Command failed with exit code: {}",
            output.status
        );

        // Verify output file was created
        assert!(
            output_file.exists(),
            "Output file was not created: {:?}",
            output_file
        );

        // Read the output file and expected output file
        let output_content =
            fs::read_to_string(&output_file).expect("Failed to read the output file");
        let expected_content =
            fs::read_to_string(&expected_file).expect("Failed to read the expected file");

        // Compare content after normalizing EOL
        assert_eq!(
            output_content.replace("\r\n", "\n"),
            expected_content.replace("\r\n", "\n"),
            "Output doesn't match expected content"
        );

        // Clean up - remove the output file
        let _ = fs::remove_file(&output_file);
    }
}
