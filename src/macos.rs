use super::utils::Check;
use failure;
use std::process::Command;

pub fn upgrade_macos() -> Result<(), failure::Error> {
    Command::new("softwareupdate")
        .args(&["--install", "--all"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}
