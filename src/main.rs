use calamine::{open_workbook, Error, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
struct Config {
    outputfile: String,
    templatepath: String,
    masterpath: String,
    masterkey: String,
    verifycontent: String,
    sources: Vec<Source>,
    layout: Vec<Layout>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Source {
    filename: String,
    id: String,
    sheet: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct Layout {
    name: String,
    source: Option<String>,
    include: Option<Vec<String>>,
}

fn read_source(source: &Source) -> Result<Vec<HashMap<String, String>>, Error> {
    let path = format!("basic example/{}", source.filename);
    let mut workbook: Xlsx<_> = open_workbook(path)?;
    let range = workbook
        .worksheet_range(&source.sheet)
        .ok_or(Error::Msg("Cannot find sheet"))??;

    let row_headers = range.rows().next().unwrap();
    let mut data = vec![];
    for row in range.rows().skip(1) {
        let mut row_data = HashMap::new();
        for (header, cell) in row_headers.iter().zip(row.iter()) {
            row_data.insert(
                String::from(header.get_string().unwrap()),
                String::from(cell.get_string().unwrap()),
            );
        }
        data.push(row_data);
    }
    Ok(data)
}

fn main() {
    let cfg: Config = serde_yaml::from_str(include_str!("../basic example/example.yaml")).unwrap();
    let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

    for source in &cfg.sources {
        let source_data = read_source(source).unwrap();
        all_source_data.insert(source.id.to_string(), source_data);
    }

    // println!("{:?}", config);
    println!("{:?}", all_source_data);
}
