use clap::Parser;
use minijinja::Error;
use septic_config_generator::config::Config;
use septic_config_generator::renderer::MiniJinjaRenderer;
use septic_config_generator::{args, datasource, DataSourceRow};
use std::collections::HashMap;
use std::collections::HashSet;
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

fn cmd_make(cfg_file: &Path, globals: &[String]) -> Result<(), Error> {
    let cfg_file = ensure_has_extension(cfg_file, "yaml");

    let cfg = Config::new(&cfg_file).unwrap_or_else(|e| {
        eprintln!("Problem reading '{}': {}", &cfg_file.display(), e);
        process::exit(1)
    });

    let mut all_source_data: HashMap<String, DataSourceRow> = HashMap::new();

    for source in &cfg.sources {
        let mut path = PathBuf::from(cfg_file.parent().unwrap());
        path.push(&source.filename);

        let source_data = datasource::read(&path, &source.sheet).unwrap_or_else(|e| {
            eprintln!("Problem reading source file '{}': {}", path.display(), e);
            process::exit(1);
        });
        all_source_data.insert(source.id.to_string(), source_data);
    }

    let template_path = PathBuf::from(cfg_file.parent().unwrap());
    let mut path = template_path;
    path.push(&cfg.templatepath);

    let renderer = MiniJinjaRenderer::new(globals, &path);

    let mut rendered = "".to_string();

    for template in &cfg.layout {
        if template.source.is_none() {
            let tmpl_rend = renderer.render(&template.name, ()).unwrap_or_else(|err| {
                eprintln!("Problem reading template: {err}");
                process::exit(1);
            });
            rendered.push_str(&tmpl_rend);
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
                    let tmpl_rend = renderer.render(&template.name, Some(row)).unwrap();
                    rendered.push_str(&tmpl_rend);
                }
            }
        }
    }
    println!("{rendered}");

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
