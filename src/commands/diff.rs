use anyhow::{Context, Result};
use clap::Parser;
use diffy::{create_patch, PatchFormatter};
use encoding_rs_io::DecodeReaderBytesBuilder;
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
pub struct Diff {
    pub file1: PathBuf,
    pub file2: PathBuf,
}

impl Diff {
    pub fn execute(&self) {
        let result = cmd_diff(&self.file1, &self.file2);
        match result {
            Ok(res) => {
                if let Some(diff) = res {
                    println!("{diff}");
                    std::process::exit(1)
                }
            }
            Err(err) => {
                eprintln!("{err:#}");
                std::process::exit(2)
            }
        }
    }
}

fn cmd_diff(file1: &Path, file2: &Path) -> Result<Option<String>> {
    let mut file_content = [String::new(), String::new()];

    for (i, file) in [file1, file2].iter().enumerate() {
        let file = fs::File::open(file).context(file.display().to_string())?;
        let mut reader = DecodeReaderBytesBuilder::new()
            .encoding(Some(encoding_rs::WINDOWS_1252))
            .build(file);
        reader.read_to_string(&mut file_content[i])?;
    }

    let diff = create_patch(&file_content[0], &file_content[1]);

    if diff.to_string().trim_end().ends_with("+++ modified") {
        Ok(None)
    } else {
        let formatted_diff = PatchFormatter::new()
            .with_color()
            .fmt_patch(&diff)
            .to_string();
        Ok(Some(formatted_diff))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn create_temp_file_with_content(content: &str) -> Result<NamedTempFile> {
        let mut temp_file = NamedTempFile::new()?;
        write!(temp_file, "{}", content)?;
        Ok(temp_file)
    }

    #[test]
    fn cmd_diff_fail_on_missing_file() -> Result<()> {
        let file1 = create_temp_file_with_content("File 1 content")?;
        let file2 = Path::new("nonexistent.txt");
        let result = cmd_diff(file1.path(), file2);
        assert!(result.is_err());
        let error_message = result.unwrap_err().root_cause().to_string();
        assert!(error_message.contains("(os error 2)"));
        Ok(())
    }

    #[test]
    fn cmd_diff_returns_diff_with_colors() -> Result<()> {
        let file1 = create_temp_file_with_content("File 1 content")?;
        let file2 = create_temp_file_with_content("File 2 content")?;
        let diff_output = cmd_diff(file1.path(), file2.path())?.expect("Expected diff, got None");
        assert!(diff_output.contains("--- original"));
        assert!(diff_output.contains("-File 1 content"));
        assert!(diff_output.contains("+File 2 content"));
        assert!(diff_output.contains("\x1b[")); // Check for color codes
        Ok(())
    }

    #[test]
    fn cmd_diff_no_diff() -> Result<()> {
        let file1 = create_temp_file_with_content("File common content")?;
        let file2 = create_temp_file_with_content("File common content")?;
        let diff_output = cmd_diff(file1.path(), file2.path())?;
        assert!(diff_output.is_none());
        Ok(())
    }
}
