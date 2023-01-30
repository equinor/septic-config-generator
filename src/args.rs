use std::path::PathBuf;

#[derive(clap::Parser)]
#[command(version, about, long_about=None)]
#[command(next_line_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(clap::Subcommand)]
pub enum Commands {
    /// Generate SEPTIC config
    Make(MakeArgs),
    /// Show difference between two text files
    Diff,
}

#[derive(clap::Parser)]
pub struct MakeArgs {
    /// The yaml config file
    pub config_file: PathBuf,
    /// Name of output file (overrides config option "outputfile")
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,
    /// Do not prompt for verification of output file before overwriting original (overrides config option "verifycontent")
    #[arg(short, long = "no-verify")]
    pub noverify: bool,
    /// Only output warnings or errors
    #[arg(short, long)]
    pub silent: bool,
    /// Global variable to use for all templates, also those without specified source. Can be repeated. Global variables overwrite other variables with same name
    #[arg(short, long, value_names = ["name", "value"])]
    pub var: Option<Vec<String>>,
}
