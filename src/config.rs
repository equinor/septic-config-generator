use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub outputfile: Option<String>,
    pub templatepath: String,
    pub masterpath: Option<String>,
    pub masterkey: Option<String>,
    #[serde(default = "_default_true")]
    pub verifycontent: bool,
    pub sources: Vec<Source>,
    pub layout: Vec<Template>,
}

const fn _default_true() -> bool {
    true
}

impl Config {
    pub fn new(filename: &PathBuf) -> Result<Config, Box<dyn Error>> {
        let content = fs::read_to_string(filename)?;
        let cfg: Config = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[derive(Deserialize, Debug)]
pub struct Source {
    pub filename: String,
    pub id: String,
    pub sheet: String,
}

#[derive(Deserialize, Debug)]
pub struct Template {
    pub name: String,
    pub source: Option<String>,
    pub include: Option<Vec<String>>,
    pub exclude: Option<Vec<String>>,
}

impl Template {
    pub fn include_set(&self) -> HashSet<String> {
        match &self.include {
            Some(include) => HashSet::<String>::from_iter(include.iter().cloned()),
            None => HashSet::new(),
        }
    }
}

impl Template {
    pub fn exclude_set(&self) -> HashSet<String> {
        match &self.exclude {
            Some(exclude) => HashSet::<String>::from_iter(exclude.iter().cloned()),
            None => HashSet::new(),
        }
    }
}
