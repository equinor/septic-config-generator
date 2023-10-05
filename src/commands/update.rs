use clap::Parser;
use self_update::cargo_crate_version;

#[derive(Parser, Debug)]
pub struct Update;

impl Update {
    pub fn execute(&self) {
        cmd_update();
    }
}

fn get_target() -> Result<String, String> {
    let target = self_update::get_target();
    let supported_targets = ["windows", "darwin", "linux"];

    for supported_target in &supported_targets {
        if target.contains(supported_target) {
            return Ok(String::from(*supported_target));
        }
    }

    Err(format!("Unsupported target: {}", target))
}

fn cmd_update() {
    let target = get_target().unwrap_or_else(|e| {
        eprintln!("{}", e);
        std::process::exit(1)
    });

    self_update::backends::github::Update::configure()
        .repo_owner("equinor")
        .repo_name("septic-config-generator")
        .bin_name("scg")
        .show_download_progress(true)
        .target(&target)
        .current_version(cargo_crate_version!())
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Problem creating updater: {e}");
            std::process::exit(1);
        })
        .update()
        .unwrap_or_else(|e| {
            eprintln!("{e}");
            std::process::exit(1);
        });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_get_target_returns_value() {
        let target = get_target().unwrap();
        if cfg!(windows) {
            assert_eq!(target, "windows");
        } else if cfg!(target_os = "macos") {
            assert_eq!(target, "darwin");
        } else if cfg!(target_os = "linux") {
            assert_eq!(target, "linux");
        }
    }
}
