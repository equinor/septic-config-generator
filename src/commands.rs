use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate SEPTIC config
    Make(Make),
    /// Show difference between two text files
    Diff(Diff),
    /// Check septic .out and .cnc files for error messages
    Checklogs(Checklogs),
    /// Check for new versions of this tool and auto-update
    Update(Update),
}

mod checklogs;
mod diff;
mod make;
mod update;

pub use checklogs::Checklogs;
pub use diff::Diff;
pub use make::Make;
pub use update::Update;
