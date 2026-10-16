use anyhow::{Result, anyhow};
use clap::{Parser, ValueEnum};
use colored::Colorize;
use glob::glob;
use regex::RegexSet;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// .cnc files of other OPC connections are only checked if they are at most this much
/// older than the newest .cnc file.
const CNC_MAX_AGE_DIFFERENCE: Duration = Duration::from_secs(5 * 60);

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

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, ValueEnum)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warning,
    Error,
    Critical,
}

#[derive(Parser, Debug)]
pub struct Checklogs {
    #[arg(
        value_name = "RUNDIR",
        help = "The Septic rundir to search for .log, .out and .cnc files"
    )]
    pub rundir: PathBuf,
    #[arg(
        long,
        value_enum,
        default_value = "warning",
        help = "Minimum level to report in .log files (does not affect .out or .cnc files)"
    )]
    pub level: LogLevel,
}

impl Checklogs {
    pub fn execute(&self) {
        let result = cmd_check_logs(&self.rundir, self.level);
        match result {
            Ok(_) => (),
            Err(err) => match err.downcast_ref() {
                Some(CheckLogsError::CheckError(_)) => {
                    eprintln!("{err:#}");
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

fn cmd_check_logs(rundir: &Path, level: LogLevel) -> Result<()> {
    let checks = [
        check_logfile_or_outfile(rundir, level).map(|result| vec![result]),
        check_cncfiles(rundir),
    ];

    let mut found_warnings = false;

    for check_result in checks {
        match check_result {
            Ok(results) => {
                for (file, lines) in results {
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

fn check_logfile_or_outfile(rundir: &Path, level: LogLevel) -> Result<(PathBuf, Vec<ErrorLine>)> {
    if rundir.join("logs").is_dir() {
        check_logfile(rundir, level)
    } else {
        check_outfile(rundir)
    }
}

fn check_logfile(rundir: &Path, level: LogLevel) -> Result<(PathBuf, Vec<ErrorLine>)> {
    let logs_dir = rundir.join("logs");
    let entries = glob(logs_dir.join("*.log").to_str().unwrap())?;
    let mut paths = Vec::new();
    for entry in entries {
        let path = entry?;
        let rotated = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .and_then(|stem| stem.rsplit_once('.'))
            .is_some_and(|(_, suffix)| {
                !suffix.is_empty() && suffix.bytes().all(|digit| digit.is_ascii_digit())
            });
        let is_labupdates = path
            .file_name()
            .is_some_and(|name| name == "labupdates.log");
        if path.is_file() && !rotated && !is_labupdates {
            paths.push(path);
        }
    }
    let path = match paths.len() {
        0 => return Err(anyhow!("No active .log file found in {logs_dir:?}")),
        1 => paths[0].clone(),
        _ => get_newest_file(&paths)
            .ok_or_else(|| anyhow!("Failed to identify the newest .log file in {logs_dir:?}"))?
            .clone(),
    };
    let regex_set = RegexSet::new(
        LogLevel::value_variants()
            .iter()
            .filter(|&&candidate| candidate >= level)
            .map(|candidate| {
                let value = candidate.to_possible_value().unwrap();
                format!(r"\[{}\]", value.get_name())
            }),
    )?;
    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
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
        0 => return Err(anyhow!("No .out file found in {:?}", rundir)),
        1 => pathvec[0].clone(),
        _ => return Err(anyhow!("More than one .out file found in {:?}", rundir)),
    };
    let lines = process_single_startlog(&path, &regex_set)?;
    Ok((path, lines))
}

/// Name of the OPC connection a .cnc file belongs to, i.e. the file stem without
/// the trailing `_<date>_<time>` stamp
fn cnc_connection_name(path: &Path) -> String {
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or_default();
    let is_digits = |s: &str| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit());
    let mut parts = stem.rsplitn(3, '_');
    match (parts.next(), parts.next(), parts.next()) {
        (Some(time), Some(date), Some(name)) if is_digits(time) && is_digits(date) => {
            name.to_owned()
        }
        _ => stem.to_owned(),
    }
}

/// Selects the newest .cnc file for each OPC connection. Connections whose newest file is
/// more than `max_age` older than the newest file overall are considered stale and are skipped.
fn select_cnc_files(files: &[PathBuf], max_age: Duration) -> Vec<PathBuf> {
    let mut newest_per_connection: HashMap<String, (&PathBuf, SystemTime)> = HashMap::new();
    for file in files {
        let Some(time) = fs::metadata(file).ok().and_then(|m| m.modified().ok()) else {
            continue;
        };
        newest_per_connection
            .entry(cnc_connection_name(file))
            .and_modify(|entry| {
                if time > entry.1 {
                    *entry = (file, time);
                }
            })
            .or_insert((file, time));
    }
    let Some(overall_newest) = newest_per_connection.values().map(|&(_, time)| time).max() else {
        return Vec::new();
    };
    let mut selected: Vec<PathBuf> = newest_per_connection
        .into_values()
        .filter(|&(_, time)| overall_newest.duration_since(time).unwrap_or_default() <= max_age)
        .map(|(file, _)| file.clone())
        .collect();
    selected.sort();
    selected
}

fn check_cncfiles(rundir: &Path) -> Result<Vec<(PathBuf, Vec<ErrorLine>)>> {
    let startlogs_dir = rundir.join("startlogs");
    let rundir = if startlogs_dir.exists() && startlogs_dir.is_dir() {
        startlogs_dir
    } else {
        rundir.to_owned()
    };
    let regex_set = RegexSet::new([r"ERROR", r"UNABLE to connect"])?;
    let entries = glob(rundir.join("*.cnc").to_str().unwrap())?;
    let pathvec: Vec<PathBuf> = entries.filter_map(Result::ok).collect();
    if pathvec.is_empty() {
        return Err(anyhow!("No .cnc file found in {:?}", rundir));
    }
    let selected = select_cnc_files(&pathvec, CNC_MAX_AGE_DIFFERENCE);
    if selected.is_empty() {
        return Err(anyhow!(
            "Failed to identify the newest .cnc file in {rundir:?}"
        ));
    }
    selected
        .into_iter()
        .map(|path| {
            let lines = process_single_startlog(&path, &regex_set)?;
            Ok((path, lines))
        })
        .collect()
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
        let dir = tempdir().unwrap().keep();
        let file_path1 = create_timestamped_file(&dir, "file1.txt", 100);
        let file_path2 = create_timestamped_file(&dir, "file2.txt", 200);
        let file_path3 = create_timestamped_file(&dir, "file3.txt", 300);

        let files = vec![file_path1, file_path2, file_path3];
        let newest_file = get_newest_file(&files);

        assert_eq!(newest_file, Some(&files[2]));
    }

    #[test]
    fn test_get_newest_file_returns_file_when_single_file() {
        let dir = tempdir().unwrap().keep();
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
        let results = check_cncfiles(Path::new(rundir)).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(
            results[0].0,
            PathBuf::from(rundir.to_owned() + "septic.cnc")
        );
        assert_eq!(results[0].1.len(), 2);
    }

    #[test]
    fn cnc_connection_name_strips_datestamp() {
        assert_eq!(
            cnc_connection_name(Path::new("THISOPC_20261013_0819.cnc")),
            "THISOPC"
        );
        assert_eq!(
            cnc_connection_name(Path::new("MY_OPC_20261013_0819.cnc")),
            "MY_OPC"
        );
        assert_eq!(cnc_connection_name(Path::new("septic.cnc")), "septic");
        assert_eq!(cnc_connection_name(Path::new("a_b.cnc")), "a_b");
    }

    #[test]
    fn select_cnc_files_picks_newest_per_connection_and_skips_stale() {
        let dir = tempdir().unwrap();
        let hour = 3600;
        let base = 1_000_000_000;
        let this_old = create_timestamped_file(dir.path(), "THISOPC_20261012_0819.cnc", base - 100);
        let this = create_timestamped_file(dir.path(), "THISOPC_20261013_0819.cnc", base);
        let that = create_timestamped_file(dir.path(), "THATOPC_20261013_0819.cnc", base - 60);
        let removed =
            create_timestamped_file(dir.path(), "OLDOPC_20261001_0819.cnc", base - 24 * hour);
        let files = vec![this_old, this.clone(), that.clone(), removed];
        assert_eq!(
            select_cnc_files(&files, Duration::from_secs(hour as u64)),
            vec![that, this]
        );
        assert!(select_cnc_files(&[], Duration::from_secs(hour as u64)).is_empty());
    }

    #[test]
    fn check_cncfiles_checks_all_active_connections() {
        let dir = tempdir().unwrap();
        let startlogs = dir.path().join("startlogs");
        fs::create_dir(&startlogs).unwrap();
        fs::write(startlogs.join("THISOPC_20261013_0819.cnc"), "ERROR a\n").unwrap();
        fs::write(
            startlogs.join("THATOPC_20261013_0819.cnc"),
            "ok\nUNABLE to connect\n",
        )
        .unwrap();
        let results = check_cncfiles(dir.path()).unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].0, startlogs.join("THATOPC_20261013_0819.cnc"));
        assert_eq!(results[0].1[0].line_num, 2);
        assert_eq!(results[1].0, startlogs.join("THISOPC_20261013_0819.cnc"));
        assert_eq!(results[1].1.len(), 1);
    }

    #[test]
    fn check_logfile_filters_each_threshold() {
        let dir = tempdir().unwrap();
        let logs = dir.path().join("logs");
        fs::create_dir(&logs).unwrap();
        let path = logs.join("myapp.log");
        let content = "[trace] trace\n[debug] debug\n[info] info\n[warning] warning\n[error] error\n[critical] critical\nERROR plain\n[ERROR] uppercase\n[unknown] unknown\nerror unbracketed\n";
        fs::write(&path, content).unwrap();
        let expected: Vec<_> = content.lines().take(6).collect();

        for (index, &level) in LogLevel::value_variants().iter().enumerate() {
            let (selected, lines) = check_logfile(dir.path(), level).unwrap();
            assert_eq!(selected, path);
            assert_eq!(lines.len(), 6 - index, "{level:?}");
            for (offset, line) in lines.iter().enumerate() {
                assert_eq!(line.line_num, index + offset + 1);
                assert_eq!(line.content, expected[index + offset]);
            }
        }
        fs::write(&path, "[info] below threshold\n").unwrap();
        assert!(
            check_logfile(dir.path(), LogLevel::Warning)
                .unwrap()
                .1
                .is_empty()
        );
        fs::write(&path, "[error] [critical] report once\n").unwrap();
        assert_eq!(
            check_logfile(dir.path(), LogLevel::Warning)
                .unwrap()
                .1
                .len(),
            1
        );
    }

    #[test]
    fn check_logfile_selects_newest_active_file_not_rotation() {
        let dir = tempdir().unwrap();
        let logs = dir.path().join("logs");
        fs::create_dir(&logs).unwrap();
        create_timestamped_file(&logs, "my.app.log", 100);
        let newest = create_timestamped_file(&logs, "app2.log", 200);
        create_timestamped_file(&logs, "labupdates.log", 400);
        for name in ["app2.1.log", "app2.2.log", "app2.10.log"] {
            create_timestamped_file(&logs, name, 300);
        }
        fs::create_dir(logs.join("directory.log")).unwrap();
        assert_eq!(
            check_logfile(dir.path(), LogLevel::Warning).unwrap().0,
            newest
        );
        fs::remove_file(&newest).unwrap();
        assert_eq!(
            check_logfile(dir.path(), LogLevel::Warning).unwrap().0,
            logs.join("my.app.log")
        );
    }

    #[test]
    fn check_logfile_or_outfile_preserves_fallback_and_new_layout_precedence() {
        let dir = tempdir().unwrap();
        let outfile = dir.path().join("myapp.out");
        fs::write(&outfile, "WARNING legacy\nINFO: legacy\n").unwrap();
        let (path, lines) = check_logfile_or_outfile(dir.path(), LogLevel::Critical).unwrap();
        assert_eq!(path, outfile);
        assert_eq!(lines.len(), 2);

        let logs = dir.path().join("logs");
        fs::write(&logs, "not a directory").unwrap();
        assert_eq!(
            check_logfile_or_outfile(dir.path(), LogLevel::Critical)
                .unwrap()
                .0,
            outfile
        );
        fs::remove_file(&logs).unwrap();
        fs::create_dir(&logs).unwrap();
        assert!(
            check_logfile_or_outfile(dir.path(), LogLevel::Warning)
                .unwrap_err()
                .to_string()
                .contains("No active .log file")
        );
        fs::write(logs.join("myapp.1.log"), "[error] rotation\n").unwrap();
        assert!(check_logfile_or_outfile(dir.path(), LogLevel::Warning).is_err());
        fs::create_dir(logs.join("directory.log")).unwrap();
        assert!(check_logfile_or_outfile(dir.path(), LogLevel::Warning).is_err());

        let logfile = logs.join("myapp.log");
        fs::write(&logfile, "[info] clean\n").unwrap();
        let (path, lines) = check_logfile_or_outfile(dir.path(), LogLevel::Warning).unwrap();
        assert_eq!(path, logfile);
        assert!(lines.is_empty());
        fs::write(&logfile, [0xff]).unwrap();
        assert!(check_logfile_or_outfile(dir.path(), LogLevel::Warning).is_err());
    }

    #[test]
    fn checklogs_parses_level_argument() {
        let args = Checklogs::try_parse_from(["checklogs", "rundir"]).unwrap();
        assert_eq!(args.level, LogLevel::Warning);
        for &level in LogLevel::value_variants() {
            let value = level.to_possible_value().unwrap();
            for args in [
                ["checklogs", "rundir", "--level", value.get_name()],
                ["checklogs", "--level", value.get_name(), "rundir"],
            ] {
                assert_eq!(Checklogs::try_parse_from(args).unwrap().level, level);
            }
        }
        assert!(Checklogs::try_parse_from(["checklogs", "rundir", "--level", "invalid"]).is_err());
        assert!(Checklogs::try_parse_from(["checklogs", "rundir", "--level"]).is_err());
    }

    #[test]
    fn checklogs_preserves_results_and_connect_checks() {
        let dir = tempdir().unwrap();
        let logs = dir.path().join("logs");
        let startlogs = dir.path().join("startlogs");
        fs::create_dir(&logs).unwrap();
        fs::create_dir(&startlogs).unwrap();
        let logfile = logs.join("myapp.log");
        let cncfile = startlogs.join("myapp.cnc");
        fs::write(&logfile, "[warning] warning\n").unwrap();
        fs::write(&cncfile, "connected\n").unwrap();
        assert!(cmd_check_logs(dir.path(), LogLevel::Error).is_ok());
        let error = cmd_check_logs(dir.path(), LogLevel::Warning).unwrap_err();
        assert!(matches!(
            error.downcast_ref(),
            Some(CheckLogsError::WarningsFound)
        ));

        fs::write(&cncfile, "UNABLE to connect\n").unwrap();
        let error = cmd_check_logs(dir.path(), LogLevel::Critical).unwrap_err();
        assert!(matches!(
            error.downcast_ref(),
            Some(CheckLogsError::WarningsFound)
        ));
        fs::remove_file(&logfile).unwrap();
        let error = cmd_check_logs(dir.path(), LogLevel::Warning).unwrap_err();
        assert!(matches!(
            error.downcast_ref(),
            Some(CheckLogsError::CheckError(_))
        ));

        let legacy = Path::new("tests/testdata/rundir/");
        let (_, lines) = check_logfile_or_outfile(legacy, LogLevel::Critical).unwrap();
        assert_eq!(lines.len(), 27);
        let error = cmd_check_logs(legacy, LogLevel::Critical).unwrap_err();
        assert!(matches!(
            error.downcast_ref(),
            Some(CheckLogsError::WarningsFound)
        ));
    }
}
