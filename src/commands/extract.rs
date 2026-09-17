use crate::config::{Config, ExtractionSource, Filename};
use crate::septic_cnfg;
use anyhow::{Context, Result, bail};
use clap::Parser;
use csv::{ReaderBuilder, Trim, WriterBuilder};
use indexmap::IndexMap;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct Extract {
    /// The yaml config file
    pub config_file: PathBuf,
    /// Septic config file to extract from (overrides extraction.from)
    pub source_file: Option<PathBuf>,
}

impl Extract {
    pub fn execute(&self) {
        if let Err(error) = cmd_extract(&self.config_file, self.source_file.as_deref()) {
            eprintln!("{error:#}");
            std::process::exit(2);
        }
    }
}

struct PathPattern {
    object_type: Option<String>,
    object_name: Regex,
    member: String,
    placeholders: Vec<String>,
    original: String,
}

struct ExtractedRow {
    label: String,
    values: Vec<Option<String>>,
}

struct PreparedExtraction {
    target: PathBuf,
    output: Vec<u8>,
    messages: Vec<String>,
}

#[derive(Debug)]
struct ExtractionResult {
    output: Vec<u8>,
    row_labels: Vec<String>,
    missing_values: Vec<String>,
}

fn cmd_extract(config_file: &Path, source_override: Option<&Path>) -> Result<()> {
    let mut config_file = config_file.to_path_buf();
    config_file
        .extension()
        .is_none()
        .then(|| config_file.set_extension("yaml"));
    let root = config_file.parent().unwrap_or_else(|| Path::new(""));
    let config = Config::new(&config_file)
        .with_context(|| format!("Problem reading '{}'", config_file.display()))?;
    let extraction = config
        .extraction
        .as_ref()
        .context("missing field 'extraction'")?;
    let encoding = encoding_rs::Encoding::for_label(config.encoding.as_bytes())
        .expect("Config::new validates encoding");

    let source_file = match source_override {
        Some(path) => path.to_path_buf(),
        None => root.join(
            extraction
                .from
                .as_deref()
                .context("missing field 'extraction.from' and no source file was provided")?,
        ),
    };
    let objects = load_objects(&source_file, encoding, &config.encoding)?;
    let mut prepared = Vec::new();
    for extraction_source in &extraction.sources {
        let result = extract_to_csv(extraction_source, &config, &objects)?;
        let target_source = config
            .sources
            .iter()
            .flatten()
            .find(|source| source.id == extraction_source.id)
            .expect("Config::new validates extraction source");
        let Filename::Single(target) = &target_source.filename else {
            unreachable!("Config::new validates extraction target type")
        };
        let target = root.join(target);
        let delimiter = target_source.delimiter.unwrap_or(';');
        let mut messages = if target.exists() {
            compare_row_labels(&read_row_labels(&target, delimiter)?, &result.row_labels)
        } else {
            Vec::new()
        };
        messages.extend(result.missing_values);
        prepared.push(PreparedExtraction {
            target,
            output: result.output,
            messages,
        });
    }

    for extraction in prepared {
        fs::write(&extraction.target, extraction.output)
            .with_context(|| format!("Problem writing '{}'", extraction.target.display()))?;
        for message in extraction.messages {
            eprintln!("{message}");
        }
    }
    Ok(())
}

fn load_objects(
    source_file: &Path,
    encoding: &'static encoding_rs::Encoding,
    encoding_name: &str,
) -> Result<Vec<septic_cnfg::Object>> {
    let bytes = fs::read(source_file)
        .with_context(|| format!("Problem reading '{}'", source_file.display()))?;
    let (contents, _, had_errors) = encoding.decode(&bytes);
    if had_errors {
        bail!(
            "Unable to decode '{}' as {}",
            source_file.display(),
            encoding_name
        );
    }
    septic_cnfg::parse(&contents)
}

fn extract_to_csv(
    extraction: &ExtractionSource,
    config: &Config,
    objects: &[septic_cnfg::Object],
) -> Result<ExtractionResult> {
    let patterns: Vec<_> = extraction
        .values
        .iter()
        .map(|value| compile_path(&value.path))
        .collect::<Result<_>>()?;
    let placeholders = &patterns[0].placeholders;
    if placeholders.is_empty() {
        bail!("extraction paths must contain at least one named placeholder");
    }
    let expected: HashSet<_> = placeholders.iter().collect();
    for pattern in &patterns[1..] {
        if pattern.placeholders.iter().collect::<HashSet<_>>() != expected {
            bail!("all extraction paths must use the same named placeholders");
        }
    }
    let row_label_pattern = Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    validate_row_label(&extraction.rowlabel.value, &expected, &row_label_pattern)?;

    let mut rows: IndexMap<Vec<String>, ExtractedRow> = IndexMap::new();
    for object in objects {
        for (column, pattern) in patterns.iter().enumerate() {
            if pattern
                .object_type
                .as_ref()
                .is_some_and(|object_type| object_type != &object.object_type)
            {
                continue;
            }
            let Some(captures) = pattern.object_name.captures(&object.name) else {
                continue;
            };
            let captured: HashMap<_, _> = placeholders
                .iter()
                .map(|name| {
                    let value = captures.name(name).with_context(|| {
                        format!("path '{}' is missing {{{name}}}", pattern.original)
                    })?;
                    Ok((name.as_str(), value.as_str()))
                })
                .collect::<Result<_>>()?;
            let key: Vec<_> = placeholders
                .iter()
                .map(|name| captured[name.as_str()].to_string())
                .collect();
            let label = render_template(&extraction.rowlabel.value, &captured, &row_label_pattern)?;
            let row = rows.entry(key).or_insert_with(|| ExtractedRow {
                label,
                values: vec![None; patterns.len()],
            });

            if let Some(value) = object.attributes.get(&pattern.member)
                && row.values[column].replace(value.clone()).is_some()
            {
                bail!(
                    "multiple values found for row '{}' and path '{}'",
                    row.label,
                    pattern.original
                );
            }
        }
    }

    let mut labels = HashSet::new();
    for row in rows.values() {
        if !labels.insert(&row.label) {
            bail!("duplicate rendered row label '{}'", row.label);
        }
    }

    let target_source = config
        .sources
        .iter()
        .flatten()
        .find(|source| source.id == extraction.id)
        .expect("Config::new validates extraction source");
    let delimiter = target_source.delimiter.unwrap_or(';');
    if !delimiter.is_ascii() {
        bail!("CSV delimiter must be an ASCII character");
    }
    let mut writer = WriterBuilder::new()
        .delimiter(delimiter as u8)
        .from_writer(Vec::new());
    let headers = std::iter::once(extraction.rowlabel.header.as_str())
        .chain(extraction.values.iter().map(|value| value.header.as_str()));
    writer.write_record(headers)?;

    for row in rows.values() {
        let record = std::iter::once(row.label.as_str()).chain(
            row.values
                .iter()
                .map(|value| value.as_deref().unwrap_or("")),
        );
        writer.write_record(record)?;
    }
    writer.flush()?;
    let missing_values = extraction
        .values
        .iter()
        .enumerate()
        .filter_map(|(column, value)| {
            let labels: Vec<_> = rows
                .values()
                .filter(|row| row.values[column].is_none())
                .map(|row| format!("'{}'", row.label))
                .collect();
            (!labels.is_empty()).then(|| {
                format!(
                    "Value '{}' not found for {}",
                    value.header,
                    labels.join(", ")
                )
            })
        })
        .collect();
    Ok(ExtractionResult {
        output: writer.into_inner()?,
        row_labels: rows.values().map(|row| row.label.clone()).collect(),
        missing_values,
    })
}

fn read_row_labels(path: &Path, delimiter: char) -> Result<Vec<String>> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .trim(Trim::All)
        .from_path(path)
        .with_context(|| format!("Problem reading existing CSV '{}'", path.display()))?;
    let mut labels = Vec::new();
    for record in reader.records() {
        let record =
            record.with_context(|| format!("Problem reading existing CSV '{}'", path.display()))?;
        labels.push(record.get(0).unwrap_or_default().to_string());
    }
    Ok(labels)
}

fn compare_row_labels(existing: &[String], extracted: &[String]) -> Vec<String> {
    let existing_set: HashSet<_> = existing.iter().collect();
    let extracted_set: HashSet<_> = extracted.iter().collect();
    let added: Vec<_> = extracted
        .iter()
        .filter(|label| !existing_set.contains(label))
        .cloned()
        .collect();
    let removed: Vec<_> = existing
        .iter()
        .filter(|label| !extracted_set.contains(label))
        .cloned()
        .collect();
    let mut messages = Vec::new();
    if !added.is_empty() {
        messages.push(format!("Rows added: {}", added.join(", ")));
    }
    if !removed.is_empty() {
        messages.push(format!("Rows removed: {}", removed.join(", ")));
    }
    messages
}

fn compile_path(path: &str) -> Result<PathPattern> {
    let (object, member) = path.rsplit_once('.').unwrap_or((path, "Meas"));
    if object.is_empty() || member.is_empty() {
        bail!("invalid extraction path '{path}'");
    }
    let (object_type, object_name) = object
        .split_once(':')
        .map_or((None, object), |(object_type, object_name)| {
            (Some(object_type.to_string()), object_name)
        });
    if object_type.as_deref().is_some_and(str::is_empty) || object_name.is_empty() {
        bail!("invalid extraction path '{path}'");
    }
    let (regex, placeholders) = compile_name_pattern(object_name, path)?;
    Ok(PathPattern {
        object_type,
        object_name: regex,
        member: member.to_string(),
        placeholders,
        original: path.to_string(),
    })
}

fn compile_name_pattern(pattern: &str, path: &str) -> Result<(Regex, Vec<String>)> {
    let placeholder = Regex::new(r"\{([A-Za-z_][A-Za-z0-9_]*)\}").unwrap();
    let mut regex = String::from("^");
    let mut placeholders = Vec::new();
    let mut end = 0;
    let without_placeholders = placeholder.replace_all(pattern, "");
    if without_placeholders.contains(['{', '}']) {
        bail!("invalid placeholder in path '{path}'");
    }
    for captures in placeholder.captures_iter(pattern) {
        let matched = captures.get(0).unwrap();
        regex.push_str(&regex::escape(&pattern[end..matched.start()]));
        let name = captures.get(1).unwrap().as_str();
        if placeholders.iter().any(|existing| existing == name) {
            bail!("duplicate placeholder '{{{name}}}' in path '{path}'");
        }
        placeholders.push(name.to_string());
        regex.push_str(&format!(r"(?P<{name}>\S+?)"));
        end = matched.end();
    }
    regex.push_str(&regex::escape(&pattern[end..]));
    regex.push('$');
    Ok((Regex::new(&regex)?, placeholders))
}

fn validate_row_label(
    template: &str,
    placeholders: &HashSet<&String>,
    placeholder_pattern: &Regex,
) -> Result<()> {
    for captures in placeholder_pattern.captures_iter(template) {
        let name = captures.get(1).unwrap().as_str();
        if !placeholders
            .iter()
            .any(|placeholder| placeholder.as_str() == name)
        {
            bail!("unknown placeholder '{{{name}}}' in row label");
        }
    }
    Ok(())
}

fn render_template(
    template: &str,
    values: &HashMap<&str, &str>,
    placeholder_pattern: &Regex,
) -> Result<String> {
    let mut error = None;
    let rendered = placeholder_pattern.replace_all(template, |captures: &regex::Captures<'_>| {
        let name = &captures[1];
        values.get(name).copied().unwrap_or_else(|| {
            error = Some(name.to_string());
            ""
        })
    });
    if let Some(name) = error {
        bail!("unknown placeholder '{{{name}}}' in row label");
    }
    Ok(rendered.into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExtractionRowLabel, ExtractionSource, ExtractionValue, Source};
    use crate::datasource::{CsvSourceReader, DataSourceReader};
    use tempfile::tempdir;

    fn config_and_extraction() -> (Config, ExtractionSource) {
        let config = Config {
            templatepath: String::new(),
            layout: Vec::new(),
            sources: Some(vec![Source {
                filename: Filename::Single("out.csv".to_string()),
                id: "extracted".to_string(),
                delimiter: Some(','),
                ..Default::default()
            }]),
            ..Default::default()
        };
        let extraction = ExtractionSource {
            id: "extracted".to_string(),
            rowlabel: ExtractionRowLabel {
                header: "Wellname".to_string(),
                value: "Well{well}".to_string(),
            },
            values: vec![
                ExtractionValue {
                    path: "Cvr:D{well}Qg.Meas".to_string(),
                    header: "QgMeas".to_string(),
                },
                ExtractionValue {
                    path: "Mvr:D{well}Zpc.Meas".to_string(),
                    header: "ZpcMeas".to_string(),
                },
            ],
        };
        (config, extraction)
    }

    #[test]
    fn extracts_rows_by_named_placeholder() {
        let (config, extraction) = config_and_extraction();
        let objects = septic_cnfg::parse(
            "Cvr: D01Qg\nMeas= 230000\nMvr: D01Zpc\nMeas= 35\nCvr: D02Qg\nMeas= 240000",
        )
        .unwrap();

        let result = extract_to_csv(&extraction, &config, &objects).unwrap();

        assert_eq!(
            String::from_utf8(result.output).unwrap(),
            "Wellname,QgMeas,ZpcMeas\nWell01,230000,35\nWell02,240000,\n"
        );
        assert_eq!(
            result.missing_values,
            ["Value 'ZpcMeas' not found for 'Well02'"]
        );
        assert_eq!(result.row_labels, ["Well01", "Well02"]);
    }

    #[test]
    fn reports_added_and_removed_rows_in_source_order() {
        let existing = vec!["D11".to_string(), "D01".to_string(), "D12".to_string()];
        let extracted = vec!["D01".to_string(), "D02".to_string(), "D03".to_string()];

        assert_eq!(
            compare_row_labels(&existing, &extracted),
            ["Rows added: D02, D03", "Rows removed: D11, D12"]
        );
    }

    #[test]
    fn reads_existing_labels_with_configured_delimiter() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("existing.csv");
        fs::write(&path, "Well;Value\n D01 ;1\nD02;2\n").unwrap();

        assert_eq!(read_row_labels(&path, ';').unwrap(), ["D01", "D02"]);
    }

    #[test]
    fn unqualified_ambiguous_path_fails() {
        let (config, mut extraction) = config_and_extraction();
        extraction.values.truncate(1);
        extraction.values[0].path = "D{well}Qg.Meas".to_string();
        let objects = septic_cnfg::parse("SopcCvr: D01Qg\nMeas= 1\nCvr: D01Qg\nMeas= 2").unwrap();

        let error = extract_to_csv(&extraction, &config, &objects).unwrap_err();

        assert!(error.to_string().contains("multiple values"));
    }

    #[test]
    fn omitted_member_defaults_to_meas() {
        let pattern = compile_path("Evr:{well}CEstCvG").unwrap();

        assert_eq!(pattern.object_type.as_deref(), Some("Evr"));
        assert_eq!(pattern.member, "Meas");
        assert_eq!(
            &pattern.object_name.captures("D01CEstCvG").unwrap()["well"],
            "D01"
        );
    }

    #[test]
    fn command_line_source_overrides_configured_input() {
        let directory = tempdir().unwrap();
        let config_file = directory.path().join("extract.yaml");
        let override_file = directory.path().join("override.cnfg");
        fs::write(
            &config_file,
            r#"{
"templatepath": "templates",
"sources": [
    {"filename": "extracted.csv", "id": "extracted", "delimiter": ","},
    {"filename": "secondary.csv", "id": "secondary"}
],
"layout": [],
"extraction": {
        "from": "missing.cnfg",
        "sources": [
            {
                "id": "extracted",
                "rowlabel": {"header": "Wellname", "value": "Well{well}"},
                "values": [{"path": "Cvr:D{well}Qg.Meas", "header": "QgMeas"}]
            },
            {
                "id": "secondary",
                "rowlabel": {"header": "Wellname", "value": "Well{well}"},
                "values": [{"path": "Cvr:D{well}Qg.Meas", "header": "Measured"}]
            }
        ]
    }
}
"#,
        )
        .unwrap();
        fs::write(&override_file, "Cvr: D01Qg\nMeas= 115.5").unwrap();

        cmd_extract(&config_file, Some(&override_file)).unwrap();

        assert_eq!(
            fs::read_to_string(directory.path().join("extracted.csv")).unwrap(),
            "Wellname,QgMeas\nWell01,115.5\n"
        );
        let rows = CsvSourceReader::new("extracted.csv", directory.path(), Some(','))
            .read()
            .unwrap();
        assert!(rows.contains_key("Well01"));
        assert_eq!(
            fs::read_to_string(directory.path().join("secondary.csv")).unwrap(),
            "Wellname;Measured\nWell01;115.5\n"
        );
    }

    #[test]
    fn loads_objects_once_for_an_extraction() {
        let directory = tempdir().unwrap();
        let source_file = directory.path().join("source.cnfg");
        fs::write(&source_file, "Cvr: D01Qg\nMeas= 1").unwrap();

        assert_eq!(
            load_objects(&source_file, encoding_rs::UTF_8, "utf-8")
                .unwrap()
                .len(),
            1
        );
    }
}
