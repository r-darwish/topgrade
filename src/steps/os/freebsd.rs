use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::{print_separator, print_warning};
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

pub fn upgrade_freebsd(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    print_separator("FreeBSD Update");
    run_type
        .execute(sudo)
        .args(&["/usr/sbin/freebsd-update", "fetch", "install"])
        .check_run()
}

pub fn upgrade_packages(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    print_separator("FreeBSD Packages");
    run_type.execute(sudo).args(&["/usr/sbin/pkg", "upgrade"]).check_run()
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
