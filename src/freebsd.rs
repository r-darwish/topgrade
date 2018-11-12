use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::Check;
use failure;
use std::path::PathBuf;

#[must_use]
pub fn upgrade_freebsd(sudo: &Option<PathBuf>, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    terminal.print_separator("FreeBSD Update");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), failure::Error> {
            Executor::new(sudo, dry_run)
                .args(&["/usr/sbin/freebsd-update", "fetch", "install"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }().is_ok();

        Some(("FreeBSD Update", success))
    } else {
        terminal.print_warning("No sudo or yay detected. Skipping system upgrade");
        None
    }
}

#[must_use]
pub fn upgrade_packages(
    sudo: &Option<PathBuf>,
    terminal: &mut Terminal,
    dry_run: bool,
) -> Option<(&'static str, bool)> {
    terminal.print_separator("FreeBSD Packages");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), failure::Error> {
            Executor::new(sudo, dry_run)
                .args(&["/usr/sbin/pkg", "upgrade"])
                .spawn()?
                .wait()?
                .check()?;
            Executor::new("/usr/sbin/pkg", dry_run)
                .arg("audit")
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }().is_ok();

        Some(("FreeBSD Packages", success))
    } else {
        terminal.print_warning("No sudo or yay detected. Skipping package upgrade");
        None
    }
}
