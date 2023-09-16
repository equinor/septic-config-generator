use clap::Parser;
use diffy::{create_patch, PatchFormatter};
use std::fs;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Parser, Debug)]
pub struct Diff {
    pub file1: PathBuf,
    pub file2: PathBuf,
}

impl Diff {
    pub fn execute(file1: &Path, file2: &Path) {
        cmd_diff(file1, file2);
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
