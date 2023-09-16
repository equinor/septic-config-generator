use crate::config::Config;
use crate::renderer::MiniJinja;

use datasource::{CsvSourceReader, DataSourceReader, DataSourceRows, ExcelSourceReader};
use diffy::{create_patch, PatchFormatter};
use glob::glob;

use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;

use std::path::{Path, PathBuf};
use std::process;

pub mod args;
pub mod commands;
pub mod config;
pub mod datasource;
pub mod renderer;

#[derive(Debug, PartialEq, Eq)]
pub enum CtxErrorType {
    /// Division by 0 error
    Div0,
    /// Unavailable value error
    NA,
    /// Invalid name error
    Name,
    /// Null value error
    Null,
    /// Number error
    Num,
    /// Invalid cell reference error
    Ref,
    /// Value error
    Value,
    /// Getting data
    GettingData,
}

#[derive(Debug, PartialEq)]
pub enum CtxDataType {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(f64),
    Error(CtxErrorType),
    Empty,
}

impl Serialize for CtxDataType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Int(value) => serializer.serialize_i64(*value),
            Self::Float(value) | Self::DateTime(value) => serializer.serialize_f64(*value),
            Self::String(value) => serializer.serialize_str(value),
            Self::Bool(value) => serializer.serialize_bool(*value),
            Self::Error(value) => {
                let s = match value {
                    CtxErrorType::Div0 => "#DIV/0!",
                    CtxErrorType::NA => "#N/A",
                    CtxErrorType::Name => "#NAME?",
                    CtxErrorType::Null => "#NULL!",
                    CtxErrorType::Num => "#NUM!",
                    CtxErrorType::Ref => "#REF!",
                    CtxErrorType::Value => "#VALUE!",
                    CtxErrorType::GettingData => "#UNKNOWN!",
                };
                serializer.serialize_str(s)
            }

            // DataTypeSer::Error(_) => serializer.serialize_str("Error in cell"), // Do I need to handle this as Err or just return a special value?
            Self::Empty => serializer.serialize_unit(),
        }
    }
}

fn _merge_maps(
    map1: &HashMap<String, String>,
    map2: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = map1.clone();
    merged.extend(map2.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    merged
}

fn bubble_error(pretext: &str, err: Box<dyn Error>) {
    eprintln!("{pretext}: {err:#}");
    let mut err = err.as_ref();
    while let Some(next_err) = err.source() {
        eprintln!();
        eprintln!("Above error caused by: {next_err:#}");
        err = next_err;
    }
}

fn set_extension_if_missing(filename: &Path, extension: &str) -> PathBuf {
    let mut file = filename.to_path_buf();
    if file.extension().is_none() {
        file.set_extension(extension);
    }

    file
}

fn render_template(
    renderer: &MiniJinja,
    template: &config::Template,
    source_data: &HashMap<String, DataSourceRows>,
    adjust_spacing: bool,
) -> Result<String, Box<dyn Error>> {
    let mut rendered = String::new();

    if let Some(src_name) = &template.source {
        let keys: Vec<String> = source_data[src_name]
            .iter()
            .map(|(key, _row)| key.clone())
            .collect();

        let mut items_set: HashSet<String> = keys.iter().cloned().collect();

        if template.include.is_some() {
            items_set = items_set
                .intersection(&template.include_set())
                .cloned()
                .collect();
        }

        items_set = items_set
            .difference(&template.exclude_set())
            .cloned()
            .collect();

        for (key, row) in &source_data[src_name] {
            if items_set.contains(key) {
                let mut tmpl_rend = renderer.render(&template.name, Some(row))?;

                if adjust_spacing {
                    tmpl_rend = tmpl_rend.trim_end().to_string();
                    tmpl_rend.push_str("\r\n\r\n");
                }
                rendered.push_str(&tmpl_rend);
            }
        }
    } else {
        rendered = renderer.render(&template.name, ())?;
    }

    if adjust_spacing {
        rendered = rendered.trim_end().to_string();
        rendered.push_str("\r\n\r\n");
    }

    Ok(rendered)
}

fn ask_should_overwrite(diff: &diffy::Patch<str>) -> Result<bool, Box<dyn Error>> {
    let f = PatchFormatter::new().with_color();
    print!("{}\n\nReplace original? [Y]es or [N]o: ", f.fmt_patch(diff));
    io::stdout().flush()?;
    let mut response = String::new();
    io::stdin().read_line(&mut response)?;
    Ok(response.trim().eq_ignore_ascii_case("y"))
}

fn collect_file_list(
    config: &Config,
    cfg_file: &Path,
    relative_root: &Path,
) -> Result<HashSet<PathBuf>, Box<dyn Error>> {
    let mut files = HashSet::new();

    // The yaml file
    files.insert(cfg_file.to_path_buf());

    // All files in templatedir
    let template_root = relative_root.join(Path::new(&config.templatepath));
    for entry in glob(&format!("{}/**/*", template_root.display()))? {
        let path = entry?;
        if path.is_file() {
            files.insert(path);
        }
    }

    // All sources
    for source in &config.sources {
        let source_path = relative_root.join(Path::new(&source.filename));
        files.insert(source_path.clone());
    }
    Ok(files)
}

fn timestamps_newer_than(
    files: &HashSet<PathBuf>,
    outfile: &PathBuf,
) -> Result<bool, Box<dyn Error>> {
    let checktime = fs::metadata(outfile)
        .map_err(|e| format!("{e} {outfile:?}"))?
        .modified()?;
    for f in files {
        let systime = fs::metadata(f)
            .map_err(|e| format!("{e} {f:?}"))?
            .modified()?;
        if systime > checktime {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn cmd_make(cfg_file: &Path, only_if_changed: bool, globals: &[String]) {
    let cfg_file = set_extension_if_missing(cfg_file, "yaml");
    let relative_root = PathBuf::from(cfg_file.parent().unwrap());
    let cfg = Config::new(&cfg_file).unwrap_or_else(|e| {
        eprintln!("Problem reading config file '{}: {e}", cfg_file.display());
        process::exit(2);
    });

    if only_if_changed & cfg.outputfile.is_some() {
        let outfile = relative_root.join(cfg.outputfile.as_ref().unwrap());
        if outfile.exists() {
            let file_list =
                collect_file_list(&cfg, &cfg_file, &relative_root).unwrap_or_else(|e| {
                    eprintln!("Problem identifying changed files: {e}");
                    process::exit(2)
                });
            let dirty = &timestamps_newer_than(&file_list, &outfile).unwrap_or_else(|e| {
                eprintln!("Problem checking timestamp: '{e}'");
                process::exit(2)
            });
            if !dirty {
                println!("No files have changed. Skipping rebuild.");
                process::exit(1);
            }
        }
    }

    let all_source_data: HashMap<_, _> = cfg
        .sources
        .iter()
        .map(|source| {
            let reader = match Path::new(&source.filename).extension() {
                Some(ext) if ext == "xlsx" => {
                    let reader = ExcelSourceReader::new(
                        &source.filename,
                        &relative_root,
                        source.sheet.as_deref(),
                    );
                    Box::new(reader) as Box<dyn DataSourceReader>
                }
                Some(ext) if ext == "csv" => {
                    let delimiter = source.delimiter.unwrap_or(';');

                    let reader =
                        CsvSourceReader::new(&source.filename, &relative_root, Some(delimiter));
                    Box::new(reader) as Box<dyn DataSourceReader>
                }
                _ => {
                    eprintln!(
                        "Unsupported file extension for source file '{}'",
                        source.filename
                    );
                    process::exit(2);
                }
            };
            let source_data = reader.read().unwrap_or_else(|e| {
                eprintln!("Problem reading source file '{}': {e}", source.filename);
                process::exit(2);
            });
            (source.id.clone(), source_data)
        })
        .collect();

    let template_path = relative_root.join(&cfg.templatepath);
    let renderer = MiniJinja::new(globals, &template_path);

    let mut rendered = String::new();

    for template in &cfg.layout {
        rendered += &render_template(&renderer, template, &all_source_data, cfg.adjustspacing)
            .unwrap_or_else(|err| {
                bubble_error("Template Error", err);
                process::exit(2);
            });
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
                .build(fs::File::open(&path).unwrap());
            let mut old_file_content = String::new();
            reader.read_to_string(&mut old_file_content).unwrap();

            let diff = create_patch(&old_file_content, &rendered);

            if diff.hunks().is_empty() {
                eprintln!("No change from original version, exiting.");
                process::exit(1);
            } else if cfg.verifycontent {
                do_write_file =
                    ask_should_overwrite(&diff).expect("error: unable to read user input");
            }
        }
        if do_write_file {
            if path.exists() {
                let backup_path = path.with_extension(format!(
                    "{}.bak",
                    path.extension().unwrap().to_str().unwrap()
                ));
                fs::rename(&path, backup_path).expect("Failed to create backup file");
            }

            let mut f = fs::File::create(&path).unwrap_or_else(|err| {
                eprintln!(
                    "Problem creating output file '{}': {}",
                    &path.display(),
                    err
                );
                process::exit(2);
            });
            let (cow, _encoding, _b) = encoding_rs::WINDOWS_1252.encode(&rendered);
            f.write_all(&cow).unwrap();
        }
    } else {
        // TODO: || with input argument for writing to stdout
        println!("{rendered}");
    }
}

pub fn cmd_diff(file1: &Path, file2: &Path) {
    let mut file_content = vec![String::new(), String::new()];

    for (i, file) in [file1, file2].iter().enumerate() {
        if file.exists() {
            let mut reader = encoding_rs_io::DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding_rs::WINDOWS_1252))
                .build(fs::File::open(file).unwrap());
            reader.read_to_string(&mut file_content[i]).unwrap();
        } else {
            eprintln!("File not found: '{}'", &file.display());
            process::exit(1);
        }
    }

    let diff = create_patch(&file_content[0], &file_content[1]);
    let f = PatchFormatter::new().with_color();
    print!("{}", f.fmt_patch(&diff));
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    fn get_all_source_data() -> HashMap<String, DataSourceRows> {
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

            let source_data = reader.read().unwrap();
            all_source_data.insert(source.id.to_string(), source_data);
        }
        all_source_data
    }

    #[test]
    fn ensure_has_extension_works() {
        let before = Path::new("file.extension");
        assert_eq!(before, set_extension_if_missing(before, "extension"));
        assert_eq!(
            before,
            set_extension_if_missing(Path::new("file"), "extension")
        );
        assert!(set_extension_if_missing(before, "other") == before);
    }

    #[test]
    fn render_with_normal_values() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"));
        let template = config::Template {
            name: "01_normals.tmpl".to_string(),
            source: Some("main".to_string()),
            ..Default::default()
        };
        let all_source_data = get_all_source_data();
        let result = render_template(&renderer, &template, &all_source_data, true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "String: one\nString: one\nBool: true\nInteger: 1\nWhole float: 1\nFloat: 1.234\n\nString: two\nString: two\nBool: false\nInteger: 2\nWhole float: 2\nFloat: 2.3456\n\nString: three\nString: three\nBool: true\nInteger: 3\nWhole float: 3\nFloat: 34.56"
        );
    }

    #[test]
    fn render_with_special_values() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"));
        let template = config::Template {
            name: "02_specials.tmpl".to_string(),
            source: Some("errors".to_string()),
            ..Default::default()
        };
        let all_source_data = get_all_source_data();
        let result = render_template(&renderer, &template, &all_source_data, true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(
            result,
            "Empty: >|none|<\nError: >|#DIV/0!|<\n\nEmpty: >||<\nError: >|#N/A|<\n\nEmpty: >|\"\"|<\nError: >|#NAME?|<\n\nEmpty: >|\"\"|<\nError: >|#NULL!|<\n\nEmpty: >|none|<\nError: >|#NUM!|<\n\nEmpty: >||<\nError: >|#REF!|<\n\nEmpty: >|\"\"|<\nError: >|#VALUE!|<"
        );
    }

    #[test]
    fn render_with_global_variables() {
        let globals = ["glob".to_string(), "globvalue".to_string()];
        let renderer = MiniJinja::new(&globals, Path::new("tests/testdata/templates/"));
        let template = config::Template {
            name: "03_globals.tmpl".to_string(),
            ..Default::default()
        };
        let all_source_data = get_all_source_data();

        let result = render_template(&renderer, &template, &all_source_data, true).unwrap();
        assert_eq!(result.trim_end(), "Global: >|globvalue|<");
    }

    #[test]
    fn render_uses_latin1_encoding() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"));
        let template = config::Template {
            name: "06_encoding.tmpl".to_string(),
            ..Default::default()
        };
        let result = render_template(&renderer, &template, &HashMap::new(), true)
            .unwrap()
            .trim()
            .replace('\r', "");
        assert_eq!(result.trim_end(), "ae: æ\noe: ø\naa: å\ns^2: s²\nm^3: m³");
    }

    #[test]
    fn render_adjusts_spacing() {
        let renderer = MiniJinja::new(&[], Path::new("tests/testdata/templates/"));
        let template = config::Template {
            name: "00_plaintext.tmpl".to_string(),
            ..Default::default()
        };
        let result_false = render_template(&renderer, &template, &HashMap::new(), false).unwrap();
        let result_true = render_template(&renderer, &template, &HashMap::new(), true).unwrap();
        assert_eq!(&result_true[result_true.len() - 5..], ".\r\n\r\n");
        #[cfg(target_os = "windows")]
        assert_eq!(&result_false[result_false.len() - 4..], "c.\r\n");
        #[cfg(not(target_os = "windows"))]
        assert_eq!(&result_false[result_false.len() - 3..], "c.\n");
    }

    #[test]
    fn collect_file_list_works() {
        let sources = vec![
            config::Source {
                filename: "source1".to_string(),
                ..Default::default()
            },
            config::Source {
                filename: "source2".to_string(),
                ..Default::default()
            },
        ];
        let relative_root = Path::new("relative_root");
        let cfg_file = relative_root.join("config.yaml");

        let dir = tempfile::tempdir().unwrap();
        let dir_path = dir.path().to_path_buf();
        let subdir_path = dir_path.join("subdir");
        fs::create_dir(&subdir_path).unwrap();
        let file1 = dir_path.join("temp1");
        let file2 = dir_path.join("temp2");
        let file3 = subdir_path.join("temp3");

        fs::write(&file1, "content1").unwrap();
        fs::write(&file2, "content2").unwrap();
        fs::write(&file3, "content3").unwrap();

        let layout = vec![];
        let cfg = config::Config {
            outputfile: Some("outfile".to_string()),
            templatepath: String::from(dir.path().to_str().unwrap()),
            sources,
            layout,
            ..Default::default()
        };

        let result = collect_file_list(&cfg, &cfg_file, relative_root).unwrap();
        let mut expected = HashSet::new();
        for filename in [
            file1.to_str().unwrap(),
            file2.to_str().unwrap(),
            file3.to_str().unwrap(),
        ]
        .iter()
        {
            expected.insert(PathBuf::from("relative_root/templates").join(filename));
        }
        for filename in ["source1", "source2", "config.yaml"].iter() {
            expected.insert(PathBuf::from("relative_root").join(filename));
        }

        assert!(result.len() == 6);
        assert!(result == expected);
    }

    #[test]
    fn timestamps_newer_than_works() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;

        let file1_path = dir.path().join("file1.txt");
        let mut file1 = File::create(&file1_path)?;
        file1.write_all(b"file1 content")?;

        let file2_path = dir.path().join("file2.txt");
        let mut file2 = File::create(&file2_path)?;
        file2.write_all(b"file2 content")?;

        let mut files = HashSet::new();
        files.insert(file1_path);
        files.insert(file2_path);

        let outfile_path = dir.path().join("outfile.txt");
        let mut outfile = File::create(&outfile_path)?;
        outfile.write_all(b"outfile content")?;

        assert!(!timestamps_newer_than(&files, &outfile_path)?);

        // Modify one of the files to make it newer than the outfile
        std::thread::sleep(std::time::Duration::from_millis(50));

        file2.write_all(b"modified content")?;
        assert!(timestamps_newer_than(&files, &outfile_path)?);
        Ok(())
    }

    #[test]
    fn timestamps_newer_than_errors_on_missing_outfile() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file1_path = dir.path().join("file1.txt");
        let mut file1 = File::create(&file1_path)?;
        file1.write_all(b"file1 content")?;

        let outfile_path = dir.path().join("outfile.txt");

        let result = timestamps_newer_than(&HashSet::from([file1_path]), &outfile_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2)"));
        Ok(())
    }

    #[test]
    fn timestamps_newer_than_errors_on_missing_infile() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file1_path = dir.path().join("file1.txt");

        let outfile_path = dir.path().join("outfile.txt");
        let mut outfile = File::create(&file1_path)?;
        outfile.write_all(b"file1 content")?;

        let result = timestamps_newer_than(&HashSet::from([file1_path]), &outfile_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2)"));
        Ok(())
    }
}
