use clap::Parser;
use log::error;
use septic_config_generator::{read_source, Config};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

#[derive(clap::Parser)]
#[command(version, about, long_about=None)]
#[command(next_line_help = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Generate SEPTIC config
    Make(MakeArgs),
    /// Show difference between two text files
    Diff,
}

#[derive(clap::Parser)]
struct MakeArgs {
    /// The yaml config file
    config_file: PathBuf,
    /// Name of output file (overrides config option "outputfile")
    #[arg(short, long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Do not prompt for verification of output file before overwriting original (overrides config option "verifycontent")
    #[arg(short, long = "no-verify")]
    noverify: bool,
    /// Only output warnings or errors
    #[arg(short, long)]
    silent: bool,
    /// Gllobal variable to use for all templates, also those without specified source. Can be repeated. Global variables overwrite other variables with same name
    #[arg(short, long)]
    var: Option<Vec<String>>,
}

fn main() {
    let args = Cli::parse();
    match args.command {
        Commands::Make(make_args) => {
            let filename = make_args.config_file;

            let cfg = Config::new(&filename).unwrap_or_else(|e| {
                error!("Problem reading '{}': {}", &filename.display(), e);
                process::exit(1)
            });

            let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

            for source in &cfg.sources {
                let source_data = read_source(source).unwrap_or_else(|e| {
                    error!("Problem reading source file '{}': {}", source.filename, e);
                    process::exit(1);
                });
                all_source_data.insert(source.id.to_string(), source_data);
            }

            // println!("{:?}", config);
            println!("{:?}", all_source_data);
        }
        Commands::Diff => todo!(),
    }
}
