use anyhow::{Result, anyhow};
use clap::Parser;
use colored::Colorize;
use glob::glob;
use regex::RegexSet;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct ErrorLine {
    line_num: usize,
    content: String,
}

#[derive(Debug)]
enum CheckLogsError {
    CheckError(String),
    WarningsFound,
}

impl std::fmt::Display for CheckLogsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CheckLogsError::CheckError(s) => write!(f, "Error checking file: {s}"),
            CheckLogsError::WarningsFound => write!(f, "Warnings were found"),
        }
    }
}
impl Error for CheckLogsError {}

#[derive(Parser, Debug)]
pub struct Checklogs {
    #[arg(
        value_name = "RUNDIR",
        help = "The Septic rundir to search for outfiles"
    )]
    pub rundir: PathBuf,
}

impl Checklogs {
    pub fn execute(&self) {
        let result = cmd_check_logs(&self.rundir);
        match result {
            Ok(_) => (),
            Err(err) => match err.downcast_ref() {
                Some(CheckLogsError::CheckError(_)) => {
                    eprintln!("{:#}", err);
                    std::process::exit(2);
                }
                Some(CheckLogsError::WarningsFound) => {
                    std::process::exit(1);
                }
                None => (),
            },
        }
    }
}

fn cmd_check_logs(rundir: &Path) -> Result<()> {
    let check_functions = [check_outfile, check_cncfile];

    let mut found_warnings = false;

    for check_fn in &check_functions {
        let check_result = check_fn(rundir);
        match check_result {
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
                return Err(anyhow!(CheckLogsError::CheckError(err.to_string())));
            }
        }
    }
    if found_warnings {
        return Err(anyhow!(CheckLogsError::WarningsFound));
    }
    Ok(())
}

fn get_newest_file(files: &[PathBuf]) -> Option<&PathBuf> {
    files
        .iter()
        .filter_map(|file| {
            fs::metadata(file)
                .ok()?
                .modified()
                .ok()
                .map(|time| (file, time))
        })
        .max_by_key(|&(_, time)| time)
        .map(|(file, _)| file)
}

fn check_outfile(rundir: &Path) -> Result<(PathBuf, Vec<ErrorLine>)> {
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
        0 => return Err(anyhow!("No .out file found in {:?}", &rundir)),
        1 => pathvec[0].clone(),
        _ => return Err(anyhow!("More than one .out file found in {:?}", &rundir)),
    };
    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
}

fn check_cncfile(rundir: &Path) -> Result<(PathBuf, Vec<ErrorLine>)> {
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
        0 => return Err(anyhow!("No .cnc file found in {:?}", &rundir)),
        1 => pathvec[0].clone(),
        _ => {
            if let Some(newest_file) = get_newest_file(&pathvec) {
                newest_file.clone()
            } else {
                return Err(anyhow!(
                    "Failed to identify the newest .cnc file in {rundir:?}"
                ));
            }
        }
    };

    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
}

fn process_single_startlog(file_name: &Path, regex_set: &RegexSet) -> Result<Vec<ErrorLine>> {
    let file = fs::File::open(file_name)?;
    let reader = BufReader::new(file);
    let mut error_lines: Vec<ErrorLine> = Vec::new();
    for (line_number, line) in reader.lines().enumerate() {
        let line = line?;
        if regex_set.is_match(&line) {
            let error_line = ErrorLine {
                line_num: line_number + 1,
                content: line,
            };
            error_lines.push(error_line);
        }
    }
    Ok(error_lines)
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{FileTime, set_file_mtime};
    use std::fs::File;
    use tempfile::tempdir;

    fn create_timestamped_file(dir: &Path, filename: &str, mod_time: i64) -> PathBuf {
        let file_path = dir.join(filename);
        File::create(&file_path).unwrap();
        let file_time = FileTime::from_unix_time(mod_time, 0);
        set_file_mtime(&file_path, file_time).unwrap();
        file_path
    }

    #[test]
    fn test_get_newest_file_returns_file_when_multiple_files() {
        let dir = tempdir().unwrap().into_path();
        let file_path1 = create_timestamped_file(&dir, "file1.txt", 100);
        let file_path2 = create_timestamped_file(&dir, "file2.txt", 200);
        let file_path3 = create_timestamped_file(&dir, "file3.txt", 300);

        let files = vec![file_path1, file_path2, file_path3];
        let newest_file = get_newest_file(&files);

        assert_eq!(newest_file, Some(&files[2]));
    }

    #[test]
    fn test_get_newest_file_returns_file_when_single_file() {
        let dir = tempdir().unwrap().into_path();
        let file_path1 = create_timestamped_file(&dir, "file1.txt", 100);

        let files = vec![file_path1];
        let newest_file = get_newest_file(&files);

        assert_eq!(newest_file, Some(&files[0]));
    }

    #[test]
    fn test_get_newest_file_returns_none_when_no_files() {
        let files = vec![];
        let newest_file = get_newest_file(&files);

        assert_eq!(newest_file, None);
    }

    #[test]
    fn check_outfile_errors_on_nonunique_file() {
        let dir = tempdir().unwrap();

        // With empty dir
        let result = check_outfile(dir.path());
        assert!(result.is_err());
        println!("{result:?}");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("No .out file found in")
        );

        // Add two .out files
        let file1_path = dir.path().join("file1.out");
        let _file1 = File::create(file1_path).unwrap();

        let file2_path = dir.path().join("file2.out");
        let _file2 = File::create(file2_path).unwrap();

        let result = check_outfile(dir.path());
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("More than one .out file found in")
        );
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
