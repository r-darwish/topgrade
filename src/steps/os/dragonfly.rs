use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::require_option;
use snafu::ResultExt;
use std::path::PathBuf;
use std::process::Command;

pub fn upgrade_packages(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    print_separator("DrgaonFly BSD Packages");
    run_type
        .execute(sudo)
        .args(&["/usr/local/sbin/pkg", "upgrade"])
        .check_run()
}

pub fn audit_packages(sudo: &Option<PathBuf>) -> Result<(), Error> {
    if let Some(sudo) = sudo {
        println!();
        Command::new(sudo)
            .args(&["/usr/local/sbin/pkg", "audit", "-Fr"])
            .spawn()
            .context(Error::ProcessExecution)?
            .wait()
            .context(Error::ProcessExecution)?;
    }
    Ok(())
}
