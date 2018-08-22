use super::terminal::Terminal;
use super::utils::Check;
use failure;
use std::process::Command;

#[must_use]
pub fn upgrade_macos(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    terminal.print_separator("App Store");

    let success = || -> Result<(), failure::Error> {
        Command::new("softwareupdate")
            .args(&["--install", "--all"])
            .spawn()?
            .wait()?
            .check()?;
        Ok(())
    }().is_ok();

    Some(("App Store", success))
}
