use clap::Parser;
use log::error;
use septic_config_generator::{argparser, excelsource, yamlconfig};
use std::collections::HashMap;
use std::process;

fn main() {
    let args = argparser::Cli::parse();
    match args.command {
        argparser::Commands::Make(make_args) => {
            let filename = make_args.config_file;

            let cfg = yamlconfig::Config::new(&filename).unwrap_or_else(|e| {
                error!("Problem reading '{}': {}", &filename.display(), e);
                process::exit(1)
            });

            let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

            for source in &cfg.sources {
                let source_data = excelsource::read(source).unwrap_or_else(|e| {
                    error!("Problem reading source file '{}': {}", source.filename, e);
                    process::exit(1);
                });
                all_source_data.insert(source.id.to_string(), source_data);
            }

            // println!("{:?}", config);
            println!("{:?}", all_source_data);
        }
        argparser::Commands::Diff => todo!(),
    }
}
