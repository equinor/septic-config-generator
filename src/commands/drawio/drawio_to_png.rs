use roxmltree::Document;
use std::fs;
use std::{path::Path, process::Command};

/// Converts a draw.io file to PNG format, using the page dimensions from the file
///
/// # Arguments
///
/// * `input_filename` - Path to the input draw.io file
/// * `output_filename` - Optional path for the output PNG file. If not provided, the output will have the same name as input but with .png extension
///
/// # Returns
///
/// * `Result<String, String>` - Success message or error message
pub fn drawio_to_png(
    input_filename: &str,
    output_filename: Option<&str>,
) -> Result<String, String> {
    // Check if the draw.io executable is available
    let drawio_command = get_drawio_command()?;

    // Verify input file exists
    let input_path = Path::new(input_filename);
    if !input_path.exists() {
        return Err(format!("Input file '{}' does not exist", input_filename));
    }

    // Make absolute path for input file
    let input_abs = fs::canonicalize(input_path)
        .map_err(|e| format!("Failed to get absolute path for input file: {}", e))?;

    // Determine output filename
    let output_path = match output_filename {
        Some(filename) => Path::new(filename).to_path_buf(),
        None => {
            let stem = input_path
                .file_stem()
                .ok_or_else(|| "Invalid input filename".to_string())?;
            input_path.with_file_name(format!("{}.png", stem.to_string_lossy()))
        }
    };

    // Make absolute path for output file
    let output_dir = output_path.parent().unwrap_or(Path::new("."));
    let _ = fs::create_dir_all(output_dir);
    let output_abs = if output_path.is_absolute() {
        output_path.clone()
    } else {
        std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?
            .join(output_path.clone())
    };

    // Extract page dimensions from the input file
    let dimensions = extract_page_dimensions(input_filename)?;
    let (width, height) = dimensions;

    println!("Input file (absolute): {}", input_abs.display());
    println!("Output file (absolute): {}", output_abs.display());
    println!("Draw.io executable: {}", drawio_command);
    println!("Page dimensions: {}x{}", width, height);

    // Create the conversion command
    let output = Command::new(drawio_command)
        .arg("--export")
        .arg("--format")
        .arg("png")
        .arg("--width")
        .arg(width.to_string())
        .arg("--height")
        .arg(height.to_string())
        .arg("--crop")
        .arg("--border")
        .arg("0")
        .arg(input_abs.to_str().unwrap())
        .arg("--output")
        .arg(output_abs.to_str().unwrap())
        .output()
        .map_err(|e| format!("Failed to execute draw.io command: {}", e))?;

    // Check if the command executed successfully
    if !output.status.success() {
        let error_message = String::from_utf8_lossy(&output.stderr);
        let output_message = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "draw.io conversion failed:\nStderr: {}\nStdout: {}",
            error_message, output_message
        ));
    }

    // Verify the output file was created
    if !output_abs.exists() {
        return Err(format!(
            "Conversion completed but output file '{}' was not created",
            output_abs.display()
        ));
    }

    // Check PNG dimensions if possible
    match check_png_dimensions(&output_abs, width, height) {
        Ok(dimensions_match) => {
            if dimensions_match {
                println!("✓ Dimensions match exactly as requested");
            } else {
                println!("⚠ Dimensions do not match requested size!");
            }
        }
        Err(e) => println!("Could not check PNG dimensions: {}", e),
    }

    Ok(format!(
        "Successfully converted '{}' to '{}' with dimensions {}x{}",
        input_filename,
        output_path.display(),
        width,
        height
    ))
}

/// Parse the draw.io file and extract page dimensions
fn extract_page_dimensions(input_filename: &str) -> Result<(u32, u32), String> {
    println!("Extracting page dimensions from: {}", input_filename);

    // Read the file content
    let file_content =
        fs::read_to_string(input_filename).map_err(|e| format!("Error reading file: {}", e))?;

    // Parse the XML
    let doc = Document::parse(&file_content).map_err(|e| format!("Error parsing XML: {}", e))?;

    // Find the mxGraphModel element with pageWidth and pageHeight attributes
    for node in doc.descendants() {
        if node.has_tag_name("mxGraphModel") {
            if let (Some(width_str), Some(height_str)) =
                (node.attribute("pageWidth"), node.attribute("pageHeight"))
            {
                // Parse the dimensions
                let width = width_str
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid pageWidth value: {}", width_str))?;
                let height = height_str
                    .parse::<u32>()
                    .map_err(|_| format!("Invalid pageHeight value: {}", height_str))?;

                println!("Found page dimensions: {}x{}", width, height);
                return Ok((width, height));
            }
        }
    }

    // If we can't find dimensions, use default dimensions
    println!("Could not find page dimensions, using defaults");
    Ok((1024, 768)) // Default dimensions
}

use imagesize::size;
/// Helper function to check PNG dimensions
fn check_png_dimensions(
    png_path: &Path,
    expected_width: u32,
    expected_height: u32,
) -> Result<bool, String> {
    // Read only the header bytes to get dimensions
    let info = size(png_path).map_err(|e| format!("Failed to read image size: {}", e))?;

    println!("\nPNG dimensions check:");
    println!("Requested: {}x{}", expected_width, expected_height);
    println!("Actual   : {}x{}", info.width, info.height);

    // cast expected to usize for the comparison
    Ok(info.width == expected_width as usize && info.height == expected_height as usize)
}

/// Helper function to get the correct draw.io command
///
/// Returns the appropriate command to use for draw.io executable
fn get_drawio_command() -> Result<String, String> {
    if cfg!(windows) {
        // First try the command in PATH
        let command_check = Command::new("draw.io.exe").arg("--version").output();

        if command_check.is_ok() {
            return Ok("draw.io.exe".to_string());
        }

        // Try default application installation paths
        let possible_paths = [
            "C:\\Program Files\\draw.io\\draw.io.exe",
            "C:\\Appl\\bin\\draw.io.exe",
            "C:\\Program Files (x86)\\draw.io\\draw.io.exe",
        ];

        for path in possible_paths.iter() {
            if Path::new(path).exists() {
                let specific_path_check = Command::new(path).arg("--version").output();

                if specific_path_check.is_ok() {
                    return Ok(path.to_string());
                }
            }
        }

        Err("draw.io.exe executable not found in PATH or at common installation locations. Please ensure draw.io is installed and available.".to_string())
    } else {
        // On non-Windows platforms, just check PATH
        let command_check = Command::new("drawio").arg("--version").output();

        if command_check.is_ok() {
            return Ok("drawio".to_string());
        }

        Err(
            "drawio executable not found in PATH. Please ensure draw.io is installed and available.".to_string()
        )
    }
}