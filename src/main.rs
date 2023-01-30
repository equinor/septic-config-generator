use clap::Parser;
use log::error;
use septic_config_generator::{args, config::Config, datasource};
use std::collections::HashMap;
use std::process;

fn main() {
    let args = args::Cli::parse();

    match args.command {
        args::Commands::Make(make_args) => {
            let var_list = make_args.var.unwrap_or_default();

            let var_map = var_list
                .chunks(2)
                .map(|chunk| (chunk[0].to_string(), chunk[1].to_string()))
                .collect::<HashMap<String, String>>();

            let filename = make_args.config_file;

            let cfg = Config::new(&filename).unwrap_or_else(|e| {
                error!("Problem reading '{}': {}", &filename.display(), e);
                process::exit(1)
            });

            let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

            for source in &cfg.sources {
                let source_data =
                    datasource::read(&source.filename, &source.sheet).unwrap_or_else(|e| {
                        error!("Problem reading source file '{}': {}", source.filename, e);
                        process::exit(1);
                    });
                all_source_data.insert(source.id.to_string(), source_data);
            }

            // println!("{:?}", config);
            println!("{:?}", all_source_data);
            println!("{:?}", var_map);
        }
        args::Commands::Diff => todo!(),
    }
}