use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::powershell;
use crate::terminal::print_separator;
use crate::utils::require;
use crate::{error::SkipStep, steps::git::Repositories};
use anyhow::Result;
use log::debug;
use std::convert::TryFrom;
use std::path::Path;
use std::{ffi::OsStr, process::Command};

pub fn run_chocolatey(ctx: &ExecutionContext) -> Result<()> {
    let choco = require("choco")?;
    let yes = ctx.config().yes();

    print_separator("Chocolatey");

    let mut cmd = &choco;
    let mut args = vec!["upgrade", "all"];

    if let Some(sudo) = ctx.sudo() {
        cmd = sudo;
        args.insert(0, "choco");
    }

    let mut command = ctx.run_type().execute(&cmd);

    command.args(&args);

    if yes {
        command.arg("--yes");
    }

    command.check_run()
}

pub fn run_winget(ctx: &ExecutionContext) -> Result<()> {
    let winget = require("winget")?;

    print_separator("winget");

    ctx.run_type().execute(&winget).args(&["upgrade", "--all"]).check_run()
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

pub fn run_wsl_topgrade(ctx: &ExecutionContext) -> Result<()> {
    let wsl = require("wsl")?;
    let topgrade = Command::new(&wsl)
        .args(&["bash", "-lc", "which topgrade"])
        .check_output()
        .map_err(|_| SkipStep(String::from("Could not find Topgrade installed in WSL")))?;

    let mut command = ctx.run_type().execute(&wsl);
    command
        .args(&["bash", "-c"])
        .arg(format!("TOPGRADE_PREFIX=WSL exec {}", topgrade));

    if ctx.config().yes() {
        command.arg("-y");
    }

    command.check_run()
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

pub fn insert_startup_scripts(ctx: &ExecutionContext, git_repos: &mut Repositories) -> Result<()> {
    let startup_dir = ctx
        .base_dirs()
        .data_dir()
        .join("Microsoft\\Windows\\Start Menu\\Programs\\Startup");
    for entry in std::fs::read_dir(&startup_dir)?.flatten() {
        let path = entry.path();
        if path.extension().and_then(OsStr::to_str) == Some("lnk") {
            if let Ok(lnk) = parselnk::Lnk::try_from(Path::new(&path)) {
                debug!("Startup link: {:?}", lnk);
                if let Some(path) = lnk.relative_path() {
                    git_repos.insert_if_repo(&startup_dir.join(path));
                }
            }
        }
    }

    Ok(())
}
