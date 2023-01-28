use calamine::{open_workbook_auto, Reader}; // Use open_workbook instead of open_workbook_auto
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

fn main() {
    let cfg: Config = serde_yaml::from_str(include_str!("../basic example/example.yaml")).unwrap();
    let mut all_source_data: HashMap<String, Vec<HashMap<String, String>>> = HashMap::new();

    for source in cfg.sources {
        let mut workbook = open_workbook_auto(&format!("{}/{}", "basic example", source.filename))
            .expect("Cannot open file");
        if let Some(Ok(sheet)) = workbook.worksheet_range(&source.sheet) {
            let row_headers = sheet.rows().next().unwrap();
            let mut excel_data = vec![];
            for row in sheet.rows().skip(1) {
                let mut row_data = HashMap::new();
                for (header, cell) in row_headers.iter().zip(row.iter()) {
                    row_data.insert(
                        String::from(header.get_string().unwrap()),
                        String::from(cell.get_string().unwrap()),
                    );
                }
                excel_data.push(row_data);
            }
            all_source_data.insert(source.id, excel_data);
        }
    }

    // println!("{:?}", config);
    println!("{:?}", all_source_data);
    println!("{:?}", all_source_data["main"]);
}
