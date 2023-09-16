use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::RegexSet;
use std::error::Error;
use std::fs;
use std::io::prelude::*;
use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug)]
struct ErrorLine {
    line_num: usize,
    content: String,
}

#[derive(Parser, Debug)]
pub struct Checklogs {
    #[arg(
        value_name = "RUNDIR",
        help = "The SEPTIC rundir to search for outfiles"
    )]
    pub rundir: PathBuf,
}

impl Checklogs {
    pub fn execute(rundir: &Path) {
        cmd_check_logs(rundir);
    }
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

fn check_outfile(rundir: &Path) -> Result<(PathBuf, Vec<ErrorLine>), Box<dyn Error>> {
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

fn check_cncfile(rundir: &Path) -> Result<(PathBuf, Vec<ErrorLine>), Box<dyn Error>> {
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
                    format!("Failed to identify the newest .cnc file in {rundir:?}").into(),
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
) -> Result<Vec<ErrorLine>, Box<dyn Error>> {
    let file = fs::File::open(file_name)?;
    let reader = BufReader::new(file);
    let mut result: Vec<ErrorLine> = Vec::new();
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;

        if regex_set.is_match(&line) {
            let error_line = ErrorLine {
                line_num: line_number + 1,
                content: line,
            };
            result.push(error_line);
        }
    }
    Ok(result)
}

pub fn cmd_check_logs(rundir: &Path) {
    let check_functions = [check_outfile, check_cncfile];

    let mut found_warnings = false;

    for check_fn in &check_functions {
        match check_fn(rundir) {
            Ok((file, lines)) => {
                let file_name = file.file_name().unwrap().to_str().unwrap();
                if !lines.is_empty() {
                    found_warnings = true;
                }
                for line in &lines {
                    let line_num = format!("[{}]", line.line_num);
                    println!(
                        "{}{}: {}",
                        file_name.bright_green(),
                        line_num.bright_green(),
                        line.content.red()
                    );
                }
            }
            Err(err) => {
                eprintln!("Error checking file: {err}");
                process::exit(2);
            }
        }
    }
    if found_warnings {
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn check_outfile_errors_on_nonunique_file() {
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
    fn check_outfile_detects_all_known_warnings() {
        let rundir = r"tests/testdata/rundir/";
        let (file, lines) = check_outfile(Path::new(rundir)).unwrap();
        assert_eq!(file, PathBuf::from(rundir.to_owned() + "septic.out"));
        assert_eq!(lines.len(), 27);
    }
    #[test]
    fn check_cncfile_detects_all_known_warnings() {
        let rundir = r"tests/testdata/rundir/";
        let (file, lines) = check_cncfile(Path::new(rundir)).unwrap();
        assert_eq!(file, PathBuf::from(rundir.to_owned() + "septic.cnc"));
        assert_eq!(lines.len(), 2);
    }
}
