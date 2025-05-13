use clap::Args;
use std::path::{Path, PathBuf};

pub mod drawio_to_png;
pub mod get_coords;

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
    pub input: PathBuf,

    /// Output PNG file path (default: <input_basename>.png)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct GetcoordsArgs {
    /// Input .drawio or .xml file path
    #[arg(short, long, required = true)]
    pub input: PathBuf,

    /// Output CSV file path (default: <input_basename>_coords.csv)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

impl Drawio {
    pub fn execute(&self) {
        match &self.command {
            DrawioCommands::Convertpng(args) => {
                self.cmd_convert_png(&args.input, args.output.as_deref())
            }
            DrawioCommands::Getcoords(args) => {
                self.cmd_get_coords(&args.input, args.output.as_deref())
            }
        }
    }

    fn cmd_convert_png(&self, input: &Path, output: Option<&Path>) {
        let result = drawio_to_png::drawio_to_png(input, output);
        match result {
            Ok((width, height, output)) => println!(
                "Converted to '{}' with dimensions {}x{}",
                output.display(),
                width,
                height
            ),
            Err(err) => {
                eprintln!("Failed to convert: {}", err);
                std::process::exit(1);
            }
        }
    }

    fn cmd_get_coords(&self, input: &Path, output: Option<&Path>) {
        match get_coords::extract_coords(input, output) {
            Ok((count, output)) => {
                println!("Extracted {} objects to '{}'", count, output.display())
            }
            Err(e) => {
                eprintln!("Failed to extract coordinates: {}", e);
                std::process::exit(1);
            }
        }
    }
}
