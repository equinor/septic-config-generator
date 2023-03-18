use chrono::Local;
use minijinja::{Environment, Error, ErrorKind, Source};
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

const SCG_VERSION: &str = env!("CARGO_PKG_VERSION");

fn timestamp(format: Option<&str>) -> String {
    let fmt = format.unwrap_or("%Y-%m-%d %H:%M:%S");
    Local::now().format(fmt).to_string()
}

fn gitcommit(long: bool) -> String {
    let args = if long {
        ["rev-parse", "--verify", "HEAD"]
    } else {
        ["rev-parse", "--short", "HEAD"]
    };

    std::process::Command::new("git")
        .args(args)
        .output()
        .map_or_else(
            |err| format!("***** Unable to execute git: {err:#} *****"),
            |cmd| String::from_utf8_lossy(&cmd.stdout).trim().to_string(),
        )
}

fn erroring_formatter(
    out: &mut minijinja::Output,
    state: &minijinja::State,
    value: &minijinja::value::Value,
) -> Result<(), minijinja::Error> {
    // A crude way to stop execution when a variable is undefined.
    if value.is_undefined() {
        return Err(Error::from(ErrorKind::UndefinedError));
    }
    minijinja::escape_formatter(out, state, value)
}

#[allow(clippy::unnecessary_wraps)]
fn load_template(template_path: &Path, name: &str) -> Result<Option<String>, Error> {
    let mut path = PathBuf::new();
    path.push(template_path);
    path.push(name);
    let file = match File::open(path) {
        Ok(f) => f,
        Err(err) => match err.kind() {
            std::io::ErrorKind::NotFound => return Ok(None),
            other_error => {
                dbg!(&other_error);
                panic!("Unknown error, please report it");
            }
        },
    };
    let mut reader = encoding_rs_io::DecodeReaderBytesBuilder::new()
        .encoding(Some(encoding_rs::WINDOWS_1252))
        .build(file);
    let mut content = String::new();

    match reader.read_to_string(&mut content) {
        Ok(_) => Ok(Some(content)),
        Err(err) => {
            dbg!(&err);
            panic!("Unknown error when reading template, please report it");
        }
    }
}

pub struct MiniJinja<'a> {
    pub env: Environment<'a>,
}

impl<'a> MiniJinja<'a> {
    pub fn new(globals: &[String], template_path: &Path) -> MiniJinja<'a> {
        let mut renderer = MiniJinja {
            env: Environment::new(),
        };
        renderer.add_globals(globals);
        renderer
            .env
            .add_global("scgversion", String::from(SCG_VERSION));
        renderer.env.add_global("gitcommit", gitcommit(false));
        renderer.env.add_global("gitcommitlong", gitcommit(true));
        renderer.env.add_function("now", timestamp);
        renderer.env.set_formatter(erroring_formatter);

        let local_template_path = template_path.to_path_buf();

        renderer.env.set_source(Source::with_loader(move |name| {
            load_template(&local_template_path, name)
        }));
        renderer
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn render<S: Serialize>(
        &self,
        template_name: &str,
        ctx: S,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let tmpl = match self.env.get_template(template_name) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(e)),
        };

        match tmpl.render(ctx) {
            Ok(r) => Ok(r),
            Err(e) => Err(Box::new(e)),
        }
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn render_to_file<S: Serialize, W: std::io::Write>(
        &self,
        template_name: &str,
        ctx: S,
        writer: W,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tmpl = match self.env.get_template(template_name) {
            Ok(t) => t,
            Err(e) => return Err(Box::new(e)),
        };

        match tmpl.render_to_write(ctx, writer) {
            Ok(()) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }

    fn add_globals(&mut self, globals: &[String]) {
        for chunk in globals.chunks(2) {
            let (key, val) = (chunk[0].to_string(), chunk[1].to_string());
            match val.as_str() {
                "true" => self.env.add_global(key, true),
                "false" => self.env.add_global(key, false),
                _ => match val.parse::<i64>() {
                    Ok(i) => self.env.add_global(key, i),
                    Err(_) => match val.parse::<f64>() {
                        Ok(f) => self.env.add_global(key, f),
                        Err(_) => self.env.add_global(key, val.clone()),
                    },
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regex::Regex;

    #[test]
    fn test_customfunction_timestamp() {
        let result = timestamp(None);
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
        let result = timestamp(Some("%a %d %b %Y %H:%M:%S"));
        let re = Regex::new(r"^\w{3} \d{1,2} \w{3} \d{4} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
    }

    #[test]
    fn test_customfunction_gitcommit() {
        let result = gitcommit(true);
        let re = Regex::new(r"^\w{40}$").unwrap();
        assert!(re.is_match(&result));
        let result = gitcommit(false);
        let re = Regex::new(r"^\w{7}$").unwrap();
        assert!(re.is_match(&result));
    }
}
