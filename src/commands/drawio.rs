use clap::Args;
use std::path::{Path, PathBuf};

pub mod components;
pub mod to_png;

#[derive(Args, Debug)]
pub struct Drawio {
    #[clap(subcommand)]
    pub command: DrawioCommands,
}

#[derive(clap::Subcommand, Debug)]
pub enum DrawioCommands {
    /// Convert draw.io files to PNG
    ToPng(ToPngArgs),

    /// Extract components from draw.io files
    Components(ComponentsArgs),
}

#[derive(Args, Debug)]
pub struct ToPngArgs {
    /// Input .drawio or .xml file path
    #[arg(short, long, required = true)]
    pub input: PathBuf,

    /// Output PNG file path (default: <input_basename>.png)
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

#[derive(Args, Debug)]
pub struct ComponentsArgs {
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
            DrawioCommands::ToPng(args) => self.cmd_to_png(&args.input, args.output.as_deref()),
            DrawioCommands::Components(args) => {
                self.cmd_components(&args.input, args.output.as_deref())
            }
        }
    }

    fn cmd_to_png(&self, input: &Path, output: Option<&Path>) {
        let result = to_png::drawio_to_png(input, output);
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

    fn cmd_components(&self, input: &Path, output: Option<&Path>) {
        match components::extract_components(input, output) {
            Ok((count, output)) => {
                println!("Extracted {} components to '{}'", count, output.display())
            }
            Err(e) => {
                eprintln!("Failed to extract components: {}", e);
                std::process::exit(1);
            }
        }
    }
}
