use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Deserialize, Debug, Default)]
pub struct Config {
    pub outputfile: Option<String>,
    pub templatepath: String,
    #[serde(default = "_default_true")]
    pub adjustspacing: bool,
    #[serde(default = "_default_true")]
    pub verifycontent: bool,
    pub sources: Vec<Source>,
    pub layout: Vec<Template>,
}

const fn _default_true() -> bool {
    true
}

impl Config {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(filename: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(filename)?;
        let cfg: Self = serde_yaml::from_str(&content)?;
        Ok(cfg)
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Source {
    pub filename: String,
    pub id: String,
    pub sheet: Option<String>,
}

#[derive(Deserialize, Debug, Default)]
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
