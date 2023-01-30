use calamine::{open_workbook, Reader, Xlsx};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub outputfile: String,
    pub templatepath: String,
    pub masterpath: String,
    pub masterkey: String,
    pub verifycontent: String,
    pub sources: Vec<Source>,
    pub layout: Vec<Layout>,
}

impl Config {
    pub fn new(filename: &String) -> Result<Config, Box<dyn Error>> {
        let content = fs::read_to_string(filename)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Source {
    pub filename: String,
    pub id: String,
    pub sheet: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Layout {
    pub name: String,
    pub source: Option<String>,
    pub include: Option<Vec<String>>,
}

pub fn read_source(source: &Source) -> Result<Vec<HashMap<String, String>>, Box<dyn Error>> {
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
