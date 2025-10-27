use anyhow::{Result, bail};
use minijinja::{Environment, context};
use schemars::JsonSchema;
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

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(untagged)] // Allows for multiple representations of the data
pub enum Filename {
    /// Single filename as a string
    Single(String),
    /// Multiple filenames as a list of strings
    Multiple(Vec<String>),
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Counter {
    /// Counter name
    pub name: String,
    #[serde(default)]
    /// Initial value for the counter, defaults to 0
    pub value: Option<i32>,
}

fn deserialize_string_or_vec_as_vec<'de, D>(
    deserializer: D,
) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum StringOrVec {
        String(String),
        Vec(Vec<String>),
    }

    let opt: Option<StringOrVec> = Option::deserialize(deserializer)?;
    Ok(opt.map(|items| match items {
        StringOrVec::String(s) => vec![s],
        StringOrVec::Vec(v) => v,
    }))
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct IncludeConditional {
    /// The condition to evaluate. Uses MiniJinja syntax.
    #[serde(rename = "if")]
    pub condition: String,
    /// List of items (or single item) to include if the condition is true (can be a single string or array of strings)
    #[serde(rename = "then", deserialize_with = "deserialize_string_or_vec_as_vec")]
    pub items: Option<Vec<String>>,
    /// Whether to continue evaluating further conditions after this one
    #[serde(rename = "continue")]
    pub continue_: Option<bool>,
}

#[derive(Deserialize, Debug, JsonSchema)]
#[serde(untagged)]
pub enum Include {
    /// Include a single element by name
    Element(String),
    /// Include a conditional element with a condition and optional items
    Conditional(IncludeConditional),
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Source {
    /// The filename(s) of the source data
    pub filename: Filename,
    /// Optional list of rows from source to include globally
    pub include: Option<Vec<Include>>,
    /// Optional list of rows from source to exclude globally
    pub exclude: Option<Vec<Include>>,
    /// The unique identifier for this source
    pub id: String,
    /// Optional sheet name for .xlsx files
    pub sheet: Option<String>,
    /// Optional delimiter for .csv files
    pub delimiter: Option<char>,
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Template {
    /// The name of the template file
    pub name: String,
    /// Optional source id to iterate over for this template
    pub source: Option<String>,
    /// Optional list of fields from source to include in iteration
    pub include: Option<Vec<Include>>,
    /// Optional list of fields from source to exclude in iteration
    pub exclude: Option<Vec<Include>>,
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
pub struct Drawio {
    /// The draw.io file to process
    pub input: String,
    /// Optional output PNG file path. If not provided, the output will have the same name as input but with .png extension
    pub pngoutput: Option<String>,
    /// Optional output CSV file path for components. If not provided, the output will have the same name as input but with _components.csv extension
    pub csvoutput: Option<String>,
}

#[derive(Deserialize, Debug, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Septic Config Generator Configuration")]
pub struct Config {
    /// The file that will be generated. Writes to stdout if not specified.
    pub outputfile: Option<String>,
    #[serde(default = "_default_encoding")]
    /// The encoding to use for template files and the outputfile. Use any label specified in https://encoding.spec.whatwg.org/#concept-encoding-get
    pub encoding: String,
    /// The directory that contains all template files
    pub templatepath: String,
    #[serde(default = "_default_true")]
    /// Whether to ensure exactly one newline between rendered template files
    pub adjustspacing: bool,
    #[serde(default = "_default_true")]
    /// Whether to warn about differences from an already existing rendered file
    pub verifycontent: bool,
    /// List of global auto-incrementing counters
    pub counters: Option<Vec<Counter>>,
    /// List of source file configurations
    pub sources: Option<Vec<Source>>,
    /// List of templates in the order they should be rendered
    pub layout: Vec<Template>,
    /// List of .drawio files to process
    pub drawio: Option<Vec<Drawio>>,
}

pub trait RowFiltering {
    fn get_include(&self) -> &Option<Vec<Include>>;

    fn get_exclude(&self) -> &Option<Vec<Include>>;

    fn apply_filters(
        &self,
        source_rows: &DataSourceRows,
        env: &Environment,
    ) -> anyhow::Result<DataSourceRows> {
        let mut items_set: HashSet<String> = source_rows.keys().cloned().collect();

        if let Some(include_list) = self.get_include() {
            let include_set = include_exclude_set(include_list, env, source_rows)?;
            items_set = items_set.intersection(&include_set).cloned().collect();
        }

        if let Some(exclude_list) = self.get_exclude() {
            let exclude_set = include_exclude_set(exclude_list, env, source_rows)?;
            items_set = items_set.difference(&exclude_set).cloned().collect();
        }

        let mut filtered_rows: DataSourceRows = DataSourceRows::new();
        for (key, row) in source_rows {
            if items_set.contains(key) {
                filtered_rows.insert(key.clone(), row.clone());
            }
        }

        Ok(filtered_rows)
    }
}

pub enum IncludeExclude {
    Include,
    Exclude,
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

macro_rules! impl_row_filtering {
    ($type:ty) => {
        impl RowFiltering for $type {
            fn get_include(&self) -> &Option<Vec<Include>> {
                &self.include
            }
            fn get_exclude(&self) -> &Option<Vec<Include>> {
                &self.exclude
            }
        }
    };
}

impl_row_filtering!(Source);
impl_row_filtering!(Template);

impl Config {
    #[allow(clippy::missing_errors_doc)]
    pub fn new(filename: &Path) -> Result<Self> {
        let content = fs::read_to_string(filename)?;
        let cfg: Self = serde_yaml::from_str(&content)?;

        if let Some(sources) = &cfg.sources {
            for source in sources {
                validate_source(source)?;
            }
        }

        validate_encoding(&cfg.encoding)?;

        Ok(cfg)
    }
}

pub fn include_exclude_set(
    items: &Vec<Include>,
    env: &Environment,
    source_data: &DataSourceRows,
) -> Result<HashSet<String>, minijinja::Error> {
    let mut result: HashSet<String> = HashSet::new();

    for item in items {
        let mut matched = false;
        match item {
            Include::Element(val) => {
                result.insert(val.clone());
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

    Ok(result)
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

#[cfg(test)]
mod tests {
    use super::*;
    use indexmap::IndexMap;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_yaml(content: &str) -> NamedTempFile {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "{content}").unwrap();
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
            ..Default::default()
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
        let res = include_exclude_set(
            template.include.as_ref().unwrap(),
            &minijinja::Environment::new(),
            &IndexMap::new(),
        )
        .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "two".to_string()]));
    }

    #[test]
    fn include_exclude_set_continue_when_condition_not_matched() {
        let template = create_template_with_includes("false", None);
        let res = include_exclude_set(
            template.include.as_ref().unwrap(),
            &minijinja::Environment::new(),
            &IndexMap::new(),
        )
        .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "three".to_string()]));
    }

    #[test]
    fn include_exclude_set_continue_when_continue_true() {
        let template = create_template_with_includes("true", Some(true));
        let res = include_exclude_set(
            template.include.as_ref().unwrap(),
            &minijinja::Environment::new(),
            &IndexMap::new(),
        )
        .unwrap();
        assert!(res == HashSet::from(["one".to_string(), "two".to_string(), "three".to_string()]));
    }
}
