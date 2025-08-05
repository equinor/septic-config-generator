use crate::config;
use anyhow::Result;
use clap::Parser;
use schemars::generate::SchemaSettings;

#[derive(Parser, Debug)]
pub struct Schema {}

impl Schema {
    pub fn execute(&self) {
        cmd_dump_schema().unwrap_or_else(|err| {
            eprintln!("{err}");
            std::process::exit(1);
        });
    }
}

fn cmd_dump_schema() -> Result<()> {
    let settings = SchemaSettings::draft07();
    let generator = settings.into_generator();
    let schema = generator.into_root_schema_for::<config::Config>();
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
