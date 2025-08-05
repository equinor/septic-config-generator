use crate::config;
use anyhow::Result;
use clap::Parser;
use schemars::schema_for;

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
    let schema = schema_for!(config::Config);
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
