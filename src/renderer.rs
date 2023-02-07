use minijinja::{Environment, Source};
use std::fs;
use std::path::Path;
use std::path::PathBuf;
pub struct MiniJinjaRenderer<'a> {
    pub env: Environment<'a>,
}

impl<'a> MiniJinjaRenderer<'a> {
    pub fn new(globals: &[String], template_path: &Path) -> MiniJinjaRenderer<'a> {
        let mut renderer = MiniJinjaRenderer {
            env: Environment::new(),
        };
        renderer.add_globals(globals);
        renderer.env.set_formatter(error_formatter);
        let local_template_path = template_path.to_path_buf();
        renderer.env.set_source(Source::with_loader(move |name| {
            let mut path = PathBuf::new();
            path.push(local_template_path.clone());
            path.push(name);
            match fs::read_to_string(path) {
                Ok(result) => Ok(Some(result)),
                Err(err) => {
                    if err.kind() == std::io::ErrorKind::NotFound {
                        Ok(None)
                    } else {
                        Err(minijinja::Error::new(
                            minijinja::ErrorKind::TemplateNotFound,
                            "failed to load template",
                        )
                        .with_source(err))
                    }
                }
            }
        }));
        renderer
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

fn error_formatter(
    out: &mut minijinja::Output,
    state: &minijinja::State,
    value: &minijinja::value::Value,
) -> Result<(), minijinja::Error> {
    // A crude way to stop execution when a variable is undefined.
    if let true = value.is_undefined() {
        return Err(minijinja::Error::from(minijinja::ErrorKind::UndefinedError));
    }
    minijinja::escape_formatter(out, state, value)
}

// fn add_globals(globals: &[String]) {
//     for chunk in globals.chunks(2) {
//         let (key, val) = (chunk[0].to_string(), chunk[1].to_string());
//         match val.as_str() {
//             "true" => self.add_global(key, true),
//             "false" => env.add_global(key, false),
//             _ => match val.parse::<i64>() {
//                 Ok(i) => env.add_global(key, i),
//                 Err(_) => match val.parse::<f64>() {
//                     Ok(f) => env.add_global(key, f),
//                     Err(_) => env.add_global(key, val.to_owned()),
//                 },
//             },
//         }
//     }
// }
