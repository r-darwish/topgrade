use crate::error::{Error, ErrorKind};
use crate::executor::{CommandExt, RunType};
use crate::terminal::{is_dumb, print_separator};
use crate::utils::{require, require_option, which, PathExt};
use std::path::PathBuf;
use std::process::Command;

pub fn run_chocolatey(run_type: RunType) -> Result<(), Error> {
    let choco = require("choco")?;

    print_separator("Chocolatey");
    run_type.execute(&choco).args(&["upgrade", "all"]).check_run()
}

pub fn run_scoop(run_type: RunType) -> Result<(), Error> {
    let scoop = require("scoop")?;

    print_separator("Scoop");

    run_type.execute(&scoop).args(&["update"]).check_run()?;
    run_type.execute(&scoop).args(&["update", "*"]).check_run()
}

pub struct Powershell {
    path: Option<PathBuf>,
    profile: Option<PathBuf>,
}

impl Powershell {
    /// Returns a powershell instance.
    ///
    /// If the powershell binary is not found, or the current terminal is dumb
    /// then the instance of this struct will skip all the powershell steps.
    pub fn new() -> Self {
        let path = which("powershell").filter(|_| !is_dumb());

        let profile = path.as_ref().and_then(|path| {
            Command::new(path)
                .args(&["-Command", "echo $profile"])
                .check_output()
                .map(|output| PathBuf::from(output.trim()))
                .and_then(|p| p.require())
                .ok()
        });

        Powershell { path, profile }
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

    pub fn profile(&self) -> Option<&PathBuf> {
        self.profile.as_ref()
    }

    pub fn update_modules(&self, run_type: RunType) -> Result<(), Error> {
        let powershell = require_option(self.path.as_ref())?;

        print_separator("Powershell Modules Update");
        run_type.execute(&powershell).args(&["Update-Module", "-v"]).check_run()
    }

    pub fn windows_update(&self, run_type: RunType) -> Result<(), Error> {
        let powershell = require_option(self.path.as_ref())?;

        if !Self::has_command(&powershell, "Install-WindowsUpdate") {
            Err(ErrorKind::SkipStep)?;
        }
        print_separator("Windows Update");

        run_type
            .execute(&powershell)
            .args(&["-Command", "Install-WindowsUpdate -MicrosoftUpdate -AcceptAll -Verbose"])
            .check_run()
    }
}

pub fn run_wsl_topgrade(run_type: RunType) -> Result<(), Error> {
    let wsl = require("wsl")?;
    let topgrade = Command::new(&wsl)
        .args(&["bash", "-l", "which", "topgrade"])
        .check_output()
        .map_err(|_| ErrorKind::SkipStep)?;

    run_type
        .execute(&wsl)
        .args(&["bash", "-c"])
        .arg(format!("TOPGRADE_PREFIX=WSL exec {}", topgrade))
        .check_run()
}
