use super::terminal::Terminal;
use super::utils::{self, Check};
use failure;
use std::process::Command;

#[must_use]
pub fn run_chocolatey(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(choco) = utils::which("choco") {
        terminal.print_separator("Chocolatey");

        let success = || -> Result<(), failure::Error> {
            Command::new(&choco).args(&["upgrade", "all"]).spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("Chocolatey", success));
    }

    None
}
