pub mod commands;
pub mod config;
pub mod datasource;
pub mod renderer;

#[cfg(target_os = "windows")]
pub const NEWLINE: &str = "\r\n";

#[cfg(not(target_os = "windows"))]
pub const NEWLINE: &str = "\n";
