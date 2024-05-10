use crate::config;
use crate::config::Counter as CounterConfig;
use crate::datasource::DataSourceRows;
use chrono::Local;
use minijinja::value::{from_args, Kwargs, Rest, Value, ValueKind};
use minijinja::{Environment, Error, ErrorKind};
use serde::Serialize;
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

const SCG_VERSION: &str = env!("CARGO_PKG_VERSION");

struct CounterMap {
    counters: HashMap<String, i32>,
}

impl CounterMap {
    fn new() -> Self {
        Self {
            counters: HashMap::new(),
        }
    }

    pub fn create(&mut self, name: &str, init_val: Option<i32>) -> anyhow::Result<()> {
        if self
            .counters
            .insert(name.to_owned(), init_val.unwrap_or(0))
            .is_some()
        {
            anyhow::bail!("Counter '{}' already exists", name)
        }
        Ok(())
    }

    pub fn increment(&mut self, name: &str, value: Option<i32>) -> Result<i32, Error> {
        let counter = self.counters.get_mut(name).ok_or_else(|| {
            Error::new(
                ErrorKind::InvalidOperation,
                format!("Counter '{}' not found", name),
            )
        })?;
        let new_value = value.map_or_else(|| *counter + 1, |v| v);
        *counter = new_value;
        Ok(new_value)
    }
}

fn filt_unpack(v: Value, unpack_keys: Rest<Value>) -> Result<Value, Error> {
    let (item_keys, _): (&[Value], Kwargs) = from_args(&unpack_keys)?;
    match v.kind() {
        ValueKind::Map => {
            let items_are_maps = v
                .try_iter()
                .unwrap()
                .all(|key| v.get_item(&key).unwrap().kind() == ValueKind::Map);
            if !items_are_maps {
                let rv = match item_keys.len() {
                    1 => v.get_item(&item_keys[0]).unwrap(),
                    _ => item_keys
                        .iter()
                        .map(|key| v.get_item(key).unwrap())
                        .collect(),
                };
                Ok(rv)
            } else {
                Err(Error::new(
                    ErrorKind::CannotUnpack,
                    "input is not a list of maps (source) or map (source row)",
                ))
            }
        }
        ValueKind::Seq => {
            let items_are_maps = v
                .try_iter()
                .unwrap()
                .all(|val| val.kind() == ValueKind::Map);
            if items_are_maps {
                let rv = v
                    .try_iter()
                    .unwrap()
                    .map(|value| {
                        let inner_vec: Vec<Value> = item_keys
                            .iter()
                            .map(|key| value.get_item(key).unwrap())
                            .collect();
                        if item_keys.len() == 1 {
                            inner_vec.into_iter().next().unwrap()
                        } else {
                            Value::from(inner_vec)
                        }
                    })
                    .collect();
                Ok(rv)
            } else {
                Err(Error::new(
                    ErrorKind::CannotUnpack,
                    "input is not a list of maps (source) or map (source row)",
                ))
            }
        }
        _ => Err(Error::new(
            ErrorKind::CannotUnpack,
            "input is not a list of maps (source) or map (source row)",
        )),
    }
}

fn filt_values(v: Value) -> Result<Value, Error> {
    eprintln!("The 'values' filter has been deprecated as it is no longer needed. Will be removed in an upcoming release. Please do not use.");
    Ok(v)
}

fn filt_bitmask(value: Value, length: Option<usize>) -> Result<String, Error> {
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

fn func_timestamp(format: Option<&str>) -> String {
    let fmt = format.unwrap_or("%Y-%m-%d %H:%M:%S");
    Local::now().format(fmt).to_string()
}

fn global_gitcommit(long: bool) -> String {
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
    let mut path = PathBuf::from(template_path);
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
    pub fn new(
        globals: &[String],
        template_path: &Path,
        counter_list: Option<Vec<CounterConfig>>,
    ) -> anyhow::Result<MiniJinja<'a>> {
        let mut renderer = MiniJinja {
            env: Environment::new(),
        };
        let counters = Arc::new(Mutex::new(CounterMap::new()));
        if let Some(cnts) = counter_list {
            for counter in cnts {
                counters
                    .lock()
                    .unwrap()
                    .create(&counter.name.clone(), counter.value)?;
                let increment_closure = {
                    let counters = counters.clone();
                    let name = counter.name.clone();
                    move |value: Option<i32>| counters.lock().unwrap().increment(&name, value)
                };
                renderer
                    .env
                    .add_function(counter.name.clone(), increment_closure);
            }
        }

        renderer.add_globals(globals);
        renderer
            .env
            .add_global("scgversion", String::from(SCG_VERSION));
        renderer
            .env
            .add_global("gitcommit", global_gitcommit(false));
        renderer
            .env
            .add_global("gitcommitlong", global_gitcommit(true));
        renderer.env.add_function("now", func_timestamp);
        renderer.env.add_filter("bitmask", filt_bitmask);
        renderer.env.add_filter("values", filt_values);
        renderer.env.add_filter("unpack", filt_unpack);
        renderer.env.set_formatter(erroring_formatter);
        // renderer.env.set_debug(false);  // TODO: enable via cmdline flag?

        let local_template_path = template_path.to_path_buf();

        renderer
            .env
            .set_loader(move |name| load_template(&local_template_path, name));
        Ok(renderer)
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn render<S: Serialize>(&self, template_name: &str, ctx: S) -> anyhow::Result<String> {
        let tmpl = self.env.get_template(template_name)?;
        Ok(tmpl.render(&ctx)?)
    }

    pub fn render_template(
        &self,
        template: &config::Template,
        source_data: &HashMap<String, DataSourceRows>,
        adjust_spacing: bool,
    ) -> anyhow::Result<String> {
        let mut rendered = String::new();

        if let Some(src_name) = &template.source {
            let keys: Vec<String> = match source_data.get(src_name) {
                Some(data) => data.iter().map(|(key, _row)| key.clone()).collect(),
                None => anyhow::bail!(
                    "unknown source '{}' referenced in {}",
                    src_name,
                    template.name
                ),
            };

            let mut items_set: HashSet<String> = keys.iter().cloned().collect();

            if template.include.is_some() {
                items_set = items_set
                    .intersection(&template.include_set(&self.env))
                    .cloned()
                    .collect();
            }

            items_set = items_set
                .difference(&template.exclude_set())
                .cloned()
                .collect();

            if let Some(data) = source_data.get(src_name) {
                for (key, row) in data {
                    if items_set.contains(key) {
                        let mut tmpl_rend = self.render(&template.name, Some(row))?;

                        if adjust_spacing {
                            tmpl_rend = tmpl_rend.trim_end().to_string();
                            tmpl_rend.push_str("\r\n\r\n");
                        }
                        rendered.push_str(&tmpl_rend);
                    }
                }
            }
        } else {
            rendered = self.render(&template.name, minijinja::context!())?;
        }

        if adjust_spacing {
            rendered = rendered.trim_end().to_string();
            rendered.push_str("\r\n\r\n");
        }

        Ok(rendered)
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
    use minijinja::{context, render};
    use regex::Regex;

    #[test]
    fn filt_unpack_source() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ A | unpack('a', 'b') }}",
            A => vec![
                context!(a => "aa", b => "bb"),
                context!(a => "cc", b => "dd"),
            ]
        };
        assert_eq!(result, "[[\"aa\", \"bb\"], [\"cc\", \"dd\"]]")
    }

    #[test]
    fn filt_unpack_source_single_key() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ A | unpack('b') }}",
            A => vec![
                context!(a => "aa", b => "bb"),
                context!(a => "cc", b => "dd"),
            ]
        };
        assert_eq!(result, "[\"bb\", \"dd\"]")
    }

    #[test]
    fn filt_unpack_source_invalid_key() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ A | unpack('a', 'c') }}",
            A => vec![
                context!(a => "aa", b => "bb"),
                context!(a => "cc", b => "dd"),
            ]
        };
        assert_eq!(result, "[[\"aa\", undefined], [\"cc\", undefined]]")
    }

    #[test]
    fn filt_unpack_source_row() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ AA | unpack('a', 'b') }}",
            AA => context!(a => "aa", b => "bb")
        };
        assert_eq!(result, "[\"aa\", \"bb\"]")
    }

    #[test]
    fn filt_unpack_source_row_single_key() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ AA | unpack('b') }}",
            AA => context!(a => "aa", b => "bb")
        };
        assert_eq!(result, "bb")
    }

    #[test]
    fn filt_unpack_source_row_invalid_key() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        let result = render! {in env, "{{ AA | unpack('a', 'c') }}",
            AA => context!(a => "aa", b => "bb")
        };
        assert_eq!(result, "[\"aa\", undefined]")
    }

    #[test]
    #[should_panic(expected = "input is not a list of maps")]
    fn filt_unpack_invalid_type() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        render! {in env, "{{ A | unpack('c') }}",
            A => "Some string"
        };
    }

    #[test]
    #[should_panic(expected = "input is not a list of maps")]
    fn filt_unpack_invalid_seq_item() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        render! {in env, "{{ A | unpack('c') }}",
            A => vec!(
                context!(a => "aa", b => "bb"),
                Value::from("Some string"),
            )
        };
    }

    #[test]
    #[should_panic(expected = "input is not a list of maps")]
    fn filt_unpack_invalid_map() {
        let mut env = Environment::new();
        env.add_filter("unpack", filt_unpack);
        render! {in env, "{{ A | unpack('c') }}",
            A => context!(
                AA => context!(a => "aa", b => "bb")
            )
        };
    }

    #[test]
    fn countermap_create() {
        let mut counter_map = CounterMap::new();
        assert!(counter_map.create("counter1", Some(10)).is_ok());
        assert!(counter_map.create("counter2", None).is_ok());
        assert!(counter_map.create("counter1", None).is_err());
    }

    #[test]
    fn countermap_increment_and_set() {
        let mut counter_map = CounterMap::new();
        counter_map.create("counter1", Some(10)).unwrap();
        assert_eq!(counter_map.increment("counter1", None).unwrap(), 11);
        assert_eq!(counter_map.increment("counter1", Some(20)).unwrap(), 20);
        assert_eq!(counter_map.increment("counter1", None).unwrap(), 21);
    }

    #[test]
    fn countermap_default_value_is_0() {
        let mut counter_map = CounterMap::new();
        counter_map.create("counter1", None).unwrap();
        assert_eq!(counter_map.increment("counter1", None).unwrap(), 1);
    }

    #[test]
    fn countermap_increment_nonexistent_fails() {
        let mut counter_map = CounterMap::new();
        counter_map.create("counter1", None).unwrap();
        assert!(counter_map.increment("counter2", None).is_err());
    }

    #[test]
    fn customfunction_timestamp_works() {
        let result = func_timestamp(None);
        let re = Regex::new(r"^\d{4}-\d{2}-\d{2} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
        let result = func_timestamp(Some("%a %d %b %Y %H:%M:%S"));
        let re = Regex::new(r"^\w{3} \d{1,2} \w{3} \d{4} \d{2}:\d{2}:\d{2}$").unwrap();
        assert!(re.is_match(&result));
    }

    #[test]
    fn customfunction_gitcommit_works() {
        let result = global_gitcommit(true);
        let re = Regex::new(r"^\w{40}$").unwrap();
        assert!(re.is_match(&result));
        let result = global_gitcommit(false);
        let re = Regex::new(r"^\w{7}$").unwrap();
        assert!(re.is_match(&result));
    }

    #[test]
    fn customfunction_bitmask_on_valid_integer() {
        let result = filt_bitmask(Value::from(1), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000001");
        let result = filt_bitmask(Value::from(31), Some(31)).unwrap();
        assert!(result == "1000000000000000000000000000000");
        let result = filt_bitmask(Value::from(vec![3]), Some(5)).unwrap();
        assert!(result == "00100");
    }

    #[test]
    fn customfunction_bitmask_errors_on_integer_oor() {
        let result = filt_bitmask(Value::from(-1), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("input value must be "));
        let result = filt_bitmask(Value::from(0), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000000");
        let result = filt_bitmask(Value::from(32), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("value is larger than"));
    }

    #[test]
    fn customfunction_bitmask_on_valid_sequence() {
        let result = filt_bitmask(Value::from(vec![0, 1, 3]), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000101");
        let result = filt_bitmask(Value::from(vec![1, 3, 31]), Some(31)).unwrap();
        assert!(result == "1000000000000000000000000000101");
        let result = filt_bitmask(Value::from(vec![1, 3]), Some(5)).unwrap();
        assert!(result == "00101");
    }

    #[test]
    fn customfunction_bitmask_errors_on_sequence_oor() {
        let result = filt_bitmask(Value::from(vec![-1, 1, 3]), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("input value must be "));
        let result = filt_bitmask(Value::from(vec![0, 1, 3]), Some(31)).unwrap();
        assert!(result == "0000000000000000000000000000101");
        let result = filt_bitmask(Value::from(vec![1, 3, 32]), Some(31));
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("value is larger than"));
    }
}
