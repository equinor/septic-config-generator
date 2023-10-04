use std::process;

use clap::Parser;
use self_update::cargo_crate_version;

#[derive(Parser, Debug)]
pub struct Update;

impl Update {
    pub fn execute(&self) {
        cmd_update();
    }
}

fn cmd_update() {
    let updater = self_update::backends::github::Update::configure()
        .repo_owner("equinor")
        .repo_name("septic-config-generator")
        .bin_name("scg")
        .show_download_progress(true)
        .current_version(cargo_crate_version!())
        .build()
        .unwrap_or_else(|e| {
            eprintln!("Problem creating updater: {e}");
            process::exit(1);
        });

    let status = updater.update().unwrap_or_else(|e| {
        eprintln!("{e}");
        eprintln!("You may be rate limited. Please try again later.");
        process::exit(1);
    });

    println!("Update status: `{}`!", status.version());

    // let status = self_update::backends::github::Update::configure()
    //     .repo_owner("equinor")
    //     .repo_name("septic-config-generator")
    //     .bin_name("github")
    //     .target("windows") // Do I need to specify target?
    //     .bin_path_in_archive("scg.exe") // Handle Windows differently from Linux/MacOS
    //     .show_download_progress(true)
    //     .current_version(cargo_crate_version!())
    //     .build()
    //     .unwrap()
    //     .update()
    //     .unwrap();
    // println!("Update status: `{}`!", status.version());
}
