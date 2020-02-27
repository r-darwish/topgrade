use crate::error::SkipStep;
use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::powershell;
use crate::terminal::print_separator;
use crate::utils::require;
use anyhow::Result;
use std::process::Command;

pub fn run_chocolatey(run_type: RunType) -> Result<()> {
    let choco = require("choco")?;

    print_separator("Chocolatey");
    run_type.execute(&choco).args(&["upgrade", "all"]).check_run()
}

pub fn run_scoop(cleanup: bool, run_type: RunType) -> Result<()> {
    let scoop = require("scoop")?;

    print_separator("Scoop");

    run_type.execute(&scoop).args(&["update"]).check_run()?;
    run_type.execute(&scoop).args(&["update", "*"]).check_run()?;

    if cleanup {
        run_type.execute(&scoop).args(&["cleanup", "*"]).check_run()?;
    }

    Ok(())
}

pub fn run_wsl_topgrade(run_type: RunType) -> Result<()> {
    let wsl = require("wsl")?;
    let topgrade = Command::new(&wsl)
        .args(&["bash", "-l", "which", "topgrade"])
        .check_output()
        .map_err(|_| SkipStep)?;

    run_type
        .execute(&wsl)
        .args(&["bash", "-c"])
        .arg(format!("TOPGRADE_PREFIX=WSL exec {}", topgrade))
        .check_run()
}

pub fn windows_update(ctx: &ExecutionContext) -> Result<()> {
    let powershell = powershell::Powershell::windows_powershell();

    if powershell.supports_windows_update() {
        print_separator("Windows Update");
        return powershell.windows_update(ctx);
    }

    let usoclient = require("UsoClient")?;

    print_separator("Windows Update");
    println!("Running Windows Update. Check the control panel for progress.");
    ctx.run_type().execute(&usoclient).arg("ScanInstallWait").check_run()?;
    ctx.run_type().execute(&usoclient).arg("StartInstall").check_run()
}

pub fn reboot() {
    Command::new("shutdown").args(&["/R", "/T", "0"]).spawn().ok();
}
