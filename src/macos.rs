use super::terminal::Terminal;
use super::utils::Check;
use failure;
use std::process::Command;

#[must_use]
pub fn upgrade_macos(terminal: &mut Terminal) -> Result<(), failure::Error> {
    terminal.print_separator("App Store");

    Command::new("softwareupdate")
        .args(&["--install", "--all"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}
