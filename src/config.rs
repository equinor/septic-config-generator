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
    pub fn new(filename: &PathBuf) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(filename)?;
        let cfg: Self = serde_yaml::from_str(&content)?;
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
        self.include.as_ref().map_or_else(HashSet::new, |include| {
            include.iter().cloned().collect::<HashSet<String>>()
        })
    }
}

impl Template {
    pub fn exclude_set(&self) -> HashSet<String> {
        self.exclude.as_ref().map_or_else(HashSet::new, |exclude| {
            exclude.iter().cloned().collect::<HashSet<String>>()
        })
    }
}
