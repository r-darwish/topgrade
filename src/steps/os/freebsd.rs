use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::{print_separator, print_warning};
use crate::utils::Check;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

#[must_use]
pub fn upgrade_freebsd(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    print_separator("FreeBSD Update");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), Error> {
            run_type
                .execute(sudo)
                .args(&["/usr/sbin/freebsd-update", "fetch", "install"])
                .check_run()?;
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
pub fn upgrade_packages(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    print_separator("FreeBSD Packages");

    if let Some(sudo) = sudo {
        let success = || -> Result<(), Error> {
            run_type.execute(sudo).args(&["/usr/sbin/pkg", "upgrade"]).check_run()?;
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
