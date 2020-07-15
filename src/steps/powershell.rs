use crate::execution_context::ExecutionContext;
use crate::executor::CommandExt;
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

    pub fn update_modules(&self, ctx: &ExecutionContext) -> Result<()> {
        let powershell = require_option(self.path.as_ref())?;

        print_separator("Powershell Modules Update");

        let cmd = if ctx.config().yes() {
            "Update-Module -AcceptLicense -Force"
        } else {
            "Update-Module"
        };

        println!("Updating modules...");
        ctx.run_type().execute(&powershell).args(&["-Command", cmd]).check_run()
    }

    #[cfg(windows)]
    pub fn supports_windows_update(&self) -> bool {
        self.path
            .as_ref()
            .map(|p| Self::has_module(&p, "PSWindowsUpdate"))
            .unwrap_or(false)
    }

    #[cfg(windows)]
    pub fn windows_update(&self, ctx: &ExecutionContext) -> Result<()> {
        let powershell = require_option(self.path.as_ref())?;

        debug_assert!(self.supports_windows_update());

        ctx.run_type()
            .execute(&powershell)
            .args(&[
                "-Command",
                &format!(
                    "Import-Module PSWindowsUpdate; Install-WindowsUpdate -MicrosoftUpdate {} -Verbose",
                    if ctx.config().accept_all_windows_updates() {
                        "-AcceptAll"
                    } else {
                        ""
                    }
                ),
            ])
            .check_run()
    }
}
