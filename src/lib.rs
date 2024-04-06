use crate::config::Config;
use anyhow::Context;
use diffy::{create_patch, PatchFormatter};
use glob::glob;
use serde::Serialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs;
use std::io;
use std::io::prelude::*;

use std::path::{Path, PathBuf};

pub mod commands;
pub mod config;
pub mod datasource;
pub mod renderer;

#[derive(Clone, Debug, PartialEq, Eq)]
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

#[derive(Clone, Debug, PartialEq)]
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
            Self::Empty => serializer.serialize_unit(),
        }
    }
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
) -> anyhow::Result<HashSet<PathBuf>> {
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

fn timestamps_newer_than(files: &HashSet<PathBuf>, outfile: &PathBuf) -> anyhow::Result<bool> {
    let checktime = fs::metadata(outfile)
        .and_then(|metadata| metadata.modified())
        .with_context(|| format!("Failed to read timestamp for {outfile:?}"))?;
    for f in files {
        let systime = fs::metadata(f)
            .and_then(|metadata| metadata.modified())
            .with_context(|| format!("Failed to read timestamp for {f:?}"))?;
        if systime > checktime {
            return Ok(true);
        }
    }
    Ok(false)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::tempdir;

    #[test]
    fn collect_file_list_works() -> Result<(), Box<dyn Error>> {
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

        let dir = tempfile::tempdir()?;
        let dir_path = dir.path().to_path_buf();
        let subdir_path = dir_path.join("subdir");
        fs::create_dir(&subdir_path)?;
        let file1 = dir_path.join("temp1");
        let file2 = dir_path.join("temp2");
        let file3 = subdir_path.join("temp3");

        fs::write(&file1, "content1")?;
        fs::write(&file2, "content2")?;
        fs::write(&file3, "content3")?;

        let layout = vec![];
        let cfg = config::Config {
            outputfile: Some("outfile".to_string()),
            templatepath: String::from(dir.path().to_str().unwrap()),
            sources,
            layout,
            ..Default::default()
        };

        let result = collect_file_list(&cfg, &cfg_file, relative_root)?;
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
        Ok(())
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
        let err = result.as_ref().unwrap_err();
        assert!(err.root_cause().to_string().contains("(os error 2)"));
        assert!(err.to_string().contains("outfile.txt"));
        Ok(())
    }

    #[test]
    fn timestamps_newer_than_errors_on_missing_infile() -> Result<(), Box<dyn Error>> {
        let dir = tempdir()?;
        let file1_path = dir.path().join("file1.txt");

        let outfile_path = dir.path().join("outfile.txt");
        let mut outfile = File::create(&outfile_path)?;
        outfile.write_all(b"file1 content")?;

        let result = timestamps_newer_than(&HashSet::from([file1_path]), &outfile_path);
        assert!(result.is_err());
        let err = result.as_ref().unwrap_err();
        assert!(err.root_cause().to_string().contains("(os error 2)"));
        assert!(err.to_string().contains("file1.txt"));
        Ok(())
    }
}
