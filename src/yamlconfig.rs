use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::PathBuf;

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
    pub fn new(filename: &PathBuf) -> Result<Config, Box<dyn Error>> {
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
