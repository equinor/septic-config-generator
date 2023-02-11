use chrono::Local;
use minijinja::{Environment, Error, ErrorKind, Source};
use serde::Serialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;

fn timestamp(format: &str) -> String {
    let now = Local::now();
    now.format(format).to_string()
}

fn gitcommit(short: bool) -> String {
    let args = match short {
        true => ["rev-parse", "--short", "HEAD"],
        _ => ["rev-parse", "--verify", "HEAD"],
    };
    std::process::Command::new("git")
        .args(args)
        .output()
        .map(|cmd| String::from_utf8_lossy(&cmd.stdout).trim().to_string())
        .unwrap_or_else(|err| format!("***** Unable to execute git: {err:#} *****"))
}

fn erroring_formatter(
    out: &mut minijinja::Output,
    state: &minijinja::State,
    value: &minijinja::value::Value,
) -> Result<(), minijinja::Error> {
    // A crude way to stop execution when a variable is undefined.
    if let true = value.is_undefined() {
        return Err(Error::from(ErrorKind::UndefinedError));
    }
    minijinja::escape_formatter(out, state, value)
}

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

pub struct MiniJinjaRenderer<'a> {
    pub env: Environment<'a>,
}

impl<'a> MiniJinjaRenderer<'a> {
    pub fn new(globals: &[String], template_path: &Path) -> MiniJinjaRenderer<'a> {
        let mut renderer = MiniJinjaRenderer {
            env: Environment::new(),
        };
        renderer.add_globals(globals);
        renderer.env.add_global("gitcommit", gitcommit(false));
        renderer.env.add_function("now", timestamp);
        renderer.env.set_formatter(erroring_formatter);

        let local_template_path = template_path.to_path_buf();

        renderer.env.set_source(Source::with_loader(move |name| {
            load_template(&local_template_path, name)
        }));
        renderer
    }

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
                        Err(_) => self.env.add_global(key, val.to_owned()),
                    },
                },
            }
        }
    }
}