use anyhow::{bail, Result};
use minijinja::{context, Environment};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::datasource::DataSourceRows;

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
    match &source.filename {
        Filename::Single(filename) => {
            let extension = Path::new(filename).extension();
            match extension {
                Some(ext) if ext == "xlsx" => {
                    if source.sheet.is_none() {
                        bail!("missing field 'sheet' for .xlsx source '{}'", source.id);
                    }
                    if source.delimiter.is_some() {
                        bail!("field 'delimiter' invalid for .xlsx source '{}'", source.id);
                    }
                }
                Some(ext) if ext == "csv" => {
                    if source.sheet.is_some() {
                        bail!("field 'sheet' invalid for .csv source '{}'", source.id);
                    }
                }
                _ => {
                    bail!("invalid file extension for source '{}'", source.id);
                }
            }
        }
        Filename::Multiple(filenames) => {
            if filenames.iter().any(|filename| {
                Path::new(filename).extension().and_then(|s| s.to_str()) != Some("csv")
            }) {
                bail!(
                    "All files in multi-file source '{}' must be .csv",
                    source.id
                );
            }
            if source.sheet.is_some() {
                bail!("field 'sheet' invalid for .csv sources in '{}'", source.id);
            }
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

#[derive(Deserialize, Debug)]
#[serde(untagged)] // Allows for multiple representations of the data
pub enum Filename {
    Single(String),
    Multiple(Vec<String>),
}

impl Default for Filename {
    fn default() -> Self {
        Filename::Single(String::new())
    }
}

impl From<&str> for Filename {
    fn from(s: &str) -> Filename {
        Filename::Single(s.to_string())
    }
}

impl From<Vec<&str>> for Filename {
    fn from(v: Vec<&str>) -> Filename {
        let owned_strings: Vec<String> = v.into_iter().map(|s| s.to_string()).collect();
        Filename::Multiple(owned_strings)
    }
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub filename: Filename,
    pub id: String,
    pub sheet: Option<String>,
    pub delimiter: Option<char>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct IncludeConditional {
    items: Option<Vec<String>>,
    #[serde(rename = "if")]
    condition: String,
    #[serde(rename = "continue")]
    continue_: Option<bool>,
}

#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Include {
    Element(String),
    Conditional(IncludeConditional),
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Template {
    pub name: String,
    pub source: Option<String>,
    pub include: Option<Vec<Include>>,
    pub exclude: Option<Vec<Include>>,
}

impl Template {
    pub fn include_set(&self, env: &Environment, source_data: &DataSourceRows) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();
        if let Some(includes) = self.include.as_ref() {
            for inc_item in includes {
                match inc_item {
                    Include::Element(elem) => {
                        result.insert(elem.clone());
                    }
                    Include::Conditional(elem) => match &elem.items {
                        Some(items) => {
                            let expr = env.compile_expression(elem.condition.as_str()).unwrap();
                            let eval = expr.eval(context! {}).unwrap();
                            if eval.is_true() {
                                result.extend(items.clone());
                                if matches!(elem.continue_, Some(false) | None) {
                                    return result;
                                }
                            }
                        }
                        None => {
                            for (key, row) in source_data {
                                let expr = env.compile_expression(elem.condition.as_str()).unwrap();
                                let eval = expr.eval(row).unwrap();
                                if eval.is_true() {
                                    result.insert(key.clone());
                                }
                            }
                        }
                    },
                }
            }
        }
        result
    }

    pub fn exclude_set(&self, env: &Environment, source_data: &DataSourceRows) -> HashSet<String> {
        let mut result: HashSet<String> = HashSet::new();
        self.source.as_ref().unwrap();
        if let Some(excludes) = self.exclude.as_ref() {
            for exc_item in excludes {
                match exc_item {
                    Include::Element(elem) => {
                        result.insert(elem.clone());
                    }
                    Include::Conditional(elem) => match &elem.items {
                        Some(items) => {
                            let expr = env.compile_expression(elem.condition.as_str()).unwrap();
                            let eval = expr.eval(context! {}).unwrap();
                            if eval.is_true() {
                                result.extend(items.clone());
                            }
                        }
                        None => {
                            for (key, row) in source_data {
                                let expr = env.compile_expression(elem.condition.as_str()).unwrap();
                                let eval = expr.eval(row).unwrap();
                                if eval.is_true() {
                                    result.insert(key.clone());
                                }
                            }
                        }
                    },
                }
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
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
        assert!(config.unwrap_err().to_string().contains("os error 2"));
    }

    #[test]
    fn validate_source_good_xlsx() {
        let source = Source {
            filename: "data.xlsx".into(),
            id: "id".to_string(),
            sheet: Some("sheet".to_string()),
            ..Default::default()
        };
        assert!(validate_source(&source).is_ok())
    }

    #[test]
    fn fail_validate_source_xlsx_no_sheet() {
        let source = Source {
            filename: "data.xlsx".into(),
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
            filename: "data.xlsx".into(),
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
            filename: "data.csv".into(),
            id: "id".to_string(),
            delimiter: Some(':'),
            ..Default::default()
        };
        assert!(validate_source(&source).is_ok())
    }

    #[test]
    fn fail_validate_source_csv_with_sheet() {
        let source = Source {
            filename: "data.csv".into(),
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
    fn validate_multisource_good_csv() {
        let source = Source {
            filename: vec!["data1.csv", "data2.csv"].into(),
            id: "id".to_string(),
            delimiter: Some(':'),
            ..Default::default()
        };
        assert!(validate_source(&source).is_ok())
    }

    #[test]
    fn fail_validate_multisource_with_xlsx() {
        let source = Source {
            filename: vec!["data1.csv", "data2.xlsx"].into(),
            id: "id".to_string(),
            delimiter: Some(':'),
            ..Default::default()
        };
        let result = validate_source(&source);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("in multi-file source 'id' must be .csv"),)
    }

    #[test]
    fn fail_validate_multisource_with_sheet() {
        let source = Source {
            filename: vec!["data1.csv", "data2.csv"].into(),
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
            filename: "data.whatever".into(),
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
