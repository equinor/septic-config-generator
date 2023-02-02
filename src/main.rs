use calamine::DataType;
use clap::Parser;
use septic_config_generator::{args, config::Config, datasource};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;

fn ensure_has_extension(filename: &PathBuf, extension: &str) -> PathBuf {
    let mut file = filename.clone();
    if file.extension().is_none() {
        file.set_extension(extension);
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

fn cmd_make(cfg_file: &PathBuf, var_list: &Vec<String>) {
    let cfg_file = ensure_has_extension(&cfg_file, "yaml");

    let globals = var_list
        .chunks(2)
        .map(|chunk| (chunk[0].to_string(), chunk[1].to_string()))
        .collect::<HashMap<String, String>>();

    let cfg = Config::new(&cfg_file).unwrap_or_else(|e| {
        eprintln!("Problem reading '{}': {}", &cfg_file.display(), e);
        process::exit(1)
    });

    let mut all_source_data: HashMap<String, HashMap<String, HashMap<String, DataType>>> =
        HashMap::new();

    for source in &cfg.sources {
        let mut path = PathBuf::from(cfg_file.parent().unwrap());
        path.push(&source.filename);

        let source_data = datasource::read(&path, &source.sheet).unwrap_or_else(|e| {
            eprintln!("Problem reading source file '{}': {}", path.display(), e);
            process::exit(1);
        });
        all_source_data.insert(source.id.to_string(), source_data);
    }

    println!("{:?}", all_source_data);
    println!("{:?}", all_source_data["main"]["D02"]);
    println!("{:?}", globals);
}

fn main() {
    let args = args::Cli::parse();

    match args.command {
        args::Commands::Make(make_args) => {
            cmd_make(&make_args.config_file, &make_args.var.unwrap_or_default());
        }
        args::Commands::Diff => todo!(),
    }
}
