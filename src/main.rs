use clap::Parser;
use septic_config_generator::commands::Commands;

#[derive(Parser, Debug)]
#[command(version, about, disable_colored_help = true, next_line_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Make(make) => make.execute(),
        Commands::Diff(diff) => diff.execute(),
        Commands::Checklogs(checklogs) => checklogs.execute(),
        Commands::Update(update) => update.execute(),
        Commands::Drawio(drawio) => drawio.execute(),
    }
}
