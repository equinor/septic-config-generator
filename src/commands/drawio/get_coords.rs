use anyhow::{Context, Result};
use base64::{Engine as _, engine::general_purpose};
use flate2::read::ZlibDecoder;
use regex::Regex;
use roxmltree::Document;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::{Path, PathBuf};

/// A structure to represent a rectangle with optional properties
#[derive(Debug, Clone)]
struct RectData {
    name: String,
    rect_type: String,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    // Optional properties for ImageStatusLabel
    texts: Option<String>,
    colors: Option<String>,
}

/// Helper function to strip HTML tags completely
fn strip_html_tags(input: &str) -> String {
    let mut result = input.to_string();
    let re = Regex::new(r"<[^>]*>").unwrap();
    while re.is_match(&result) {
        result = re.replace_all(&result, "").to_string();
    }
    result.trim().to_string()
}

/// Helper function to clean attribute values but preserve individual quotes
fn clean_attribute_value(value: &str) -> String {
    // This preserves the individual quotes around each value
    // but removes any extraneous quotes that might be present
    value.trim().to_string()
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
        Some(output_path) => std::path::PathBuf::from(output_path),
        None => {
            let out = format!("{}_coords", input.with_extension("").to_string_lossy());
            PathBuf::from(out).with_extension("csv")
        }
    };

    // println!("Input file:         {}", input.display());
    // println!("Output csv file:    {}", output.display());

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

    let types = ["ImageXvrPlot", "ImageXvr", "ImageStatusLabel"];
    let mut rectangles = Vec::new();

    for node in doc.descendants().filter(|n| n.has_tag_name("object")) {
        let label = node.attribute("label").unwrap_or("");
        let clean_label = strip_html_tags(label);

        if let Some(matched_type) = types.iter().find(|&&t| clean_label.contains(t)) {
            let name = extract_name(&clean_label, matched_type);

            // Extract additional properties for ImageStatusLabel and ImageXvr
            let texts = if *matched_type == "ImageStatusLabel" && node.has_attribute("Texts") {
                node.attribute("Texts").map(clean_attribute_value)
            } else {
                None
            };

            // Check for Colors attribute for both types, but don't assume it's always present
            let colors = if (*matched_type == "ImageStatusLabel" || *matched_type == "ImageXvr")
                && node.has_attribute("Colors")
            {
                node.attribute("Colors").map(clean_attribute_value)
            } else {
                None
            };

            if let Some((x, y, width, height)) = extract_coordinates(&node) {
                rectangles.push(RectData {
                    name,
                    rect_type: matched_type.to_string(),
                    x,
                    y,
                    width,
                    height,
                    texts,
                    colors,
                });
            }
        }
    }
    Ok(rectangles)
}

/// Extract the name of the rectangle
fn extract_name(clean_label: &str, matched_type: &str) -> String {
    let name = clean_label
        .replace(matched_type, "")
        .replace("=", "")
        .trim()
        .to_string();
    if name.is_empty() {
        "Unnamed".to_string()
    } else {
        name
    }
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

/// Write rectangle data to a CSV file
fn write_rectangles_to_csv(output: &Path, rectangles: &[RectData]) -> Result<()> {
    let mut file = File::create(output)
        .with_context(|| format!("Error creating output file {}", output.display()))?;

    // Write header with all possible columns
    writeln!(file, "name,type,y1,x1,y2,x2,texts,colors")
        .with_context(|| format!("Error writing to output file {}", output.display()))?;

    // Write data rows with all columns
    for rect in rectangles {
        writeln!(
            file,
            "{},{},{},{},{},{},{},{}",
            rect.name,
            rect.rect_type,
            rect.x,
            rect.y,
            rect.x + rect.width,
            rect.y + rect.height,
            rect.texts.as_ref().unwrap_or(&String::new()),
            rect.colors.as_ref().unwrap_or(&String::new())
        )
        .with_context(|| format!("Error writing to output file {}", output.display()))?;
    }

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
        // cargo run -- drawio getcoords --input <input_file> --output <output_file>
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
