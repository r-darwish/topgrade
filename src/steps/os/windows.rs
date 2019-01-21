use crate::error::Error;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{self, which};
use log::error;
use std::path::PathBuf;
use std::process::Command;

#[must_use]
pub fn run_chocolatey(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(choco) = utils::which("choco") {
        print_separator("Chocolatey");

        let success = || -> Result<(), Error> {
            run_type.execute(&choco).args(&["upgrade", "all"]).check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Chocolatey", success));
    }

    None
}

#[must_use]
pub fn run_scoop(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(scoop) = utils::which("scoop") {
        print_separator("Scoop");

        let success = || -> Result<(), Error> {
            run_type.execute(&scoop).args(&["update"]).check_run()?;
            run_type.execute(&scoop).args(&["update", "*"]).check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Scoop", success));
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
        || -> Result<(), Error> {
            Command::new(&powershell)
                .args(&["-Command", &format!("Get-Command {}", command)])
                .check_output()?;
            Ok(())
        }()
        .is_ok()
    }

    pub fn profile(&self) -> Option<PathBuf> {
        if let Some(powershell) = &self.path {
            let result = Command::new(powershell)
                .args(&["-Command", "echo $profile"])
                .check_output()
                .map(|output| PathBuf::from(output.trim()));

            match result {
                Err(e) => error!("Error getting Powershell profile: {}", e),
                Ok(path) => return Some(path),
            }
        }
        None
    }

    #[must_use]
    pub fn update_modules(&self, run_type: RunType) -> Option<(&'static str, bool)> {
        if let Some(powershell) = &self.path {
            print_separator("Powershell Modules Update");

            let success = || -> Result<(), Error> {
                run_type.execute(&powershell).arg("Update-Module").check_run()?;
                Ok(())
            }()
            .is_ok();

            return Some(("Powershell Modules Update", success));
        }

        None
    }

    #[must_use]
    pub fn windows_update(&self, run_type: RunType) -> Option<(&'static str, bool)> {
        if let Some(powershell) = &self.path {
            if Self::has_command(&powershell, "Install-WindowsUpdate") {
                print_separator("Windows Update");

                let success = || -> Result<(), Error> {
                    run_type
                        .execute(&powershell)
                        .args(&["-Command", "Install-WindowsUpdate -MicrosoftUpdate -AcceptAll -Verbose"])
                        .check_run()?;
                    Ok(())
                }()
                .is_ok();

                return Some(("Windows Update", success));
            }
        }

        None
    }
}
