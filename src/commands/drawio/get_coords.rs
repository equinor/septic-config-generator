use base64::{Engine as _, engine::general_purpose};
use flate2::read::ZlibDecoder;
use regex::Regex;
use roxmltree::Document;
use std::fs::{self, File};
use std::io::Read;
use std::io::Write;
use std::path::Path;

// Type alias for rectangle data: (name, type, x, y, width, height)
type Rectangle = (String, String, i32, i32, i32, i32);
// Type alias for the result of rectangle extraction
type RectangleResult = Result<Vec<Rectangle>, String>;

/// Helper function to strip HTML tags completely
fn strip_html_tags(input: &str) -> String {
    let mut result = input.to_string();
    let re = Regex::new(r"<[^>]*>").unwrap();
    while re.is_match(&result) {
        result = re.replace_all(&result, "").to_string();
    }
    result.trim().to_string()
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
pub fn extract_coords(
    input_filename: &str,
    output_filename: Option<&str>,
) -> Result<(usize, String), String> {
    let output_file = determine_output_filename(input_filename, output_filename);
    let file_content = read_file_content(input_filename)?;
    let xml_content = process_drawio_file(input_filename, &file_content)?;
    let rectangles = parse_xml_and_extract_rectangles(&xml_content)?;
    write_rectangles_to_csv(&output_file, &rectangles)?;
    Ok((rectangles.len(), output_file))
}

/// Determine the output filename
fn determine_output_filename(input_filename: &str, output_filename: Option<&str>) -> String {
    output_filename.map_or_else(
        || {
            format!(
                "{}_coords.csv",
                Path::new(input_filename)
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
            )
        },
        |name| name.to_string(),
    )
}

/// Read the content of the input file
fn read_file_content(input_filename: &str) -> Result<String, String> {
    fs::read_to_string(input_filename).map_err(|e| format!("Error reading file: {}", e))
}

/// Process .drawio files to extract and decompress diagram content
fn process_drawio_file(input_filename: &str, file_content: &str) -> Result<String, String> {
    if input_filename.to_lowercase().ends_with(".drawio") {
        if let Ok(doc) = Document::parse(file_content) {
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
    }
    Ok(file_content.to_string())
}

/// Parse XML content and extract rectangle data
fn parse_xml_and_extract_rectangles(xml_content: &str) -> RectangleResult {
    let doc = Document::parse(xml_content).map_err(|e| format!("Error parsing XML: {}", e))?;
    let types = ["ImageXvrPlot", "ImageXvr"];
    let mut rectangles = Vec::new();
    for node in doc.descendants().filter(|n| n.has_tag_name("object")) {
        let label = node.attribute("label").unwrap_or("");
        let clean_label = strip_html_tags(label);
        if let Some(matched_type) = types.iter().find(|&&t| clean_label.contains(t)) {
            let name = extract_name(&clean_label, matched_type);
            if let Some((x, y, width, height)) = extract_coordinates(&node) {
                rectangles.push((name, matched_type.to_string(), x, y, width, height));
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
fn write_rectangles_to_csv(output_file: &str, rectangles: &[Rectangle]) -> Result<(), String> {
    let mut file =
        File::create(output_file).map_err(|e| format!("Error creating output file: {}", e))?;
    writeln!(file, "name,type,y1,x1,y2,x2")
        .map_err(|e| format!("Error writing to output file: {}", e))?;
    for (name, rect_type, x, y, width, height) in rectangles {
        writeln!(
            file,
            "{},{},{},{},{},{}",
            name,
            rect_type,
            y,
            x,
            y + height,
            x + width
        )
        .map_err(|e| format!("Error writing to output file: {}", e))?;
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
        assert!(
            test_dir.exists(),
            "Test directory not found: {:?}",
            test_dir
        );
        assert!(
            input_file.exists(),
            "Test input file not found: {:?}",
            input_file
        );
        assert!(
            expected_file.exists(),
            "Expected output file not found: {:?}",
            expected_file
        );

        // Remove the output file if it exists from a previous test run
        let _ = fs::remove_file(&output_file);

        // Convert paths to strings for command arguments
        let input_path = input_file.to_str().unwrap();
        let output_path = output_file.to_str().unwrap();

        // Build command that runs the full CLI
        // cargo run -- drawio getcoords --input <input_file> --output <output_file>
        let output = Command::new("cargo")
            .args([
                "run",
                "--",
                "drawio",
                "getcoords",
                "--input",
                input_path,
                "--output",
                output_path,
            ])
            .output()
            .expect("Failed to execute process");

        // Print command output for debugging
        println!(
            "Command stdout: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        println!(
            "Command stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );

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

        // Normalize line endings (convert CRLF to LF)
        let normalized_output = output_content.replace("\r\n", "\n");
        let normalized_expected = expected_content.replace("\r\n", "\n");

        // Compare the content
        assert_eq!(
            normalized_output, normalized_expected,
            "Output doesn't match expected content"
        );

        // Clean up - remove the output file
        let _ = fs::remove_file(&output_file);
    }
}
