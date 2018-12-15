use crate::error::{Error, ErrorKind};
use crate::executor::Executor;
use crate::terminal::{print_separator, print_warning};
use crate::utils::Check;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

#[must_use]
pub fn upgrade_freebsd(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    print_separator("FreeBSD Update");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), Error> {
            Executor::new(sudo, dry_run)
                .args(&["/usr/sbin/freebsd-update", "fetch", "install"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        Some(("FreeBSD Update", success))
    } else {
        print_warning("No sudo or yay detected. Skipping system upgrade");
        None
    }
}

#[must_use]
pub fn upgrade_packages(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    print_separator("FreeBSD Packages");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), Error> {
            Executor::new(sudo, dry_run)
                .args(&["/usr/sbin/pkg", "upgrade"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        Some(("FreeBSD Packages", success))
    } else {
        print_warning("No sudo or yay detected. Skipping package upgrade");
        None
    }
}

pub fn audit_packages(sudo: &Option<PathBuf>) -> Result<(), Error> {
    if let Some(sudo) = sudo {
        println!();
        Command::new(sudo)
            .args(&["/usr/sbin/pkg", "audit", "-Fr"])
            .spawn()
            .context(ErrorKind::ProcessExecution)?
            .wait()
            .context(ErrorKind::ProcessExecution)?;
    }
    Ok(())
}
