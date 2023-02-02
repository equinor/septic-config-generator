use calamine::DataType;
use clap::Parser;
use minijinja::Environment;
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

fn add_globals(env: &mut Environment, globals: &Vec<String>) {
    for chunk in globals.chunks(2) {
        let (key, val) = (chunk[0].to_string(), chunk[1].to_string());
        match val.as_str() {
            "true" => env.add_global(key, true),
            "false" => env.add_global(key, false),
            _ => match val.parse::<i64>() {
                Ok(i) => env.add_global(key, i),
                Err(_) => match val.parse::<f64>() {
                    Ok(f) => env.add_global(key, f),
                    Err(_) => env.add_global(key, val.to_owned()),
                },
            },
        }
    }
}

fn cmd_make(cfg_file: &PathBuf, globals: &Vec<String>) {
    let cfg_file = ensure_has_extension(&cfg_file, "yaml");

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

    let mut env = Environment::new();

    add_globals(&mut env, globals);

    for template in cfg.layout {
        println!("{}", template.name);
    }
    // println!("{:?}", all_source_data);
    // println!("{:?}", all_source_data["main"]["D02"]);
    println!("{:?}", env);
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
