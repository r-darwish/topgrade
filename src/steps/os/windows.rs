use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{self, which, Check};
use failure::ResultExt;
use log::error;
use std::path::PathBuf;
use std::process::Command;

#[must_use]
pub fn run_chocolatey(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(choco) = utils::which("choco") {
        print_separator("Chocolatey");

        let success = || -> Result<(), Error> {
            run_type
                .execute(&choco)
                .args(&["upgrade", "all"])
                .spawn()?
                .wait()?
                .check()?;
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
            run_type.execute(&scoop).args(&["update"]).spawn()?.wait()?.check()?;
            run_type
                .execute(&scoop)
                .args(&["update", "*"])
                .spawn()?
                .wait()?
                .check()?;
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
                .output()
                .context(ErrorKind::ProcessExecution)?
                .check()?;
            Ok(())
        }()
        .is_ok()
    }

    pub fn profile(&self) -> Option<PathBuf> {
        if let Some(powershell) = &self.path {
            let result = || -> Result<PathBuf, Error> {
                let output = Command::new(powershell)
                    .args(&["-Command", "echo $profile"])
                    .output()
                    .context(ErrorKind::ProcessExecution)?;
                output.status.check()?;
                Ok(PathBuf::from(
                    String::from_utf8_lossy(&output.stdout).trim().to_string(),
                ))
            }();

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
                run_type
                    .execute(&powershell)
                    .arg("Update-Module")
                    .spawn()?
                    .wait()?
                    .check()?;
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
                        .spawn()?
                        .wait()?
                        .check()?;
                    Ok(())
                }()
                .is_ok();

                return Some(("Windows Update", success));
            }
        }

        None
    }
}
