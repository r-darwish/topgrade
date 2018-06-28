use super::utils::Check;
use failure;
use std::path::PathBuf;
use std::process::Command;

pub fn run_chocolatey(choco: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&choco)
        .args(&["upgrade", "all"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}
