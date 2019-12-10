#[cfg(windows)]
use crate::error::SkipStep;
use crate::executor::{CommandExt, RunType};
use crate::terminal::{is_dumb, print_separator};
use crate::utils::{require_option, which, PathExt};
use anyhow::Result;
use std::path::PathBuf;
use std::process::Command;

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
        let path = which("pwsh").or_else(|| which("powershell")).filter(|_| !is_dumb());

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

    #[cfg(windows)]
    pub fn windows_powershell() -> Self {
        Powershell {
            path: which("powershell").filter(|_| !is_dumb()),
            profile: None,
        }
    }

    #[cfg(windows)]
    pub fn has_module(powershell: &PathBuf, command: &str) -> bool {
        Command::new(&powershell)
            .args(&["-Command", &format!("Get-Module -ListAvailable {}", command)])
            .check_output()
            .map(|result| !result.is_empty())
            .unwrap_or(false)
    }

    pub fn profile(&self) -> Option<&PathBuf> {
        self.profile.as_ref()
    }

    pub fn update_modules(&self, run_type: RunType) -> Result<()> {
        let powershell = require_option(self.path.as_ref())?;

        print_separator("Powershell Modules Update");
        println!("Updating modules...");
        run_type
            .execute(&powershell)
            .args(&["-Command", "Update-Module"])
            .check_run()
    }

    #[cfg(windows)]
    pub fn windows_update(&self, run_type: RunType) -> Result<()> {
        let powershell = require_option(self.path.as_ref())?;

        if !Self::has_module(&powershell, "PSWindowsUpdate") {
            return Err(SkipStep.into());
        }
        print_separator("Windows Update");

        run_type
            .execute(&powershell)
            .args(&[
                "-Command",
                "Import-Module PSWindowsUpdate; Install-WindowsUpdate -MicrosoftUpdate -AcceptAll -Verbose",
            ])
            .check_run()
    }
}
