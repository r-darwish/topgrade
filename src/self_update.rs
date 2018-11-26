use super::terminal::Terminal;
use failure::Error;
use self_update_crate;
use std::env;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
#[cfg(unix)]
use std::process::Command;

pub fn self_update(terminal: &mut Terminal) -> Result<(), Error> {
    terminal.print_separator("Self update");
    #[cfg(unix)]
    let current_exe = env::current_exe();

    let target = self_update_crate::get_target()?;
    let result = self_update_crate::backends::github::Update::configure()?
        .repo_owner("r-darwish")
        .repo_name("topgrade")
        .target(&target)
        .bin_name(if cfg!(windows) { "topgrade.exe" } else { "topgrade" })
        .show_output(false)
        .show_download_progress(true)
        .current_version(self_update_crate::cargo_crate_version!())
        .no_confirm(true)
        .build()?
        .update()?;

    if let self_update_crate::Status::Updated(version) = &result {
        println!("\nTopgrade upgraded to {}", version);
    } else {
        println!("Topgrade is up-to-date");
    }

    #[cfg(unix)]
    {
        if result.updated() {
            terminal.print_warning("Respawning...");
            let err = Command::new(current_exe?).args(env::args().skip(1)).exec();
            Err(err)?
        }
    }

    Ok(())
}
