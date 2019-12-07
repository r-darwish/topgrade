use super::terminal::*;
#[cfg(windows)]
use crate::error::Upgraded;
use anyhow::{bail, Result};
use self_update_crate;
use self_update_crate::backends::github::{GitHubUpdateStatus, Update};
use std::env;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn self_update() -> Result<()> {
    print_separator("Self update");
    let current_exe = env::current_exe();

    let target = self_update_crate::get_target();
    let result = Update::configure()
        .repo_owner("r-darwish")
        .repo_name("topgrade")
        .target(&target)
        .bin_name(if cfg!(windows) { "topgrade.exe" } else { "topgrade" })
        .show_output(false)
        .show_download_progress(true)
        .current_version(self_update_crate::cargo_crate_version!())
        .no_confirm(true)
        .build()
        .and_then(Update::update_extended)?;

    if let GitHubUpdateStatus::Updated(release) = &result {
        println!("\nTopgrade upgraded to {}:\n", release.version());
        println!("{}", release.body);
    } else {
        println!("Topgrade is up-to-date");
    }

    {
        if result.updated() {
            print_warning("Respawning...");
            let mut command = Command::new(current_exe?);
            command.args(env::args().skip(1)).env("TOPGRADE_NO_SELF_UPGRADE", "");

            #[cfg(unix)]
            {
                let err = command.exec();
                bail!(err);
            }

            #[cfg(windows)]
            {
                let status = command.spawn().and_then(|mut c| c.wait())?;
                bail!(Upgraded(status));
            }
        }
    }

    Ok(())
}
