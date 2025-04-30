use clap::Args;
use std::process;

mod get_coords;

#[derive(Args, Debug)]
pub struct Drawio {
    #[clap(subcommand)]
    pub command: DrawioCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DrawioCommands {
    /// Convert draw.io files to PNG
    Convertpng(ConvertpngArgs),

    /// Extract coordinates from draw.io files
    Getcoords(GetcoordsArgs),
}

#[derive(Args, Debug)]
pub struct ConvertpngArgs {
    /// Input .drawio or .xml file path
    #[arg(short, long, required = true)]
    pub input: String,

    /// Output PNG file path (default: <input_basename>.png)
    #[arg(short, long)]
    pub output: Option<String>,
}

#[derive(Args, Debug)]
pub struct GetcoordsArgs {
    /// Input .drawio or .xml file path
    #[arg(short, long, required = true)]
    pub input: String,

    /// Output CSV file path (default: <input_basename>_coords.csv)
    #[arg(short, long)]
    pub output: Option<String>,
}

impl Drawio {
    pub fn execute(&self) {
        match &self.command {
            DrawioCommands::Convertpng(args) => self.convert_png(args),
            DrawioCommands::Getcoords(args) => self.get_coords(args),
        }
    }

    fn convert_png(&self, args: &ConvertpngArgs) {
        // Placeholder for PNG conversion - this will be implemented later
        println!("PNG conversion not yet implemented");
        println!(
            "Would convert {} to {}",
            &args.input,
            args.output.as_deref().unwrap_or("output.png")
        );
    }

    fn get_coords(&self, args: &GetcoordsArgs) {
        match get_coords::extract_nested_objects(&args.input, args.output.as_deref()) {
            Ok((count, output)) => println!("Extracted {} objects to {}", count, output),
            Err(e) => {
                eprintln!("Failed to extract coordinates: {}", e);
                process::exit(1);
            }
        }
    }
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
