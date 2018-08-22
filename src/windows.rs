use super::terminal::Terminal;
use super::utils::{self, which, Check};
use failure;
use std::path::PathBuf;
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

pub struct Powershell {
    path: Option<PathBuf>,
}

impl Powershell {
    pub fn new() -> Self {
        Powershell {
            path: which("powershell"),
        }
    }

    pub fn has_command(powershell: &PathBuf, command: &str) -> bool {
        || -> Result<(), failure::Error> {
            Command::new(&powershell)
                .args(&["-Command", &format!("Get-Command {}", command)])
                .output()?
                .check()?;
            Ok(())
        }().is_ok()
    }

    #[must_use]
    pub fn update_modules(&self, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
        if let Some(powershell) = &self.path {
            terminal.print_separator("Powershell Modules Update");

            let success = || -> Result<(), failure::Error> {
                Command::new(&powershell).arg("Update-Module").spawn()?.wait()?.check()?;
                Ok(())
            }().is_ok();

            return Some(("Powershell Modules Update", success));
        }

        None
    }

    #[must_use]
    pub fn windows_update(&self, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
        if let Some(powershell) = &self.path {
            if Self::has_command(&powershell, "Install-WindowsUpdate") {
                terminal.print_separator("Windows Update");

                let success = || -> Result<(), failure::Error> {
                    Command::new(&powershell)
                        .args(&["-Command", "Install-WindowsUpdate -MicrosoftUpdate -AcceptAll -Verbose"])
                        .spawn()?
                        .wait()?
                        .check()?;
                    Ok(())
                }().is_ok();

                return Some(("Windows Update", success));
            }
        }

        None
    }
}
