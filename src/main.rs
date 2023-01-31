use calamine::DataType;
use clap::Parser;
use septic_config_generator::{args, config::Config, datasource};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

fn add_missing_yaml_extension(filename: &PathBuf) -> PathBuf {
    let mut file = filename.clone();
    if file.extension().is_none() {
        file.set_extension("yaml");
    }
    file
}

fn _merge_maps(
    map1: &HashMap<String, String>,
    map2: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = map1.clone();
    merged.extend(map2.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    merged
}

fn main() {
    let args = args::Cli::parse();

    match args.command {
        args::Commands::Make(make_args) => {
            let var_list = make_args.var.unwrap_or_default();

            let var_map = var_list
                .chunks(2)
                .map(|chunk| (chunk[0].to_string(), chunk[1].to_string()))
                .collect::<HashMap<String, String>>();

            let filename = add_missing_yaml_extension(&make_args.config_file);

            let cfg = Config::new(&filename).unwrap_or_else(|e| {
                eprintln!("Problem reading '{}': {}", &filename.display(), e);
                process::exit(1)
            });
            // let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();
            let mut all_source_data: HashMap<String, HashMap<String, HashMap<String, DataType>>> =
                HashMap::new();

            for source in &cfg.sources {
                let source_data =
                    datasource::read(&source.filename, &source.sheet).unwrap_or_else(|e| {
                        eprintln!("Problem reading source file '{}': {}", source.filename, e);
                        process::exit(1);
                    });
                all_source_data.insert(source.id.to_string(), source_data);
            }

            // println!("{:?}", config);
            println!("{:?}", all_source_data);
            println!("{:?}", all_source_data["main"]["D02"]);
            println!("{:?}", var_map);
            // println!(
            //     "{:?}",
            //     merge_maps(&all_source_data["main"]["D02"], &var_map)
            // );
        }
        args::Commands::Diff => todo!(),
    }
}
