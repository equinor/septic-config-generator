use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Generate Septic config
    Make(Make),
    /// Show difference between two text files
    Diff(Diff),
    /// Check septic .out and .cnc files for error messages
    Checklogs(Checklogs),
    /// Check for new versions of this tool and auto-update
    Update(Update),
    /// Work with draw.io files
    Drawio(Drawio),
    /// Export json schema for yaml config
    Schema(Schema),
}

mod checklogs;
mod diff;
mod drawio;
mod make;
mod schema;
mod update;

pub use checklogs::Checklogs;
pub use diff::Diff;
pub use drawio::Drawio;
pub use make::Make;
pub use schema::Schema;
pub use update::Update;
