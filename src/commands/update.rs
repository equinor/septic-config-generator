use clap::Parser;
use self_update::cargo_crate_version;

#[derive(Parser, Debug)]
pub struct Update {
    /// Provide authentication token (GitHub PAT). Used for testing.
    #[arg(short, long, hide = true)]
    pub token: Option<String>,
}

impl Update {
    pub fn execute(&self) {
        cmd_update(&self.token);
    }
}

fn cmd_update(token: &Option<String>) {
    let binding = self_update::backends::github::Update::configure();
    let mut updater = binding;
    let mut updater = updater
        .repo_owner("equinor")
        .repo_name("septic-config-generator")
        .bin_name("scg")
        .show_download_progress(true)
        .current_version(cargo_crate_version!());

    if let Some(token) = token {
        updater = updater.auth_token(token)
    }

    let updater = updater.build().unwrap_or_else(|e| {
        eprintln!("Problem creating updater: {e}");
        std::process::exit(1);
    });

    updater.update().unwrap_or_else(|e| {
        eprintln!("{e}");
        std::process::exit(1);
    });
}
