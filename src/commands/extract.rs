use crate::config::{Config, ExtractionSource, Filename};
use crate::septic_cnfg;
use anyhow::{Context, Result, bail};
use clap::Parser;
use csv::{ReaderBuilder, StringRecord, Trim, WriterBuilder};
use minijinja::Environment;
use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct Extract {
    /// The yaml config file
    pub config_file: PathBuf,
    /// Septic config file to extract from (overrides extract.from)
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

struct PreparedExtraction {
    target: PathBuf,
    output: Vec<u8>,
    messages: Vec<String>,
}

#[derive(Debug)]
struct ExtractionResult {
    output: Vec<u8>,
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
    let extraction = config.extract.as_ref().context("missing field 'extract'")?;
    let encoding = encoding_rs::Encoding::for_label(config.encoding.as_bytes())
        .expect("Config::new validates encoding");

    let source_file = match source_override {
        Some(path) => path.to_path_buf(),
        None => root.join(
            extraction
                .from
                .as_deref()
                .context("missing field 'extract.from' and no source file was provided")?,
        ),
    };
    let objects = load_objects(&source_file, encoding, &config.encoding)?;
    let mut prepared = Vec::new();
    for extraction_source in &extraction.to {
        let target_source = config
            .sources
            .iter()
            .flatten()
            .find(|source| source.id == extraction_source.id)
            .expect("Config::new validates extraction source");
        let target = match &target_source.filename {
            Filename::Single(target) => target,
            Filename::Multiple(_) => extraction_source
                .filename
                .as_ref()
                .expect("Config::new validates extraction filename"),
        };
        let target = root.join(target);
        let result = extract_to_csv(extraction_source, &config, &target, &objects)?;
        prepared.push(PreparedExtraction {
            target,
            output: result.output,
            messages: result.missing_values,
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
    target: &Path,
    objects: &[septic_cnfg::Object],
) -> Result<ExtractionResult> {
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
    let (headers, mut records) = read_existing_csv(target, delimiter)?;
    if headers.is_empty() {
        bail!(
            "CSV file '{}' must contain at least one column",
            target.display()
        );
    }
    let mut headers: Vec<_> = headers.iter().map(ToString::to_string).collect();
    let mut header_indexes: HashMap<_, _> = headers
        .iter()
        .enumerate()
        .map(|(index, header)| (header.clone(), index))
        .collect();
    for object in &extraction.objects {
        for header in &object.headers {
            if !header_indexes.contains_key(header) {
                header_indexes.insert(header.clone(), headers.len());
                headers.push(header.clone());
            }
        }
    }

    let env = Environment::new();
    let mut missing_values = Vec::new();
    for record in &mut records {
        record.resize(headers.len(), String::new());
        let context = headers
            .iter()
            .enumerate()
            .map(|(index, header)| {
                (
                    header.clone(),
                    record.get(index).cloned().unwrap_or_default(),
                )
            })
            .collect::<HashMap<_, _>>();
        let row_label = record.first().cloned().unwrap_or_default();
        if row_label.trim().is_empty() {
            bail!(
                "first column in '{}' must not contain empty row labels",
                target.display()
            );
        }

        for object in &extraction.objects {
            let object_name = env.template_from_str(&object.name)?.render(&context)?;
            if let Some(props) = &object.props {
                for (member, header) in props.iter().zip(&object.headers) {
                    let extracted =
                        find_attribute(objects, object.r#type.as_deref(), &object_name, member)?;
                    let index = header_indexes[header];
                    if let Some(extracted) = extracted {
                        record[index] = extracted;
                    } else {
                        record[index].clear();
                        missing_values
                            .push(format!("Value '{}' not found for '{}'", header, row_label));
                    }
                }
            } else if let Some(regexps) = &object.regexps {
                for (regex_pattern, header) in regexps.iter().zip(&object.headers) {
                    let regex = Regex::new(regex_pattern)?;
                    let extracted = find_freetext(objects, &object_name, &regex)?;
                    let index = header_indexes[header];
                    if let Some(extracted) = extracted {
                        record[index] = extracted;
                    } else {
                        record[index].clear();
                        missing_values
                            .push(format!("Value '{}' not found for '{}'", header, row_label));
                    }
                }
            }
        }
    }

    let mut writer = WriterBuilder::new()
        .delimiter(delimiter as u8)
        .from_writer(Vec::new());
    writer.write_record(headers)?;
    for record in records {
        writer.write_record(record)?;
    }
    writer.flush()?;
    Ok(ExtractionResult {
        output: writer.into_inner()?,
        missing_values,
    })
}

fn read_existing_csv(path: &Path, delimiter: char) -> Result<(StringRecord, Vec<Vec<String>>)> {
    let mut reader = ReaderBuilder::new()
        .delimiter(delimiter as u8)
        .trim(Trim::All)
        .from_path(path)
        .with_context(|| format!("Problem reading existing CSV '{}'", path.display()))?;
    let headers = reader
        .headers()
        .with_context(|| format!("Problem reading existing CSV '{}'", path.display()))?
        .clone();
    let mut records = Vec::new();
    for record in reader.records() {
        let record =
            record.with_context(|| format!("Problem reading existing CSV '{}'", path.display()))?;
        records.push(record.iter().map(ToString::to_string).collect());
    }
    Ok((headers, records))
}

fn find_attribute(
    objects: &[septic_cnfg::Object],
    object_type: Option<&str>,
    object_name: &str,
    member: &str,
) -> Result<Option<String>> {
    let values: Vec<_> = objects
        .iter()
        .filter(|object| {
            object_type.is_none_or(|object_type| object.object_type == object_type)
                && object.name == object_name
        })
        .flat_map(|object| {
            object
                .attributes
                .iter()
                .filter_map(move |(attribute, value)| {
                    member_matches(member, attribute).then_some(value)
                })
        })
        .collect();
    match values.as_slice() {
        [] => Ok(None),
        [value] => Ok(Some((*value).clone())),
        _ => bail!("multiple values found for object '{object_name}' and member '{member}'"),
    }
}

fn find_freetext(
    objects: &[septic_cnfg::Object],
    object_name: &str,
    regex: &Regex,
) -> Result<Option<String>> {
    let mut matches = Vec::new();
    for object in objects.iter().filter(|object| object.name == object_name) {
        for (member, value) in &object.attributes {
            let text = format!("{member}={value}");
            for captures in regex.captures_iter(&text) {
                let capture = captures
                    .get(1)
                    .context("extract freetext regex capture group did not match")?;
                matches.push(capture.as_str().to_string());
            }
        }
    }
    match matches.as_slice() {
        [] => Ok(None),
        [value] => Ok(Some(value.clone())),
        _ => bail!("multiple freetext matches found for object '{object_name}'"),
    }
}

fn member_matches(requested: &str, actual: &str) -> bool {
    if matches!(requested, "High" | "Low" | "SetPnt" | "Iv") {
        return actual == format!("{requested}On") || actual == format!("{requested}Off");
    }
    requested == actual
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{ExtractionObject, ExtractionSource, Source};
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
            filename: None,
            objects: vec![
                ExtractionObject {
                    r#type: Some("Cvr".to_string()),
                    name: "{{ WellName }}Qg".to_string(),
                    props: Some(vec!["Meas".to_string()]),
                    regexps: None,
                    headers: vec!["QgMeas".to_string()],
                },
                ExtractionObject {
                    r#type: Some("Mvr".to_string()),
                    name: "{{ WellName }}Zpc".to_string(),
                    props: Some(vec!["Meas".to_string()]),
                    regexps: None,
                    headers: vec!["ZpcMeas".to_string()],
                },
            ],
        };
        (config, extraction)
    }

    #[test]
    fn extracts_values_for_existing_csv_rows() {
        let (config, extraction) = config_and_extraction();
        let directory = tempdir().unwrap();
        let target = directory.path().join("out.csv");
        fs::write(&target, "WellName,QgMeas,ZpcMeas\nD01,,\nD02,,\n").unwrap();
        let objects = septic_cnfg::parse(
            "Cvr: D01Qg\nMeas= 230000\nMvr: D01Zpc\nMeas= 35\nCvr: D02Qg\nMeas= 240000",
        )
        .unwrap();

        let result = extract_to_csv(&extraction, &config, &target, &objects).unwrap();

        assert_eq!(
            String::from_utf8(result.output).unwrap(),
            "WellName,QgMeas,ZpcMeas\nD01,230000,35\nD02,240000,\n"
        );
        assert_eq!(
            result.missing_values,
            ["Value 'ZpcMeas' not found for 'D02'"]
        );
    }

    #[test]
    fn appends_configured_headers_that_are_missing_from_csv() {
        let (config, mut extraction) = config_and_extraction();
        extraction.objects.truncate(1);
        let directory = tempdir().unwrap();
        let target = directory.path().join("out.csv");
        fs::write(&target, "WellName\nD01\n").unwrap();
        let objects = septic_cnfg::parse("Cvr: D01Qg\nMeas= 230000").unwrap();

        let result = extract_to_csv(&extraction, &config, &target, &objects).unwrap();

        assert_eq!(
            String::from_utf8(result.output).unwrap(),
            "WellName,QgMeas\nD01,230000\n"
        );
    }

    #[test]
    fn extracts_multiple_props_from_one_rendered_object_name() {
        let (config, mut extraction) = config_and_extraction();
        extraction.objects.truncate(1);
        extraction.objects[0].name = "{{ WellName }}Rate".to_string();
        extraction.objects[0].props = Some(vec!["Low".to_string(), "SetPnt".to_string()]);
        extraction.objects[0].headers = vec!["QgLoLim".to_string(), "QgSP".to_string()];
        let directory = tempdir().unwrap();
        let target = directory.path().join("out.csv");
        fs::write(&target, "WellName,QgLoLim,QgSP\nW11,,\nW12,,\n").unwrap();
        let objects = septic_cnfg::parse(
            "Cvr: W11Rate\nLowOff= 2.0\nSetPntOn= 3.4\nCvr: W12Rate\nLowOff= 2.1\nSetPntOn= 3.5",
        )
        .unwrap();

        let result = extract_to_csv(&extraction, &config, &target, &objects).unwrap();

        assert_eq!(
            String::from_utf8(result.output).unwrap(),
            "WellName,QgLoLim,QgSP\nW11,2.0,3.4\nW12,2.1,3.5\n"
        );
    }

    #[test]
    fn extracts_freetext_capture_from_member_value_pair() {
        let (config, mut extraction) = config_and_extraction();
        extraction.objects = vec![ExtractionObject {
            r#type: None,
            name: "{{ WellName }}Rate".to_string(),
            props: None,
            regexps: Some(vec![
                "Low(On|Off)".to_string(),
                "SetPnt(On|Off)".to_string(),
            ]),
            headers: vec!["RateLoLimActive".to_string(), "RateSpActive".to_string()],
        }];
        let directory = tempdir().unwrap();
        let target = directory.path().join("out.csv");
        fs::write(&target, "WellName\nW11\nW12\n").unwrap();
        let objects = septic_cnfg::parse(
            "Cvr: W11Rate\nLowOn= 2.0\nSetPntOff= 3.4\nCvr: W12Rate\nLowOff= 2.1\nSetPntOn= 3.5",
        )
        .unwrap();

        let result = extract_to_csv(&extraction, &config, &target, &objects).unwrap();

        assert_eq!(
            String::from_utf8(result.output).unwrap(),
            "WellName,RateLoLimActive,RateSpActive\nW11,On,Off\nW12,Off,On\n"
        );
    }

    #[test]
    fn freetext_fails_on_multiple_matches() {
        let objects = septic_cnfg::parse("Cvr: W11Rate\nLowOn= 2.0\nLowOff= 2.1").unwrap();
        let regex = Regex::new("Low(On|Off)").unwrap();

        let error = find_freetext(&objects, "W11Rate", &regex).unwrap_err();

        assert!(error.to_string().contains("multiple freetext matches"));
    }

    #[test]
    fn reads_existing_csv_with_configured_delimiter() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("existing.csv");
        fs::write(&path, "Well;Value\n D01 ;1\nD02;2\n").unwrap();

        let (headers, rows) = read_existing_csv(&path, ';').unwrap();
        assert_eq!(headers.get(0), Some("Well"));
        assert_eq!(rows[0][0], "D01");
    }

    #[test]
    fn duplicate_object_member_fails() {
        let (config, mut extraction) = config_and_extraction();
        extraction.objects.truncate(1);
        extraction.objects[0].r#type = None;
        let directory = tempdir().unwrap();
        let target = directory.path().join("out.csv");
        fs::write(&target, "WellName,QgMeas\nD01,\n").unwrap();
        let objects = septic_cnfg::parse("SopcCvr: D01Qg\nMeas= 1\nCvr: D01Qg\nMeas= 2").unwrap();

        let error = extract_to_csv(&extraction, &config, &target, &objects).unwrap_err();

        assert!(error.to_string().contains("multiple values"));
    }

    #[test]
    fn exact_type_limits_object_matches() {
        let objects = septic_cnfg::parse("SopcCvr: D01Qg\nMeas= 1\nCvr: D01Qg\nMeas= 2").unwrap();

        assert_eq!(
            find_attribute(&objects, Some("Cvr"), "D01Qg", "Meas").unwrap(),
            Some("2".to_string())
        );
    }

    #[test]
    fn selected_member_basenames_match_on_off_suffixes() {
        let objects = septic_cnfg::parse(
            "Cvr: W11Rate\nHighOn= 7.0\nLow= 1.0\nLowOff= 2.0\nLowPenalty= 99.0\nIvOn= 1",
        )
        .unwrap();

        assert_eq!(
            find_attribute(&objects, Some("Cvr"), "W11Rate", "High").unwrap(),
            Some("7.0".to_string())
        );
        assert_eq!(
            find_attribute(&objects, Some("Cvr"), "W11Rate", "Low").unwrap(),
            Some("2.0".to_string())
        );
        assert_eq!(
            find_attribute(&objects, Some("Cvr"), "W11Rate", "Iv").unwrap(),
            Some("1".to_string())
        );
        assert_eq!(
            find_attribute(&objects, Some("Cvr"), "W11Rate", "LowPenalty").unwrap(),
            Some("99.0".to_string())
        );
    }

    #[test]
    fn selected_member_basenames_fail_if_both_suffixes_exist() {
        let objects = septic_cnfg::parse("Cvr: W11Rate\nSetPntOn= 3.4\nSetPntOff= 0.0").unwrap();

        let error = find_attribute(&objects, Some("Cvr"), "W11Rate", "SetPnt").unwrap_err();

        assert!(error.to_string().contains("multiple values"));
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
    {"filename": ["first.csv", "secondary.csv"], "id": "secondary"}
],
"layout": [],
"extract": {
        "from": "missing.cnfg",
        "to": [
            {
                "id": "extracted",
                "objects": [{"type": "Cvr", "name": "{{ WellName }}Qg", "props": ["Meas"], "headers": ["QgMeas"]}]
            },
            {
                "id": "secondary",
                "filename": "secondary.csv",
                "objects": [{"type": "Cvr", "name": "{{ WellName }}Qg", "props": ["Meas"], "headers": ["Measured"]}]
            }
        ]
    }
}
"#,
        )
        .unwrap();
        fs::write(
            directory.path().join("extracted.csv"),
            "WellName,QgMeas\nD01,\n",
        )
        .unwrap();
        fs::write(
            directory.path().join("secondary.csv"),
            "WellName;Measured\nD01;\n",
        )
        .unwrap();
        fs::write(&override_file, "Cvr: D01Qg\nMeas= 115.5").unwrap();

        cmd_extract(&config_file, Some(&override_file)).unwrap();

        assert_eq!(
            fs::read_to_string(directory.path().join("extracted.csv")).unwrap(),
            "WellName,QgMeas\nD01,115.5\n"
        );
        assert_eq!(
            fs::read_to_string(directory.path().join("secondary.csv")).unwrap(),
            "WellName;Measured\nD01;115.5\n"
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
