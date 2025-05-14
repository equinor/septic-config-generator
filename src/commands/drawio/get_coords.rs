use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use csv::{QuoteStyle, WriterBuilder};
use flate2::read::ZlibDecoder;
use html_escape::decode_html_entities;
use roxmltree::Document;
use std::collections::{HashMap, HashSet};
use std::fs::{self};
use std::io::Read;
use std::path::{Path, PathBuf};

/// A structure to represent a rectangle with septic properties
#[derive(Debug, Clone)]
struct RectData {
    // Store all septic_ properties in a HashMap
    properties: HashMap<String, String>,
    // Geometry properties
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

/// Helper function to clean attribute values by decoding HTML entities and trimming
fn clean_attribute_value(value: &str) -> String {
    decode_html_entities(value.trim()).to_string()
}

/// Helper function to decompress base64+zlib encoded diagram data
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

/// Extract rectangle coordinates from a .drawio or .xml file
pub fn extract_coords(input: &Path, output: Option<&Path>) -> Result<(usize, PathBuf)> {
    let output = match &output {
        Some(output_path) => PathBuf::from(output_path),
        None => {
            let out = format!("{}_coords", input.with_extension("").to_string_lossy());
            PathBuf::from(out).with_extension("csv")
        }
    };

    let xml_content = process_drawio_file(input)?;
    let rectangles = parse_xml_and_extract_rectangles(&xml_content)?;
    write_rectangles_to_csv(&output, &rectangles)?;
    Ok((rectangles.len(), output.to_owned()))
}

/// Read the content of the input file
fn read_file_content(input: &Path) -> Result<String> {
    fs::read_to_string(input).with_context(|| format!("Failed to read file: {}", input.display()))
}

/// Process .drawio files to extract and decompress diagram content
fn process_drawio_file(input: &Path) -> Result<String> {
    let file_content = read_file_content(input)?;
    if let Ok(doc) = Document::parse(&file_content) {
        for node in doc
            .descendants()
            .filter(|n| n.has_tag_name("diagram") && n.text().is_some())
        {
            let diagram_text = node.text().unwrap();
            if let Ok(decompressed) = decompress_diagram(diagram_text) {
                return Ok(decompressed);
            } else if let Ok(decoded) = general_purpose::STANDARD.decode(diagram_text) {
                if let Ok(decoded_str) = String::from_utf8(decoded) {
                    return Ok(decoded_str);
                }
            }
        }
    }
    Ok(file_content.to_string())
}

/// Parse XML content and extract rectangle data
fn parse_xml_and_extract_rectangles(xml_content: &str) -> Result<Vec<RectData>> {
    let doc = Document::parse(xml_content).context("Error parsing XML")?;
    let mut rectangles = Vec::new();

    // Find all septic_ property names across all objects to get a complete list of columns
    let mut all_septic_props = Vec::new();

    // First pass: collect all unique septic property names
    for node in doc.descendants().filter(|n| n.has_tag_name("object")) {
        for attr in node.attributes() {
            let attr_name = attr.name().to_lowercase();
            if attr_name.starts_with("septic_") && !all_septic_props.contains(&attr_name) {
                all_septic_props.push(attr_name);
            }
        }
    }

    // Sort property names for consistent column order
    all_septic_props.sort();

    // Second pass: extract rectangle data
    for node in doc.descendants().filter(|n| n.has_tag_name("object")) {
        // Check if this object has any septic_ property
        let has_septic_props = node
            .attributes()
            .any(|attr| attr.name().to_lowercase().starts_with("septic_"));

        if has_septic_props {
            // Extract all septic_ properties
            let mut properties = HashMap::new();
            for attr in node.attributes() {
                let attr_name = attr.name().to_lowercase();
                if attr_name.starts_with("septic_") {
                    let property_name = attr_name.trim_start_matches("septic_").to_string();
                    let property_value = clean_attribute_value(attr.value());
                    properties.insert(property_name, property_value);
                }
            }

            // Extract coordinates
            if let Some((x, y, width, height)) = extract_coordinates(&node) {
                rectangles.push(RectData {
                    properties,
                    x,
                    y,
                    width,
                    height,
                });
            }
        }
    }

    Ok(rectangles)
}

/// Extract coordinates from an object node
fn extract_coordinates(node: &roxmltree::Node) -> Option<(i32, i32, i32, i32)> {
    for cell in node.children().filter(|n| n.has_tag_name("mxCell")) {
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
            return Some((x, y, width, height));
        }
    }
    None
}

/// Write rectangle data to a CSV file using the `csv` crate
fn write_rectangles_to_csv(output: &Path, rectangles: &[RectData]) -> Result<()> {
    // 1) Collect all property-names across all rectangles
    let mut all_props: HashSet<String> = HashSet::new();
    for rect in rectangles {
        for key in rect.properties.keys() {
            all_props.insert(key.clone());
        }
    }

    // 2) Define the order you want:
    //    first these props (if present), interleaved with coords,
    //    then any remaining props alphabetically.
    let property_priority = ["type", "name"];
    let coords = ["x1", "y1", "x2", "y2"];

    // Build the header row
    let mut header: Vec<String> = Vec::new();

    // 2a) Add any priority props (type, name)
    for &p in &property_priority {
        if all_props.contains(p) {
            header.push(p.to_string());
        }
    }
    // 2b) Then the coords
    header.extend(coords.iter().map(|&c| c.to_string()));

    // 2c) Finally, any other props sorted alphabetically
    let mut remaining: Vec<_> = all_props
        .into_iter()
        .filter(|p| !property_priority.contains(&p.as_str()))
        .collect();
    remaining.sort();
    header.extend(remaining);

    // 3) Create the CSV writer with no quoting and write the header
    let mut wtr = WriterBuilder::new()
        .quote_style(QuoteStyle::Never)
        .from_path(output)
        .with_context(|| format!("Error creating CSV writer for {}", output.display()))?;
    wtr.write_record(&header)?;

    // 4) Write each rectangle’s row by matching each header column
    for rect in rectangles {
        let record: Vec<String> = header
            .iter()
            .map(|col| match col.as_str() {
                "x1" => rect.x.to_string(),
                "y1" => rect.y.to_string(),
                "x2" => (rect.x + rect.width).to_string(),
                "y2" => (rect.y + rect.height).to_string(),
                other => rect.properties.get(other).cloned().unwrap_or_default(),
            })
            .collect();
        wtr.write_record(&record)?;
    }

    // 5) Flush to disk
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
