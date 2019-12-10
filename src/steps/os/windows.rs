use crate::error::SkipStep;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::require;
use anyhow::Result;
use std::process::Command;

pub fn run_chocolatey(run_type: RunType) -> Result<()> {
    let choco = require("choco")?;

    print_separator("Chocolatey");
    run_type.execute(&choco).args(&["upgrade", "all"]).check_run()
}

pub fn run_scoop(run_type: RunType) -> Result<()> {
    let scoop = require("scoop")?;

    print_separator("Scoop");

    run_type.execute(&scoop).args(&["update"]).check_run()?;
    run_type.execute(&scoop).args(&["update", "*"]).check_run()
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

pub fn reboot() {
    Command::new("shutdown").args(&["/R", "/T", "0"]).spawn().ok();
}
