use clap::Parser;
use septic_config_generator::commands::Commands;

use septic_config_generator::cmd_make;

#[derive(Parser, Debug)]
#[command(version, about, disable_colored_help = true, next_line_help = true)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Make(make_args) => {
            cmd_make(
                &make_args.config_file,
                make_args.ifchanged,
                &make_args.var.unwrap_or_default(),
            );
        }
        Commands::Diff(diff) => {
            diff.execute();
        }
        Commands::Checklogs(checklogs) => {
            checklogs.execute();
        }
    }
}
