use septic_config_generator::{read_source, Config};
use std::collections::HashMap;
use std::process;

fn main() {
    let filename = String::from("basic example/example.yaml");

    let cfg = Config::new(&filename).unwrap_or_else(|e| {
        eprintln!("Problem reading '{}': {}", filename, e);
        process::exit(1)
    });

    let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

    for source in &cfg.sources {
        let source_data = read_source(source).unwrap_or_else(|e| {
            eprintln!("Problem reading source file '{}': {}", source.filename, e);
            process::exit(1);
        });
        all_source_data.insert(source.id.to_string(), source_data);
    }

    // println!("{:?}", config);
    println!("{:?}", all_source_data);
}
