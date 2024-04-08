use clap::Parser;
use self_update::cargo_crate_version;
use self_update::errors::Error as su_Error;

#[derive(Parser, Debug)]
pub struct Update {
    /// Provide authentication token (GitHub PAT). Used for testing.
    #[arg(short, long, hide = true)]
    pub token: Option<String>,
}

impl Update {
    pub fn execute(&self) {
        cmd_update(&self.token).unwrap_or_else(|err| {
            eprintln!("{err}");
            if let su_Error::Network(_) = err {
                if err.to_string().contains("403") {
                    eprintln!("Most likely you are rate limited. Wait a while before trying again or use another network.")
                };
            };
            std::process::exit(1);
        });
    }
}

fn cmd_update(token: &Option<String>) -> Result<(), su_Error> {
    let mut binding = self_update::backends::github::Update::configure();
    let mut updater = binding
        .repo_owner("equinor")
        .repo_name("septic-config-generator")
        .bin_name("scg")
        .show_download_progress(true)
        .current_version(cargo_crate_version!());

    if let Some(token) = token {
        updater = updater.auth_token(token)
    }

    let status = updater.build()?.update()?;
    match status {
        self_update::Status::UpToDate(_) => println!("Already at latest version, nothing to do."),
        self_update::Status::Updated(version) => {
            println!("Successfully updated to v{:#}!", version)
        }
    }
    Ok(())
}
