use anyhow::{bail, Result};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

const fn _default_true() -> bool {
    true
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub outputfile: Option<String>,
    pub templatepath: String,
    #[serde(default = "_default_true")]
    pub adjustspacing: bool,
    #[serde(default = "_default_true")]
    pub verifycontent: bool,
    pub counters: Option<Vec<Counter>>,
    pub sources: Vec<Source>,
    pub layout: Vec<Template>,
}

impl Config {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(filename: &Path) -> Result<Self> {
        let content = fs::read_to_string(filename)?;
        let cfg: Self = serde_yaml::from_str(&content)?;

        for source in &cfg.sources {
            validate_source(source)?;
        }

        Ok(cfg)
    }
}

fn validate_source(source: &Source) -> Result<()> {
    let extension = Path::new(&source.filename).extension();
    match extension {
        Some(ext) if ext == "xlsx" => {
            if source.sheet.is_none() {
                bail!("missing field 'sheet' for .xlsx source {}", source.filename);
            }
            if source.delimiter.is_some() {
                bail!(
                    "field 'delimiter' invalid for .xlsx source {}",
                    source.filename
                );
            }
        }
        Some(ext) if ext == "csv" => {
            if source.sheet.is_some() {
                bail!("field 'sheet' invalid for .csv source {}", source.filename);
            }
        }
        _ => {
            bail!("invalid file extension for source '{}'", source.filename);
        }
    }

    Ok(())
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    pub name: String,
    #[serde(default)]
    pub value: Option<i32>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub filename: String,
    pub id: String,
    pub sheet: Option<String>,
    pub delimiter: Option<char>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
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

    pub fn exclude_set(&self) -> HashSet<String> {
        self.exclude.as_ref().map_or_else(HashSet::new, |exclude| {
            exclude.iter().cloned().collect::<HashSet<String>>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_yaml(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{}", content).unwrap();
        temp_file
    }

    #[test]
    fn config_read_valid_content() {
        let content = r#"
outputfile: test.cnfg
templatepath: templates
adjustspacing: true
verifycontent: true
sources:
  - filename: test.csv
    id: main
counters:
  - name: mycounter
    value: 267
layout:
  - name: template1.cnfg
    source: main
    include:
      - one
    exclude:
      - two
"#;
        let temp_file = create_temp_yaml(content);
        let config = Config::new(temp_file.path());
        assert!(config.is_ok())
    }

    #[test]
    fn config_fail_read_on_invalid_yaml() {
        let temp_file = create_temp_yaml("random:");
        let config = Config::new(temp_file.path());
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("unknown field"));
    }

    #[test]
    fn config_fail_read_on_missing_config_file() {
        let config = Config::new(Path::new("nonexistent_file.yaml"));
        assert!(config.is_err());
        assert!(config.unwrap_err().to_string().contains("No such file"));
    }

    #[test]
    fn validate_source_good_xlsx() {
        let source = Source {
            filename: "data.xlsx".to_string(),
            id: "id".to_string(),
            sheet: Some("sheet".to_string()),
            ..Default::default()
        };
        assert!(validate_source(&source).is_ok())
    }

    #[test]
    fn fail_validate_source_xlsx_no_sheet() {
        let source = Source {
            filename: "data.xlsx".to_string(),
            id: "id".to_string(),
            ..Default::default()
        };
        let result = validate_source(&source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("missing field 'sheet'"))
    }

    #[test]
    fn fail_validate_source_xlsx_with_delimiter() {
        let source = Source {
            filename: "data.xlsx".to_string(),
            id: "id".to_string(),
            sheet: Some("sheet".to_string()),
            delimiter: Some(':'),
        };
        let result = validate_source(&source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("field 'delimiter' invalid"))
    }

    #[test]
    fn validate_source_good_csv() {
        let source = Source {
            filename: "data.csv".to_string(),
            id: "id".to_string(),
            delimiter: Some(':'),
            ..Default::default()
        };
        assert!(validate_source(&source).is_ok())
    }

    #[test]
    fn fail_validate_source_csv_with_sheet() {
        let source = Source {
            filename: "data.csv".to_string(),
            id: "id".to_string(),
            sheet: Some("sheet".to_string()),
            ..Default::default()
        };
        let result = validate_source(&source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("field 'sheet' invalid"))
    }

    #[test]
    fn fail_validate_unknown_source_type() {
        let source = Source {
            filename: "data.whatever".to_string(),
            id: "id".to_string(),
            ..Default::default()
        };
        let result = validate_source(&source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("invalid file extension"))
    }
}
