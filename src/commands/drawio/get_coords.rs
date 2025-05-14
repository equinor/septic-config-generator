use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use csv::{QuoteStyle, WriterBuilder};
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

/// Public API to extract coords and write CSV
pub fn extract_coords(input: &Path, output: Option<&Path>) -> Result<(usize, PathBuf)> {
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

/// Single-pass XML parse to extract rectangles
fn parse_xml_and_extract_rectangles(xml: &str) -> Result<Vec<RectData>> {
    const PREFIX: &str = "septic_";
    let doc = Document::parse(xml).context("Error parsing XML")?;
    let mut rects = Vec::new();
    for obj in doc.descendants().filter(|n| n.has_tag_name("object")) {
        if !obj
            .attributes()
            .any(|a| a.name().to_ascii_lowercase().starts_with(PREFIX))
        {
            continue;
        }
        let mut props = HashMap::with_capacity(8);
        for attr in obj.attributes() {
            let name = attr.name();
            if name.to_ascii_lowercase().starts_with(PREFIX) {
                let key = name[PREFIX.len()..].to_ascii_lowercase();
                props.insert(key, clean_attribute_value(attr.value()));
            }
        }
        if let Some((x, y, w, h)) = extract_coordinates(&obj) {
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

/// Fast coordinate extraction
#[inline]
fn extract_coordinates(node: &roxmltree::Node) -> Option<(i32, i32, i32, i32)> {
    node.children()
        .find(|n| n.has_tag_name("mxCell"))
        .and_then(|cell| {
            cell.children()
                .find(|n| n.has_tag_name("mxGeometry"))
                .map(|geom| {
                    let parse_i = |k| geom.attribute(k).unwrap_or("0").parse::<i32>().unwrap_or(0);
                    (
                        parse_i("x"),
                        parse_i("y"),
                        parse_i("width"),
                        parse_i("height"),
                    )
                })
        })
}

/// Write rectangles to CSV with buffered I/O and reused buffers
fn write_rectangles_to_csv(output: &Path, rects: &[RectData]) -> Result<()> {
    const PRIORITY: [&str; 2] = ["type", "name"];
    const COORDS: [&str; 4] = ["x1", "y1", "x2", "y2"];

    let file = File::create(output)
        .with_context(|| format!("Error creating file {}", output.display()))?;
    let buf = BufWriter::with_capacity(1 << 20, file);
    let mut wtr = WriterBuilder::new()
        .quote_style(QuoteStyle::Never)
        .from_writer(buf);

    // Build header
    let mut all = HashSet::new();
    for r in rects {
        all.extend(r.properties.keys().cloned());
    }
    let mut header: Vec<String> = Vec::with_capacity(all.len() + COORDS.len());
    for &p in &PRIORITY {
        if all.contains(p) {
            header.push(p.to_string());
        }
    }
    header.extend(COORDS.iter().map(|&c| c.to_string()));
    let mut rem: Vec<_> = all
        .into_iter()
        .filter(|p| !PRIORITY.contains(&p.as_str()))
        .collect();
    rem.sort_unstable();
    header.extend(rem);
    wtr.write_record(&header)?;

    // Cache indices
    let idx_map: HashMap<_, _> = header
        .iter()
        .enumerate()
        .map(|(i, v)| (v.as_str(), i))
        .collect();
    // Pre-alloc record buffer
    let mut record = vec![String::new(); header.len()];

    for r in rects {
        // clear previous
        for cell in &mut record {
            cell.clear();
        }
        // coords
        if let Some(&i) = idx_map.get("x1") {
            record[i].push_str(&r.x.to_string());
        }
        if let Some(&i) = idx_map.get("y1") {
            record[i].push_str(&r.y.to_string());
        }
        if let Some(&i) = idx_map.get("x2") {
            record[i].push_str(&(r.x + r.width).to_string());
        }
        if let Some(&i) = idx_map.get("y2") {
            record[i].push_str(&(r.y + r.height).to_string());
        }
        // props
        for (k, v) in &r.properties {
            if let Some(&i) = idx_map.get(k.as_str()) {
                record[i].push_str(v);
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
    fn test_integration_getcoords() {
        // Define paths
        let test_dir = Path::new("tests/testdata");
        let input_file = test_dir.join("test.drawio");
        let expected_file = test_dir.join("drawio_test_coords.csv");
        let output_file = test_dir.join("output_test_coords.csv");

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
                "getcoords",
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
