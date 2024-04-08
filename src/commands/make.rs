use crate::config::{Config, Source};
use crate::datasource::{CsvSourceReader, DataSourceReader, DataSourceRows, ExcelSourceReader};
use crate::renderer::MiniJinja;
use crate::{
    ask_should_overwrite, collect_file_list, create_patch, timestamps_newer_than, CtxDataType,
};
use anyhow::Context;
use clap::Parser;
use minijinja::Value;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum MakeError {
    TimeStampError(anyhow::Error),
    CfgFileReadError(anyhow::Error),
    CollectFileList(anyhow::Error),
    CreateOutputFile(anyhow::Error),
    LoadSourceError(anyhow::Error),
    MiniJinjaError(minijinja::Error),
    NoFilesChanged,
    NoChangeFromPrevious,
}

impl std::fmt::Display for MakeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MakeError::TimeStampError(e) => write!(f, "{e:#}"),
            MakeError::CfgFileReadError(e) => write!(f, "Problem reading config file: {e:#}"),
            MakeError::CollectFileList(e) => write!(f, "Problem identifying changed files: {e:#}"),
            MakeError::CreateOutputFile(e) => write!(f, "Problem creating output file: {e:#}"),
            MakeError::LoadSourceError(e) => write!(f, "{e:#}"),
            MakeError::MiniJinjaError(e) => write!(f, "{e:#}"),
            MakeError::NoFilesChanged => write!(f, "No files have changed, skipping rebuild."),
            MakeError::NoChangeFromPrevious => {
                write!(f, "No change from previous version, exiting.")
            }
        }
    }
}
impl std::error::Error for MakeError {}

impl From<MakeError> for i32 {
    fn from(err: MakeError) -> i32 {
        match err {
            MakeError::TimeStampError(_) => 2,
            MakeError::CfgFileReadError(_) => 2,
            MakeError::CollectFileList(_) => 2,
            MakeError::CreateOutputFile(_) => 2,
            MakeError::LoadSourceError(_) => 2,
            MakeError::MiniJinjaError(_) => 2,
            MakeError::NoFilesChanged => 1,
            MakeError::NoChangeFromPrevious => 1,
        }
    }
}

#[derive(Parser, Debug)]
pub struct Make {
    /// The yaml config file
    pub config_file: PathBuf,
    // /// Name of output file (overrides config option "outputfile")
    // #[arg(short, long, value_name = "FILE")]
    // pub output: Option<PathBuf>,
    // /// Only output warnings or errors
    // #[arg(short, long)]
    // pub silent: bool,
    /// Global variable to use for all templates, also those without specified source. Can be repeated. Global variables overwrite other variables with same name
    #[arg(short, long, value_names = ["name", "value"])]
    pub var: Option<Vec<String>>,
    /// Only make if layout or source files have changed since last make
    #[arg(long)]
    pub ifchanged: bool,
}

impl Make {
    pub fn execute(&self) {
        let result = cmd_make(
            &self.config_file,
            self.ifchanged,
            &self.var.clone().unwrap_or_default(),
        );

        match result {
            Ok(_) => (),
            Err(err) => {
                eprintln!("{:#}", err);
                std::process::exit(err.into())
            }
        }
    }
}

fn cmd_make(
    cfg_file: &Path,
    only_if_changed: bool,
    globals: &[String],
) -> anyhow::Result<(), MakeError> {
    let mut cfg_file = cfg_file.to_path_buf();
    cfg_file
        .extension()
        .is_none()
        .then(|| cfg_file.set_extension("yaml"));

    let relative_root = PathBuf::from(
        cfg_file
            .parent()
            .expect("BUG: Unable to obtain parent of cfg_file"),
    );

    let cfg = Config::new(&cfg_file).map_err(MakeError::CfgFileReadError)?;

    if only_if_changed & cfg.outputfile.is_some() {
        let outfile = relative_root.join(
            cfg.outputfile
                .as_ref()
                .expect("BUG: Unable to unwrap cfg.outputfile"),
        );
        if outfile.exists() {
            let file_list = collect_file_list(&cfg, &cfg_file, &relative_root)
                .map_err(MakeError::CollectFileList)?;
            if !timestamps_newer_than(&file_list, &outfile).map_err(MakeError::TimeStampError)? {
                return Err(MakeError::NoFilesChanged);
            }
        }
    }

    let all_source_data: HashMap<String, DataSourceRows> =
        load_all_source_data(&cfg, &relative_root).map_err(MakeError::LoadSourceError)?;

    let template_path = relative_root.join(&cfg.templatepath);
    let mut renderer =
        MiniJinja::new(globals, &template_path, cfg.counters).map_err(MakeError::MiniJinjaError)?;

    for (key, source_data) in all_source_data.iter() {
        let values_vec: Vec<HashMap<String, CtxDataType>> = source_data.values().cloned().collect();
        renderer
            .env
            .add_global(key, Value::from_serializable(&values_vec));
    }

    let mut rendered = String::new();

    for template in &cfg.layout {
        rendered +=
            &MiniJinja::render_template(&renderer, template, &all_source_data, cfg.adjustspacing)
                .map_err(MakeError::MiniJinjaError)?
    }
    if cfg.adjustspacing {
        rendered = rendered.trim_end().to_string();
        rendered.push('\n');
    }

    if let Some(path) = cfg.outputfile.as_ref().map(|f| relative_root.join(f)) {
        let mut do_write_file = true;

        if path.exists() {
            let mut reader = encoding_rs_io::DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding_rs::WINDOWS_1252))
                .build(
                    fs::File::open(&path)
                        .expect("BUG: Unable to open existing outputfile for diff"),
                );
            let mut old_file_content = String::new();
            reader
                .read_to_string(&mut old_file_content)
                .expect("BUG: Unable to read existing outputfile for diff");

            let diff = create_patch(&old_file_content, &rendered);

            if diff.hunks().is_empty() {
                return Err(MakeError::NoChangeFromPrevious);
            } else if cfg.verifycontent {
                do_write_file =
                    ask_should_overwrite(&diff).expect("error: unable to read user input");
            }
        }
        if do_write_file {
            if path.exists() {
                let backup_path = match path.extension() {
                    Some(ext) => {
                        path.with_extension(format!("{}.bak", ext.to_str().unwrap_or_default()))
                    }
                    None => path.with_extension("bak"),
                };
                fs::rename(&path, backup_path).expect("BUG: Failed to create backup file");
            }

            let mut f = fs::File::create(&path)
                .with_context(|| format!("Problem creating output file '{}'", &path.display()))
                .map_err(MakeError::CreateOutputFile)?;

            let (cow, _encoding, _b) = encoding_rs::WINDOWS_1252.encode(&rendered);
            f.write_all(&cow)
                .with_context(|| format!("Problem writing output file '{}'", &path.display()))
                .map_err(MakeError::CreateOutputFile)?
        }
    } else {
        // TODO: || with input argument for writing to stdout
        println!("{rendered}");
    }
    Ok(())
}

fn load_all_source_data(
    cfg: &Config,
    relative_root: &Path,
) -> anyhow::Result<HashMap<String, DataSourceRows>> {
    cfg.sources
        .iter()
        .map(|source| {
            let source_data = load_source_data(source, relative_root)?;
            Ok((source.id.clone(), source_data))
        })
        .collect()
}

fn load_source_data(source: &Source, relative_root: &Path) -> anyhow::Result<DataSourceRows> {
    let reader: Box<dyn DataSourceReader> = match Path::new(&source.filename).extension() {
        Some(ext) if ext == "xlsx" => Box::new(ExcelSourceReader::new(
            &source.filename,
            relative_root,
            source.sheet.as_deref(),
        )),
        Some(ext) if ext == "csv" => {
            let delimiter = source.delimiter.unwrap_or(';');

            Box::new(CsvSourceReader::new(
                &source.filename,
                relative_root,
                Some(delimiter),
            ))
        }
        _ => anyhow::bail!(
            "Unsupported file extension for source file '{}'",
            source.filename
        ),
    };
    reader
        .read()
        .with_context(|| format!("Problem reading source file '{}'", source.filename))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config;
    use crate::datasource::{DataSourceReader, DataSourceRows, ExcelSourceReader};

    fn get_all_source_data() -> anyhow::Result<HashMap<String, DataSourceRows>> {
        let mut all_source_data: HashMap<String, DataSourceRows> = HashMap::new();
        let source_main = config::Source {
            filename: "test.xlsx".to_string(),
            id: "main".to_string(),
            sheet: Some("Normals".to_string()),
            ..Default::default()
        };
        let source_errors = config::Source {
            filename: "test.xlsx".to_string(),
            id: "errors".to_string(),
            sheet: Some("Specials".to_string()),
            ..Default::default()
        };
        for source in [source_main, source_errors] {
            let reader = ExcelSourceReader::new(
                &source.filename,
                Path::new("tests/testdata/"),
                source.sheet.as_deref(),
            );

            let source_data = reader.read()?;
            all_source_data.insert(source.id.to_string(), source_data);
        }
        Ok(all_source_data)
    }

    #[test]
    fn render_with_normal_values() -> anyhow::Result<()> {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "01_normals.tmpl".to_string(),
            source: Some("main".to_string()),
            ..Default::default()
        };
        let all_source_data = get_all_source_data()?;
        let result = renderer
            .render_template(&template, &all_source_data, true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "String: one\nString: one\nBool: true\nInteger: 1\nWhole float: 1\nFloat: 1.234\n\nString: two\nString: two\nBool: false\nInteger: 2\nWhole float: 2\nFloat: 2.3456\n\nString: three\nString: three\nBool: true\nInteger: 3\nWhole float: 3\nFloat: 34.56"
        );
        Ok(())
    }

    #[test]
    fn render_with_special_values() -> anyhow::Result<()> {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "02_specials.tmpl".to_string(),
            source: Some("errors".to_string()),
            ..Default::default()
        };
        let all_source_data = get_all_source_data()?;
        let result = renderer
            .render_template(&template, &all_source_data, true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "Empty: >|none|<\nError: >|#DIV/0!|<\n\nEmpty: >||<\nError: >|#N/A|<\n\nEmpty: >|\"\"|<\nError: >|#NAME?|<\n\nEmpty: >|\"\"|<\nError: >|#NULL!|<\n\nEmpty: >|none|<\nError: >|#NUM!|<\n\nEmpty: >||<\nError: >|#REF!|<\n\nEmpty: >|\"\"|<\nError: >|#VALUE!|<"
        );
        Ok(())
    }

    #[test]
    fn render_with_global_variables() -> anyhow::Result<()> {
        let globals = ["glob".to_string(), "globvalue".to_string()];
        let renderer =
            MiniJinja::new(&globals, Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "03_globals.tmpl".to_string(),
            ..Default::default()
        };
        let all_source_data = get_all_source_data()?;

        let result = renderer
            .render_template(&template, &all_source_data, true)
            .unwrap();
        assert_eq!(result.trim_end(), "Global: >|globvalue|<");
        Ok(())
    }

    #[test]
    // FIXME: Horrible test with too much code duplication from cmd_main()
    fn render_with_global_source_no_iteration() -> anyhow::Result<()> {
        let mut renderer =
            MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "08_sources.tmpl".to_string(),
            ..Default::default()
        };
        let all_source_data = get_all_source_data()?;
        for (key, source_data) in all_source_data.iter() {
            let values_vec: Vec<_> = source_data.values().cloned().collect();
            renderer
                .env
                .add_global(key, Value::from_serializable(&values_vec));
        }
        let result = MiniJinja::render_template(&renderer, &template, &HashMap::new(), true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "Num rows: 3\nRows in order: onetwothree\nSingle value: 34.56"
        );
        Ok(())
    }

    #[test]
    // FIXME: Horrible test with too much code duplication from cmd_main()
    fn render_with_global_source_and_iteration() -> anyhow::Result<()> {
        let mut renderer =
            MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "08_sources.tmpl".to_string(),
            source: Some("main".to_string()),
            include: Some(vec!["one".to_string()]),
            ..Default::default()
        };
        let all_source_data = get_all_source_data()?;
        for (key, source_data) in all_source_data.iter() {
            let values_vec: Vec<_> = source_data.values().cloned().collect();
            renderer
                .env
                .add_global(key, Value::from_serializable(&values_vec));
        }
        let result = MiniJinja::render_template(&renderer, &template, &all_source_data, true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "Num rows: 3\nRows in order: onetwothree\nSingle value: 34.56"
        );
        Ok(())
    }

    #[test]
    fn render_uses_latin1_encoding() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "06_encoding.tmpl".to_string(),
            ..Default::default()
        };
        let result = MiniJinja::render_template(&renderer, &template, &HashMap::new(), true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(result.trim_end(), "ae: æ\noe: ø\naa: å\ns^2: s²\nm^3: m³");
    }

    #[test]
    fn render_adjusts_spacing() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"), None).unwrap();
        let template = config::Template {
            name: "00_plaintext.tmpl".to_string(),
            ..Default::default()
        };
        let result_false =
            MiniJinja::render_template(&renderer, &template, &HashMap::new(), false).unwrap();
        let result_true =
            MiniJinja::render_template(&renderer, &template, &HashMap::new(), true).unwrap();
        assert_eq!(&result_true[result_true.len() - 5..], ".\r\n\r\n");
        #[cfg(target_os = "windows")]
        assert_eq!(&result_false[result_false.len() - 4..], "c.\r\n");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(&result_false[result_false.len() - 3..], "c.\n");
    }
}
