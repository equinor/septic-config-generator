use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

pub fn read_config(cfg_file: &Path) -> Result<(Config, PathBuf), Box<dyn Error>> {
    let relative_root = PathBuf::from(cfg_file.parent().unwrap());
    let cfg = Config::new(cfg_file)?;

    Ok((cfg, relative_root))
}
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::prelude::*;
    use tempfile::tempdir;

    #[test]
    fn test_read_config_invalid_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();

        // let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "random").unwrap();
        let result = read_config(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid type"));
    }
    #[test]
    fn test_read_config_invalid_yaml() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();

        // let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "random: ").unwrap();
        let result = read_config(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing field"));
    }

    #[test]
    fn test_read_config_file_does_not_exist() {
        let result = read_config(Path::new("nonexistent_file.yaml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2)"));
    }
}
