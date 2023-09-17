use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate SEPTIC config
    Make(Make),
    /// Show difference between two text files
    Diff(Diff),
    /// Check septic .out and .cnc files for error messages
    Checklogs(Checklogs),
}

mod checklogs;
mod diff;
mod make;

pub use checklogs::Checklogs;
pub use diff::Diff;
pub use make::Make;
