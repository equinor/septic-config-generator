use crate::config::Config;
use crate::renderer::MiniJinja;
use datasource::DataSourceRows;
use diffy::{create_patch, PatchFormatter};
use glob::glob;
use regex::RegexSet;
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

pub mod args;
pub mod config;
pub mod datasource;
pub mod renderer;

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

fn bubble_error(pretext: &str, err: Box<dyn Error>) {
    eprintln!("{pretext}: {err:#}");
    let mut err = err.as_ref();
    while let Some(next_err) = err.source() {
        eprintln!();
        eprintln!("Above error caused by: {next_err:#}");
        err = next_err;
    }
}

fn ensure_has_extension(filename: &Path, extension: &str) -> PathBuf {
    let mut file = filename.to_path_buf();
    if file.extension().is_none() {
        file.set_extension(extension);
    }

    file
}

fn read_config(cfg_file: &Path) -> Result<(Config, PathBuf), Box<dyn Error>> {
    let relative_root = PathBuf::from(cfg_file.parent().unwrap());
    let cfg = Config::new(cfg_file)?;

    Ok((cfg, relative_root))
}

fn read_source_data(
    source: &config::Source,
    relative_root: &Path,
) -> Result<DataSourceRows, Box<dyn Error>> {
    let path = relative_root.join(&source.filename);
    let source_data = datasource::read(&path, &source.sheet)?;

    Ok(source_data)
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
        files.insert(source_path.to_path_buf());
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
    let cfg_file = ensure_has_extension(cfg_file, "yaml");
    let (cfg, relative_root) = read_config(&cfg_file).unwrap_or_else(|e| {
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
            let source_data = read_source_data(source, &relative_root).unwrap_or_else(|e| {
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

fn get_newest_file(files: &[PathBuf]) -> Option<&PathBuf> {
    let mut newest_file: Option<&PathBuf> = None;
    let mut newest_time: Option<std::time::SystemTime> = None;

    for file in files {
        if let Ok(metadata) = fs::metadata(file) {
            if let Ok(modified_time) = metadata.modified() {
                if newest_time.is_none() || modified_time > newest_time.unwrap() {
                    newest_file = Some(file);
                    newest_time = Some(modified_time);
                }
            }
        }
    }

    newest_file
}

fn check_outfile(rundir: &Path) -> Result<(PathBuf, Vec<String>), Box<dyn Error>> {
    let regex_set = RegexSet::new([
        r"ERROR",
        r"WARNING",
        r"ILLEGAL",
        r"MISSING",
        r"FMU error:",
        r"^No Xvr match",
        r"^No matching XVR found for SopcEvr",
        r"INFO:",
    ])?;
    let entries = glob(rundir.join("*.out").to_str().unwrap())?;
    let pathvec: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
    let path = match pathvec.len() {
        0 => return Err(format!("No .out file found in {:?}", &rundir).into()),
        1 => pathvec[0].clone(),
        _ => {
            return Err(format!(
                "More than one .out file found in {:?}: {:?}",
                &rundir,
                pathvec
                    .iter()
                    .map(|path| path.file_name().unwrap().to_string_lossy())
                    .collect::<Vec<_>>()
            )
            .into())
        }
    };
    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
}

fn check_cncfile(rundir: &Path) -> Result<(PathBuf, Vec<String>), Box<dyn Error>> {
    let startlogs_dir = rundir.join("startlogs");
    let rundir = if startlogs_dir.exists() && startlogs_dir.is_dir() {
        startlogs_dir
    } else {
        rundir.to_owned()
    };
    let regex_set = RegexSet::new([r"ERROR", r"UNABLE to connect"])?;

    let entries = glob(rundir.join("*.cnc").to_str().unwrap())?;
    let pathvec: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
    let path = match pathvec.len() {
        0 => return Err(format!("No .cnc file found in {:?}", &rundir).into()),
        1 => pathvec[0].clone(),
        _ => {
            if let Some(newest_file) = get_newest_file(&pathvec) {
                newest_file.clone()
            } else {
                return Err(
                    format!("Failed to identify the newest .cnc file in {:?}", rundir).into(),
                );
            }
        }
    };

    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
}

fn process_single_startlog(
    file_name: &Path,
    regex_set: &RegexSet,
) -> Result<Vec<String>, Box<dyn Error>> {
    let file = fs::File::open(file_name)?;
    let reader = BufReader::new(file);
    let mut result: Vec<String> = Vec::new();
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        let matches: Vec<_> = regex_set.matches(&line).into_iter().collect();

        if !matches.is_empty() {
            result.push(format!("[{}]: {}", line_number + 1, line));
        }
    }
    Ok(result)
}

pub fn cmd_check(rundir: PathBuf) {
    let check_functions = [check_outfile, check_cncfile];

    for check_fn in &check_functions {
        match check_fn(&rundir) {
            Ok((file, lines)) => {
                let file_name = file.file_name().unwrap().to_str().unwrap();
                for line in &lines {
                    println!("{file_name} {line}");
                }
            }
            Err(err) => {
                eprintln!("Error checking file: {}", err);
                process::exit(1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn test_ensure_has_extension() {
        let before = Path::new("file.extension");
        assert_eq!(before, ensure_has_extension(before, "extension"));
        assert_eq!(before, ensure_has_extension(Path::new("file"), "extension"));
        assert!(ensure_has_extension(before, "other") == before);
    }

    #[test]
    fn test_read_config_invalid_content() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("test.yaml");
        let mut file = File::create(&file_path).unwrap();

        // let mut file = NamedTempFile::new().unwrap();
        writeln!(file, "random").unwrap();
        let result = read_config(&file_path);
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
        let result = read_config(&file_path);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("missing field"));
    }

    #[test]
    fn test_read_config_file_does_not_exist() {
        let result = read_config(Path::new("nonexistent_file.yaml"));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2)"));
    }

    #[test]
    fn test_read_source_file_does_not_exist() {
        let source = config::Source {
            filename: String::from("nonexistent_file.xlsx"),
            id: String::from("myid"),
            sheet: String::from("mysheet"),
        };

        let relative_root = Path::new("./");
        let result = read_source_data(&source, relative_root);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("(os error 2"),);
    }

    #[test]
    fn test_read_source_file_sheet_does_not_exist() {
        let source = config::Source {
            filename: String::from("test.xlsx"),
            id: String::from("myid"),
            sheet: String::from("nonexistent_sheet"),
        };

        let relative_root = Path::new("tests/testdata");
        let result = read_source_data(&source, relative_root);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Cannot find sheet"));
    }

    fn get_all_source_data() -> HashMap<String, DataSourceRows> {
        let mut all_source_data: HashMap<String, DataSourceRows> = HashMap::new();
        let source_main = config::Source {
            filename: "test.xlsx".to_string(),
            id: "main".to_string(),
            sheet: "Normals".to_string(),
        };
        let source_errors = config::Source {
            filename: "test.xlsx".to_string(),
            id: "errors".to_string(),
            sheet: "Specials".to_string(),
        };
        for source in [source_main, source_errors] {
            let source_data = read_source_data(&source, Path::new("tests/testdata/")).unwrap();
            all_source_data.insert(source.id.to_string(), source_data);
        }
        all_source_data
    }

    #[test]
    fn test_render_template_with_normals() {
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
            "String: one\nBool: true\nInteger: 1\nWhole float: 1\nFloat: 1.234\n\nString: two\nBool: false\nInteger: 2\nWhole float: 2\nFloat: 2.3456\n\nString: three\nBool: true\nInteger: 3\nWhole float: 3\nFloat: 34.56"
        );
    }

    #[test]
    fn test_render_template_with_specials() {
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
    fn test_render_template_with_global() {
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
    fn test_render_template_encoding() {
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
    fn test_render_template_adjustspacing() {
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
    fn test_collect_file_list() {
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
    fn test_timestamps_newer_than() -> Result<(), Box<dyn Error>> {
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
    fn test_timestamps_newer_than_file_outfile_not_exists() -> Result<(), Box<dyn Error>> {
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
    fn test_timestamps_newer_than_file_infile_not_exists() -> Result<(), Box<dyn Error>> {
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

    #[test]
    fn test_check_outfile_not_unique_file() {
        let dir = tempdir().unwrap();

        // With empty dir
        let result = check_outfile(dir.path());
        assert!(result.is_err());
        println!("{result:?}");
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No .out file found in"));

        // Add two .out files
        let file1_path = dir.path().join("file1.out");
        let _file1 = File::create(file1_path).unwrap();

        let file2_path = dir.path().join("file2.out");
        let _file2 = File::create(file2_path).unwrap();

        let result = check_outfile(dir.path());
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("More than one .out file found in"));
    }

    #[test]
    fn test_check_outfile() {
        let rundir = r"tests/testdata/rundir/";
        let (file, lines) = check_outfile(Path::new(rundir)).unwrap();
        assert_eq!(file, PathBuf::from(rundir.to_owned() + "septic.out"));
        assert_eq!(lines.len(), 27);
    }
    #[test]
    fn test_check_cncfile() {
        let rundir = r"tests/testdata/rundir/";
        let (file, lines) = check_cncfile(Path::new(rundir)).unwrap();
        assert_eq!(file, PathBuf::from(rundir.to_owned() + "septic.cnc"));
        assert_eq!(lines.len(), 2);
    }
}
