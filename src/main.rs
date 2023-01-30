use calamine::{open_workbook, Reader, Xlsx};
use septic_config_generator::{Config, Source};
use std::collections::HashMap;
use std::error::Error;
use std::process;

fn read_source(source: &Source) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
    let path = format!("basic example/{}", source.filename);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range(&source.sheet)
        .ok_or_else(|| format!("Cannot find sheet '{}'", source.sheet))??;

    let row_headers = range.rows().next().unwrap();
    let data = range
        .rows()
        .skip(1)
        .map(|row| {
            row_headers
                .iter()
                .zip(row.iter())
                .map(|(header, cell)| {
                    (
                        header.get_string().unwrap().to_string(),
                        cell.get_string().unwrap().to_string(),
                    )
                })
                .collect::<HashMap<String, String>>()
        })
        .collect::<Vec<HashMap<String, String>>>();
    Ok(data)
}

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
