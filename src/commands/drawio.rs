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
