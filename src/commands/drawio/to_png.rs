use anyhow::{Context, Result, bail};
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
pub fn drawio_to_png(input: &Path, output: Option<&Path>) -> Result<(u32, u32, PathBuf)> {
    let output = match output {
        Some(output) => PathBuf::from(output),
        None => input.with_extension("png"),
    };

    let drawio_command = get_drawio_executable()?;

    let relative_root = output
        .parent()
        .expect("drawio_to_png: Unable to obtain parent of output");

    if !relative_root.exists() {
        fs::create_dir_all(relative_root).with_context(|| {
            format!(
                "Failed to create output directory: {}",
                relative_root.display()
            )
        })?
    }

    let (width, height) = extract_page_dimensions(input)
        .with_context(|| format!("Failed to extract page dimensions from {}", input.display()))?;

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
        .arg(input)
        .arg("--output")
        .arg(&output)
        .output()
        .context("Drawio command failed")?;

    if !cmd_output.status.success() {
        let error_message = String::from_utf8_lossy(&cmd_output.stderr);
        let output_message = String::from_utf8_lossy(&cmd_output.stdout);

        bail!(
            "draw.io conversion failed:\nStderr: {}\nStdout: {}",
            error_message,
            output_message
        );
    }

    if !output.exists() {
        bail!(
            "Conversion completed but output file '{}' was not created",
            output.display()
        );
    }

    match check_png_dimensions(&output, width as usize, height as usize) {
        Ok((size_ok, actual_width, actual_height)) => {
            if size_ok {
                println!("✓ Dimensions match exactly as requested");
            } else {
                println!(
                    "⚠ Dimensions do not match requested size: {}x{} vs {}x{}",
                    actual_width, actual_height, width, height
                );
            }
        }
        Err(e) => bail!("Could not check PNG dimensions: {}", e),
    }

    Ok((width, height, output))
}

fn extract_page_dimensions(input: &Path) -> Result<(u32, u32)> {
    let file_content = fs::read_to_string(input)?;
    let doc = Document::parse(&file_content)?;

    // Find the mxGraphModel element with pageWidth and pageHeight attributes
    for node in doc.descendants() {
        if node.has_tag_name("mxGraphModel") {
            if let (Some(width_str), Some(height_str)) =
                (node.attribute("pageWidth"), node.attribute("pageHeight"))
            {
                let width = width_str
                    .parse::<u32>()
                    .with_context(|| format!("Invalid pageWidth value: {}", width_str))?;
                let height = height_str
                    .parse::<u32>()
                    .with_context(|| format!("Invalid pageHeight value: {}", height_str))?;
                return Ok((width, height));
            }
        }
    }

    println!("Could not find page dimensions, using defaults");
    Ok((1024, 768))
}

/// Helper function to check PNG dimensions
fn check_png_dimensions(
    png_path: &Path,
    expected_width: usize,
    expected_height: usize,
) -> Result<(bool, usize, usize)> {
    // Read only the header bytes to get dimensions
    let info = size(png_path)
        .with_context(|| format!("Failed to read image size from {}", png_path.display()))?;

    // cast expected to usize for the comparison
    let size_ok = (info.width, info.height) == (expected_width, expected_height);
    Ok((size_ok, info.width, info.height))
}

fn get_drawio_executable() -> Result<PathBuf> {
    let executable = if cfg!(target_os = "windows") {
        "draw.io.exe"
    } else if cfg!(target_os = "macos") {
        "draw.io"
    } else {
        "drawio"
    };

    // First try the command in PATH
    let command_check = Command::new(executable).arg("--version").output();

    if command_check.is_ok() {
        return Ok(PathBuf::from(executable));
    }

    // Fall back to default application installation paths
    let possible_paths = if cfg!(target_os = "windows") {
        vec![
            "C:\\Program Files\\draw.io",
            "C:\\Program Files (x86)\\draw.io",
            "C:\\Appl\\bin",
        ]
    } else if cfg!(target_os = "macos") {
        vec!["/Applications/draw.io.app/Contents/MacOS"]
    } else {
        vec![]
    };

    for path in possible_paths.iter() {
        let drawio = PathBuf::from(path).join(executable);
        if drawio.exists() {
            let specific_path_check = Command::new(&drawio).arg("--version").output();

            if specific_path_check.is_ok() {
                return Ok(drawio);
            }
        }
    }
    bail!(
        "{} not found in PATH or at common installation locations. Please ensure draw.io is installed and available.",
        executable
    )
}
