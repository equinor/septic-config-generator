use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::path::Path;

const fn _default_true() -> bool {
    true
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

impl Config {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(filename: &Path) -> Result<Self, Box<dyn Error>> {
        let content = fs::read_to_string(filename)?;
        let mut cfg: Self = serde_yaml::from_str(&content)?;

        for source in &mut cfg.sources {
            validate_source(source)?
        }

        Ok(cfg)
    }
}

fn validate_source(source: &mut Source) -> Result<(), Box<dyn Error>> {
    let extension = Path::new(&source.filename).extension();
    match extension {
        Some(ext) if ext == "xlsx" => {
            if source.sheet.is_none() {
                return Err(format!(
                    "missing field 'sheet' for .xlsx file with source id '{}'",
                    source.id
                )
                .into());
            }
            if source.delimiter.is_some() {
                return Err(format!(
                    "field 'delimiter' cannot be specified for .xlsx file with source id '{}'",
                    source.id
                )
                .into());
            }
        }
        Some(ext) if ext == "csv" => {
            if source.sheet.is_some() {
                return Err(format!(
                    "field 'sheet' cannot be specified for .csv file with source id '{}'",
                    source.id
                )
                .into());
            }
            if source.delimiter.is_none() {
                source.delimiter = Some(';');
            }
        }
        _ => {
            return Err("Invalid file extension".into());
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug, Default)]
pub struct Source {
    pub filename: String,
    pub id: String,
    pub sheet: Option<String>,
    pub delimiter: Option<char>,
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
        let result = Config::new(&file_path);
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
        let result = Config::new(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing field"));
    }

    #[test]
    fn test_read_config_file_does_not_exist() {
        let result = Config::new(Path::new("nonexistent_file.yaml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2)"));
    }
}
