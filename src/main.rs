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

    let mut all_source_data: HashMap<
        String,
        HashMap<String, HashMap<String, datasource::DataTypeSer>>,
    > = HashMap::new();

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

            let mut items: HashSet<String> = all_source_data[src_name].keys().cloned().collect();

            if template.include.is_some() {
                items = items
                    .intersection(&template.include_set())
                    .cloned()
                    .collect();
            }

            items = items.difference(&template.exclude_set()).cloned().collect();

            for key in all_source_data[src_name].keys() {
                if items.contains(key) {
                    println!("{key}");
                    let ctx = &all_source_data[src_name][key];
                    let tmpl_rend = tmpl.render(ctx)?;
                    rendered.push_str(&tmpl_rend);
                }
            }

            // TODO: all_source_data[src] is not sorted.
            // Consider using Vec<(String, DataTypeSer)> or crate linked_hash_map
            // let filtered = all_source_data.iter().filter(|(key, _)| key == &target_string);
            // let result = filtered.next();
            //
            // let result = all_source_data.iter().find(|(key, _value)| key == &target);
        }
        // let res = tmpl.render(ctx)?;
        println!("{rendered}");
    }

    // env.set_debug(true);
    // let tmpl = env.get_template("hello.txt")?;
    // let res = tmpl.render(context! {nae => "World"})?;

    // println!("{res}");
    // println!("{:?}", env.source().unwrap());
    // println!("{:?}", all_source_data);
    // println!("{:?}", all_source_data["main"]["D02"]);
    // println!("{:?}", env);

    // Load templates from file:
    // https://github.com/mitsuhiko/minijinja/blob/main/examples/render-template/src/main.rs Ownership issues?
    // https://github.com/mitsuhiko/minijinja/blob/main/examples/loader/src/main.rs
    // https://github.com/mitsuhiko/minijinja/blob/main/examples/source/src/main.rs
    // Create context:
    // https://github.com/mitsuhiko/minijinja/blob/main/examples/dynamic-context/src/main.rs
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
