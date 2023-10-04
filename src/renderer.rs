use chrono::Local;
use minijinja::value::{Value, ValueKind};
use minijinja::{Environment, Error, ErrorKind};
use serde::Serialize;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const SCG_VERSION: &str = env!("CARGO_PKG_VERSION");

type CounterMap = Arc<Mutex<HashMap<String, i32>>>;

fn counterwrapper(name: String, counters: &CounterMap) -> impl Fn() -> Result<i32, Error> {
    let counters = counters.clone();
    move || counter(&name, &counters)
}

fn counter(name: &str, counters: &CounterMap) -> Result<i32, Error> {
    let mut counters = counters
        .lock()
        .map_err(|_| Error::new(ErrorKind::InvalidOperation, "Mutex lock poisoned"))?;
    let counter = counters.entry(name.to_owned()).or_insert(0);
    *counter += 1;
    Ok(*counter)
}

fn setcounterwrapper(name: String, counters: &CounterMap) -> impl Fn(i32) -> Result<i32, Error> {
    let counters = counters.clone();
    move |value| setcounter(&name, value, &counters)
}

fn setcounter(name: &str, value: i32, counters: &CounterMap) -> Result<i32, Error> {
    let mut counters = counters
        .lock()
        .map_err(|_| Error::new(ErrorKind::InvalidOperation, "Mutex lock poisoned"))?;
    counters.insert(name.to_owned(), value);
    Ok(value)
}

fn createcounter(name: &str, init_val: Option<i32>, counters: &CounterMap) -> Result<(), Error> {
    let mut counters = counters
        .lock()
        .map_err(|_| Error::new(ErrorKind::InvalidOperation, "Mutex lock poisoned"))?;
    if counters.contains_key(name) {
        return Err(Error::new(
            ErrorKind::InvalidOperation,
            "Counter already exists",
        ));
    }
    let init_val = init_val.unwrap_or(0);
    counters.insert(name.to_owned(), init_val);
    Ok(())
}

fn bitmask(value: Value, length: Option<usize>) -> Result<String, Error> {
    let value = match value.kind() {
        ValueKind::Number => Value::from(vec![value]),
        ValueKind::Seq => value,
        _ => {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                "input value must be a sequence of integers or an integer",
            ))
        }
    };
    let length = length.unwrap_or(31);

    let mut mask = vec!['0'; length];
    for elem in value.as_seq().unwrap().iter() {
        let pos = usize::try_from(elem).map_err(|_| {
            Error::new(
                ErrorKind::InvalidOperation,
                "input value must be a sequence of integers or an integer",
            )
        })?;
        if (1..=length).contains(&pos) {
            mask[length - pos] = '1';
        } else if pos > length {
            return Err(Error::new(
                ErrorKind::InvalidOperation,
                format!("value is larger than mask size ({pos} > {length})"),
            ));
        }
    }

    Ok(mask.into_iter().collect())
}

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
) -> Result<(), Error> {
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
        let counters = CounterMap::new(Mutex::new(HashMap::new()));
        createcounter("teller", Some(15), &counters).unwrap(); // TODO: Removeme
        renderer.add_globals(globals);
        renderer
            .env
            .add_global("scgversion", String::from(SCG_VERSION));
        renderer.env.add_global("gitcommit", gitcommit(false));
        renderer.env.add_global("gitcommitlong", gitcommit(true));
        renderer
            .env
            .add_function("teller", counterwrapper("teller".to_string(), &counters)); // TODO: Removeme
        renderer.env.add_function(
            "setteller",
            setcounterwrapper("teller".to_string(), &counters),
        ); // TODO: Removeme
        renderer.env.add_function("now", timestamp);
        renderer.env.add_filter("bitmask", bitmask);
        renderer.env.set_formatter(erroring_formatter);

        let local_template_path = template_path.to_path_buf();

        renderer
            .env
            .set_loader(move |name| load_template(&local_template_path, name));
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
    fn customfunction_timestamp_works() {
        let result = timestamp(None);
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
        let result = timestamp(Some("%a %d %b %Y %H:%M:%S"));
        let re = Regex::new(r"^\w{3} \d{1,2} \w{3} \d{4} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
    }

    #[test]
    fn customfunction_gitcommit_works() {
        let result = gitcommit(true);
        let re = Regex::new(r"^\w{40}$").unwrap();
        assert!(re.is_match(&result));
        let result = gitcommit(false);
        let re = Regex::new(r"^\w{7}$").unwrap();
        assert!(re.is_match(&result));
    }

    #[test]
    fn customfunction_bitmask_on_valid_integer() {
        let result = bitmask(Value::from(1), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000001");
        let result = bitmask(Value::from(31), Some(31)).unwrap();
        assert!(result == "1000000000000000000000000000000");
        let result = bitmask(Value::from(vec![3]), Some(5)).unwrap();
        assert!(result == "00100");
    }

    #[test]
    fn customfunction_bitmask_errors_on_integer_oor() {
        let result = bitmask(Value::from(-1), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("input value must be "));
        let result = bitmask(Value::from(0), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000000");
        let result = bitmask(Value::from(32), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("value is larger than"));
    }

    #[test]
    fn customfunction_bitmask_on_valid_sequence() {
        let result = bitmask(Value::from(vec![0, 1, 3]), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000101");
        let result = bitmask(Value::from(vec![1, 3, 31]), Some(31)).unwrap();
        assert!(result == "1000000000000000000000000000101");
        let result = bitmask(Value::from(vec![1, 3]), Some(5)).unwrap();
        assert!(result == "00101");
    }

    #[test]
    fn customfunction_bitmask_errors_on_sequence_oor() {
        let result = bitmask(Value::from(vec![-1, 1, 3]), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("input value must be "));
        let result = bitmask(Value::from(vec![0, 1, 3]), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000101");
        let result = bitmask(Value::from(vec![1, 3, 32]), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("value is larger than"));
    }
}
