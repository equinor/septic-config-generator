use imagesize::size;
use roxmltree::Document;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
pub fn drawio_to_png(input: &Path, output: Option<&Path>) -> Result<(u32, u32, PathBuf), String> {
    let output = match output {
        Some(output) => PathBuf::from(output),
        None => input.with_extension("png"),
    };

    // Check if the draw.io executable is available
    let drawio_command = get_drawio_command()?;

    // Verify input file exists
    if !input.exists() {
        return Err(format!(
            "Input file '{}' does not exist",
            input.to_str().unwrap()
        ));
    }

    // Make absolute path for output file
    let relative_root = output
        .parent()
        .expect("drawio_to_png: Unable to obtain parent of output");

    if !relative_root.exists() {
        fs::create_dir_all(relative_root)
            .map_err(|e| format!("Failed to create output directory: {}", e))?;
    }

    // Extract page dimensions from the input file
    let (width, height) = extract_page_dimensions(input)?;

    // println!("Input file:         {}", input.display());
    // println!("Output png file:    {}", output.display());
    // println!("Draw.io executable: {}", drawio_command);
    // println!("Page dimensions:    {}x{}", width, height);

    // Create the conversion command
    let cmd_output = Command::new(drawio_command)
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
        .arg(
            input
                .to_str()
                .expect("drawio_to_png: Unable to convert input path to str"),
        )
        .arg("--output")
        .arg(
            output
                .to_str()
                .expect("drawio_to_png: Unable to convert input path to str"),
        )
        .output()
        .map_err(|e| format!("Failed to execute draw.io command: {}", e))?;

    // Check if the command executed successfully
    if !cmd_output.status.success() {
        let error_message = String::from_utf8_lossy(&cmd_output.stderr);
        let output_message = String::from_utf8_lossy(&cmd_output.stdout);
        return Err(format!(
            "draw.io conversion failed:\nStderr: {}\nStdout: {}",
            error_message, output_message
        ));
    }

    // Verify the output file was created
    if !output.exists() {
        return Err(format!(
            "Conversion completed but output file '{}' was not created",
            output.display()
        ));
    }

    // Check PNG dimensions if possible
    match check_png_dimensions(&output, width as usize, height as usize) {
        Ok((size_ok, actual_width, actual_height)) => {
            if size_ok {
                println!("✓ Dimensions match exactly as requested");
            } else {
                println!(
                    "⚠ Dimensions do not match requested size: ({}x{}) vs ({}x{})",
                    actual_width, actual_height, width, height
                );
            }
        }
        Err(e) => println!("Could not check PNG dimensions: {}", e),
    }

    Ok((width, height, output))
}

/// Parse the draw.io file and extract page dimensions
fn extract_page_dimensions(input: &Path) -> Result<(u32, u32), String> {
    let file_content =
        fs::read_to_string(input).map_err(|e| format!("Error reading file: {}", e))?;

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
                return Ok((width, height));
            }
        }
    }

    // If we can't find dimensions, use default dimensions
    println!("Could not find page dimensions, using defaults");
    Ok((1024, 768)) // Default dimensions
}

/// Helper function to check PNG dimensions
fn check_png_dimensions(
    png_path: &Path,
    expected_width: usize,
    expected_height: usize,
) -> Result<(bool, usize, usize), String> {
    // Read only the header bytes to get dimensions
    let info = size(png_path).map_err(|e| format!("Failed to read image size: {}", e))?;

    // cast expected to usize for the comparison
    let size_ok = info.width == expected_width && info.height == expected_height;
    Ok((size_ok, info.width, info.height))
}

/// Helper function to get the correct draw.io command
///
/// Returns the appropriate command to use for draw.io executable
fn get_drawio_command() -> Result<String, String> {
    if cfg!(target_os = "windows") {
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
        if cfg!(target_os = "macos") {
            // On macOS, check the default installation path first
            let application_path = "/Applications/draw.io.app/Contents/MacOS/draw.io";
            let command_check = Command::new(application_path).arg("--version").output();

            if command_check.is_ok() {
                return Ok(application_path.to_string());
            }
        }
        // On non-Windows platforms, check the PATH
        let command_check = Command::new("drawio").arg("--version").output();
        if command_check.is_ok() {
            return Ok("drawio".to_string());
        }

        Err("drawio executable not found in PATH. Please ensure draw.io is installed and available.".to_string())
    }
}
