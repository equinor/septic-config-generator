use crate::config::Config;
use crate::renderer::MiniJinja;
use diffy::{create_patch, PatchFormatter};
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;

pub mod args;
pub mod config;
pub mod datasource;
pub mod renderer;
pub type DataSourceRow = Vec<(String, HashMap<String, CtxDataType>)>;

#[derive(Debug)]
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

#[derive(Debug)]
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

fn ensure_has_extension(filename: &Path, extension: &str) -> PathBuf {
    let mut file = filename.to_path_buf();
    if file.extension().is_none() {
        file.set_extension(extension);
    }
    file
}

fn bubble_error(pretext: &str, err: Box<dyn std::error::Error>) {
    eprintln!("{pretext}: {err:#}");
    let mut err = &*err;
    while let Some(next_err) = err.source() {
        eprintln!();
        eprintln!("Above error caused by: {next_err:#}");
        err = next_err;
    }
}

pub fn cmd_make(cfg_file: &Path, globals: &[String]) {
    let cfg_file = ensure_has_extension(cfg_file, "yaml");
    let relative_root = PathBuf::from(cfg_file.parent().unwrap());

    let cfg = Config::new(&cfg_file).unwrap_or_else(|e| {
        eprintln!("Problem reading '{}': {}", &cfg_file.display(), e);
        process::exit(1)
    });

    let mut all_source_data: HashMap<String, DataSourceRow> = HashMap::new();

    for source in &cfg.sources {
        let path = relative_root.join(&source.filename);
        let source_data = datasource::read(&path, &source.sheet).unwrap_or_else(|e| {
            eprintln!("Problem reading source file '{}': {e}", path.display());
            process::exit(1);
        });
        all_source_data.insert(source.id.to_string(), source_data);
    }

    let template_path = relative_root.join(&cfg.templatepath);
    let renderer = MiniJinja::new(globals, &template_path);

    let mut rendered = String::new();

    for template in &cfg.layout {
        if template.source.is_none() {
            let mut tmpl_rend = renderer.render(&template.name, ()).unwrap_or_else(|err| {
                bubble_error("Template error", err);
                process::exit(1);
            });
            if cfg.adjustspacing {
                tmpl_rend = tmpl_rend.trim_end().to_string();
                tmpl_rend.push_str("\n\n");
            }
            rendered.push_str(&tmpl_rend);
        } else {
            let src_name = &template.source.clone().unwrap();

            let keys: Vec<String> = all_source_data[src_name]
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

            for (key, row) in &all_source_data[src_name] {
                if items_set.contains(key) {
                    let mut tmpl_rend =
                        renderer
                            .render(&template.name, Some(row))
                            .unwrap_or_else(|err| {
                                bubble_error("Template Error", err);
                                process::exit(1);
                            });

                    if cfg.adjustspacing {
                        tmpl_rend = tmpl_rend.trim_end().to_string();
                        tmpl_rend.push_str("\n\n");
                    }
                    rendered.push_str(&tmpl_rend);
                }
            }
        }
    }
    if cfg.adjustspacing {
        rendered = rendered.trim_end().to_string();
    }

    if cfg.outputfile.is_none() {
        // TODO: || with input argument for writing to stdout
        println!("{rendered}");
    } else {
        let path = relative_root.join(cfg.outputfile.unwrap());

        let mut do_write_file = true;
        let mut has_diff = false;

        if path.exists() {
            let mut reader = encoding_rs_io::DecodeReaderBytesBuilder::new()
                .encoding(Some(encoding_rs::WINDOWS_1252))
                .build(fs::File::open(&path).unwrap());
            let mut old_file_content = String::new();
            reader.read_to_string(&mut old_file_content).unwrap();

            let diff = create_patch(&old_file_content, &rendered);

            has_diff = !diff.hunks().is_empty();
            if !has_diff {
                do_write_file = false;
            } else if has_diff && cfg.verifycontent {
                let f = PatchFormatter::new().with_color();
                print!("{}", f.fmt_patch(&diff));
                print!("\n\nReplace original? [Y]es or [N]o: ");
                let mut response = String::new();
                io::stdout().flush().unwrap();
                io::stdin()
                    .read_line(&mut response)
                    .expect("error: unable to read user input");

                do_write_file = response.len() > 1
                    && response
                        .trim_end()
                        .chars()
                        .last()
                        .unwrap()
                        .to_lowercase()
                        .next()
                        .unwrap()
                        == 'y';
            }
        }
        if path.exists() && !has_diff {
            eprintln!("No change from original version, exiting.");
        } else if do_write_file {
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
                process::exit(1);
            });
            let (cow, _encoding, _b) = encoding_rs::WINDOWS_1252.encode(&rendered);
            f.write_all(&cow).unwrap();
        }
    }
}

pub fn cmd_diff(file1: &Path, file2: &Path) {
    let mut file1_content = String::new();
    let mut file2_content = String::new();

    if file1.exists() {
        let mut reader1 = encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::WINDOWS_1252))
            .build(fs::File::open(file1).unwrap());
        reader1.read_to_string(&mut file1_content).unwrap();
    } else {
        eprintln!("File not found: '{}'", &file1.display());
        process::exit(1);
    }
    if file2.exists() {
        let mut reader2 = encoding_rs_io::DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::WINDOWS_1252))
            .build(fs::File::open(file2).unwrap());
        reader2.read_to_string(&mut file2_content).unwrap();
    } else {
        eprintln!("File not found: '{}'", &file2.display());
        process::exit(1);
    }
    let diff = create_patch(&file1_content, &file2_content);
    let f = PatchFormatter::new().with_color();
    print!("{}", f.fmt_patch(&diff));
}
