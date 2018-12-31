use super::error::{Error, ErrorKind};
use super::terminal::*;
use failure::ResultExt;
use self_update_crate;
use self_update_crate::backends::github::{GitHubUpdateStatus, Update};
#[cfg(unix)]
use std::env;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::process::Command;

pub fn self_update() -> Result<(), Error> {
    print_separator("Self update");
    #[cfg(unix)]
    let current_exe = env::current_exe();

    let target = self_update_crate::get_target().context(ErrorKind::SelfUpdate)?;
    let result = Update::configure()
        .and_then(|mut u| {
            u.repo_owner("r-darwish")
                .repo_name("topgrade")
                .target(&target)
                .bin_name(if cfg!(windows) { "topgrade.exe" } else { "topgrade" })
                .show_output(false)
                .show_download_progress(true)
                .current_version(self_update_crate::cargo_crate_version!())
                .no_confirm(true)
                .build()
        })
        .and_then(|u| u.update2())
        .context(ErrorKind::SelfUpdate)?;

    if let GitHubUpdateStatus::Updated(release) = &result {
        println!("\nTopgrade upgraded to {}:\n", release.version());
        println!("{}", release.body);
    } else {
        println!("Topgrade is up-to-date");
    }

    #[cfg(unix)]
    {
        if result.updated() {
            print_warning("Respawning...");
            let err = Command::new(current_exe.context(ErrorKind::SelfUpdate)?)
                .args(env::args().skip(1))
                .env("TOPGRADE_NO_SELF_UPGRADE", "")
                .exec();
            Err(err).context(ErrorKind::SelfUpdate)?
        }
    }

    Ok(())
}
