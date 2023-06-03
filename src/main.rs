use clap::Parser;
use septic_config_generator::{args, cmd_check_logs, cmd_diff, cmd_make};

fn main() {
    let args = args::Cli::parse();

    match args.command {
        args::Commands::Make(make_args) => {
            cmd_make(
                &make_args.config_file,
                make_args.ifchanged,
                &make_args.var.unwrap_or_default(),
            );
        }
        args::Commands::Diff(diff_args) => {
            cmd_diff(&diff_args.file1, &diff_args.file2);
        }
        args::Commands::Checklogs(check_args) => {
            cmd_check_logs(check_args.rundir);
        }
    }
}
