use anyhow::{Result, bail};
use minijinja::{Environment, context};
use serde::Deserialize;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use crate::datasource::DataSourceRows;

const fn _default_true() -> bool {
    true
}

fn _default_encoding() -> String {
    "Windows-1252".to_string()
}

#[derive(Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct Config {
    pub outputfile: Option<String>,
    #[serde(default = "_default_encoding")]
    pub encoding: String,
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

        validate_encoding(&cfg.encoding)?;

        Ok(cfg)
    }
}

fn validate_encoding(encoding: &str) -> Result<()> {
    if encoding_rs::Encoding::for_label(encoding.as_bytes()).is_none() {
        bail!("invalid encoding '{}'", encoding);
    }
    Ok(())
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
    #[serde(rename = "if")]
    condition: String,
    #[serde(rename = "then")]
    items: Option<Vec<String>>,
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

pub enum IncludeExclude {
    Include,
    Exclude,
}

impl Template {
    pub fn include_exclude_set(
        &self,
        env: &Environment,
        source_data: &DataSourceRows,
        include_exclude: IncludeExclude,
    ) -> Result<HashSet<String>, minijinja::Error> {
        let mut result: HashSet<String> = HashSet::new();
        let items = match include_exclude {
            IncludeExclude::Include => self.include.as_ref(),
            IncludeExclude::Exclude => self.exclude.as_ref(),
        };

        if let Some(items) = items {
            for inc_item in items {
                let mut matched = false;
                match inc_item {
                    Include::Element(elem) => {
                        result.insert(elem.clone());
                    }
                    Include::Conditional(elem) => {
                        let expr = env.compile_expression(elem.condition.as_str())?;
                        if let Some(items) = &elem.items {
                            let eval = expr.eval(context! {})?;
                            if eval.is_true() {
                                matched = true;
                                result.extend(items.clone());
                            }
                        } else {
                            for (key, row) in source_data {
                                let eval = expr.eval(row)?;
                                if eval.is_true() {
                                    matched = true;
                                    result.insert(key.clone());
                                }
                            }
                        }
                        if matched && !elem.continue_.unwrap_or(false) {
                            break;
                        }
                    }
                }
            }
        }
        Ok(result)
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
encoding: Windows-1252
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
    fn config_correct_default_values() {
        let content = r#"
outputfile: test.cnfg
templatepath: templates
sources:
  - filename: test.csv
    id: main
layout:
  - name: template1.cnfg
    source: main
"#;
        let temp_file = create_temp_yaml(content);
        let config = Config::new(temp_file.path());
        assert!(config.is_ok());
        let config = config.unwrap();
        assert!(config.adjustspacing);
        assert!(config.verifycontent);
        assert_eq!(config.encoding, "Windows-1252");
    }

    #[test]
    fn fail_validate_encoding_unknown() {
        let result = validate_encoding("unknown");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("invalid encoding"));
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("missing field 'sheet'")
        )
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("field 'delimiter' invalid")
        )
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("field 'sheet' invalid")
        )
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("in multi-file source 'id' must be .csv"),
        )
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("field 'sheet' invalid")
        )
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("invalid file extension")
        )
    }

    fn create_template_with_includes(
        include_condition: &str,
        include_continue: Option<bool>,
    ) -> Template {
        Template {
            name: "templatename".to_string(),
            source: None,
            include: Some(vec![
                Include::Element("one".to_string()),
                Include::Conditional(IncludeConditional {
                    items: Some(vec!["two".to_string()]),
                    condition: include_condition.to_string(),
                    continue_: include_continue,
                }),
                Include::Element("three".to_string()),
            ]),
            ..Default::default()
        }
    }

    #[test]
    fn include_exclude_set_break_when_condition_matches() {
        let template = create_template_with_includes("true", None);
        let res = template
            .include_exclude_set(
                &minijinja::Environment::new(),
                &IndexMap::new(),
                IncludeExclude::Include,
            )
            .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "two".to_string()]));
    }

    #[test]
    fn include_exclude_set_continue_when_condition_not_matched() {
        let template = create_template_with_includes("false", None);
        let res = template
            .include_exclude_set(
                &minijinja::Environment::new(),
                &IndexMap::new(),
                IncludeExclude::Include,
            )
            .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "three".to_string()]));
    }

    #[test]
    fn include_exclude_set_continue_when_continue_true() {
        let template = create_template_with_includes("true", Some(true));
        let res = template
            .include_exclude_set(
                &minijinja::Environment::new(),
                &IndexMap::new(),
                IncludeExclude::Include,
            )
            .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "two".to_string(), "three".to_string()]));
    }
}
