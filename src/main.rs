use clap::Parser;
use minijinja::{Environment, Error, Source};

use septic_config_generator::{args, config::Config, datasource};
use std::collections::HashMap;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

fn ensure_has_extension(filename: &Path, extension: &str) -> PathBuf {
    let mut file = filename.to_path_buf();
    if file.extension().is_none() {
        file.set_extension(extension);
    }
    file
}

fn _merge_maps(
    map1: &HashMap<String, String>,
    map2: &HashMap<String, String>,
) -> HashMap<String, String> {
    let mut merged = map1.clone();
    merged.extend(map2.iter().map(|(k, v)| (k.to_string(), v.to_string())));
    merged
}

fn add_globals(env: &mut Environment, globals: &[String]) {
    for chunk in globals.chunks(2) {
        let (key, val) = (chunk[0].to_string(), chunk[1].to_string());
        match val.as_str() {
            "true" => env.add_global(key, true),
            "false" => env.add_global(key, false),
            _ => match val.parse::<i64>() {
                Ok(i) => env.add_global(key, i),
                Err(_) => match val.parse::<f64>() {
                    Ok(f) => env.add_global(key, f),
                    Err(_) => env.add_global(key, val.to_owned()),
                },
            },
        }
    }
}

fn error_formatter(
    out: &mut minijinja::Output,
    state: &minijinja::State,
    value: &minijinja::value::Value,
) -> Result<(), Error> {
    // A crude way to stop execution when a variable is undefined.
    if let true = value.is_undefined() {
        return Err(Error::from(minijinja::ErrorKind::UndefinedError));
    }
    minijinja::escape_formatter(out, state, value)
}

fn cmd_make(cfg_file: &Path, globals: &[String]) -> Result<(), Error> {
    let cfg_file = ensure_has_extension(cfg_file, "yaml");

    let cfg = Config::new(&cfg_file).unwrap_or_else(|e| {
        eprintln!("Problem reading '{}': {}", &cfg_file.display(), e);
        process::exit(1)
    });

    let mut all_source_data: HashMap<String, datasource::RowItem> = HashMap::new();

    for source in &cfg.sources {
        let mut path = PathBuf::from(cfg_file.parent().unwrap());
        path.push(&source.filename);

        let source_data = datasource::read(&path, &source.sheet).unwrap_or_else(|e| {
            eprintln!("Problem reading source file '{}': {}", path.display(), e);
            process::exit(1);
        });
        all_source_data.insert(source.id.to_string(), source_data);
    }

    let mut env = Environment::new();

    add_globals(&mut env, globals);

    let template_path = PathBuf::from(cfg_file.parent().unwrap());
    env.set_source(Source::with_loader(move |name| {
        let mut path = template_path.clone();
        path.push(&cfg.templatepath);
        path.push(name);
        match fs::read_to_string(path) {
            Ok(result) => Ok(Some(result)),
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    Ok(None)
                } else {
                    Err(Error::new(
                        minijinja::ErrorKind::TemplateNotFound,
                        "failed to load template",
                    )
                    .with_source(err))
                }
            }
        }
    }));

    env.set_formatter(error_formatter);

    let mut rendered = "".to_string();

    for template in &cfg.layout {
        let tmpl = env.get_template(&template.name).unwrap_or_else(|e| {
            eprintln!("Problem reading template file: {e}");
            process::exit(1);
        });

        if template.source.is_none() {
            rendered.push_str(&tmpl.render({})?);
        } else {
            let src_name = &template.source.clone().unwrap();

            let keys: Vec<String> = all_source_data[src_name]
                .iter()
                .map(|(key, _row)| key.clone())
                .collect();

            let mut items_set: HashSet<String> = keys.iter().cloned().collect();

            if template.include.is_some() {
                items_set = items_set
                    .intersection(&template.include_set())
                    .cloned()
                    .collect();
            }

            items_set = items_set
                .difference(&template.exclude_set())
                .cloned()
                .collect();

            for (key, row) in all_source_data[src_name].iter() {
                if items_set.contains(key) {
                    let tmpl_rend = tmpl.render(row)?;
                    rendered.push_str(&tmpl_rend);
                }
            }
        }
        println!("{rendered}");
    }

    Ok(())
}

fn main() {
    let args = args::Cli::parse();

    match args.command {
        args::Commands::Make(make_args) => {
            cmd_make(&make_args.config_file, &make_args.var.unwrap_or_default()).unwrap();
        }
        args::Commands::Diff => todo!(),
    }
}
