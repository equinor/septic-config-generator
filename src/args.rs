use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(version, about, disable_colored_help = true, next_line_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate SEPTIC config
    Make(MakeArguments),
    /// Show difference between two text files
    Diff(DiffArguments),
    /// Check septic .out and .cnc files for error messages
    Checklogs(ChecklogsArguments),
    /// Check for new versions of scg and auto-update
    Update,
}

#[derive(Parser, Debug)]
pub struct MakeArguments {
    /// The yaml config file
    pub config_file: PathBuf,
    // /// Name of output file (overrides config option "outputfile")
    // #[arg(short, long, value_name = "FILE")]
    // pub output: Option<PathBuf>,
    // /// Only output warnings or errors
    // #[arg(short, long)]
    // pub silent: bool,
    /// Global variable to use for all templates, also those without specified source. Can be repeated. Global variables overwrite other variables with same name
    #[arg(short, long, value_names = ["name", "value"])]
    pub var: Option<Vec<String>>,
    /// Only make if layout or source files have changed since last make
    #[arg(long)]
    pub ifchanged: bool,
}

#[derive(Parser, Debug)]
pub struct DiffArguments {
    pub file1: PathBuf,
    pub file2: PathBuf,
}

#[derive(Parser, Debug)]
pub struct ChecklogsArguments {
    #[arg(
        value_name = "RUNDIR",
        help = "The SEPTIC rundir to search for outfiles"
    )]
    pub rundir: PathBuf,
}
