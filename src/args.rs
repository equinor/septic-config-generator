use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(version, about, disable_colored_help = true, next_line_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Generate SEPTIC config
    Make(MakeArguments),
    /// Show difference between two text files
    Diff,
}

#[derive(Parser)]
pub struct MakeArguments {
    /// The yaml config file
    pub config_file: PathBuf,
    /// Name of output file (overrides config option "outputfile")
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// Only output warnings or errors
    #[arg(short, long)]
    pub silent: bool,
    /// Global variable to use for all templates, also those without specified source. Can be repeated. Global variables overwrite other variables with same name
    #[arg(short, long, value_names = ["name", "value"])]
    pub var: Option<Vec<String>>,
}
