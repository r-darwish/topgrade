#[cfg(target_os = "linux")]
use crate::error::SkipStep;
use crate::error::TopgradeError;
use crate::execution_context::ExecutionContext;
use crate::executor::{ExecutorExitStatus, RunType};
use crate::terminal::{print_separator, print_warning};
use crate::utils::{require, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use log::debug;
use std::env;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Command;

pub fn run_fisher(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let fish = require("fish")?;
    base_dirs
        .home_dir()
        .join(".config/fish/functions/fisher.fish")
        .require()?;

    print_separator("Fisher");
    run_type
        .execute(&fish)
        .args(&["-c", "fisher self-update"])
        .check_run()?;

    run_type.execute(&fish).args(&["-c", "fisher"]).check_run()
}

pub fn run_brew(ctx: &ExecutionContext) -> Result<()> {
    let brew = require("brew")?;
    print_separator("Brew");
    let run_type = ctx.run_type();

    run_type.execute(&brew).arg("update").check_run()?;
    run_type
        .execute(&brew)
        .args(&["upgrade", "--ignore-pinned"])
        .check_run()
}

pub fn run_nix(ctx: &ExecutionContext) -> Result<()> {
    let nix = require("nix")?;
    let nix_channel = require("nix-channel")?;
    let nix_env = require("nix-env")?;
    print_separator("Nix");

    let multi_user = fs::metadata(&nix)?.uid() == 0;
    debug!("Multi user nix: {}", multi_user);

    #[cfg(target_os = "linux")]
    {
        use super::linux::Distribution;

        if let Ok(Distribution::NixOS) = Distribution::detect() {
            debug!("Nix on NixOS must be upgraded via 'nixos-rebuild switch', skipping.");
            return Err(SkipStep.into());
        }
    }

    let run_type = ctx.run_type();

    if multi_user {
        if let Some(sudo) = ctx.sudo() {
            run_type.execute(&sudo).arg("nix").arg("upgrade-nix").check_run()?;
        } else {
            print_warning("Need sudo to upgrade Nix");
        }
    } else {
        run_type.execute(&nix).arg("upgrade-nix").check_run()?;
    }
    run_type.execute(&nix_channel).arg("--update").check_run()?;
    run_type.execute(&nix_env).arg("--upgrade").check_run()
}

pub fn run_yadm(ctx: &ExecutionContext) -> Result<()> {
    let yadm = require("yadm")?;

    print_separator("yadm");

    ctx.run_type().execute(&yadm).arg("pull").check_run()
}

pub fn run_asdf(run_type: RunType) -> Result<()> {
    let asdf = require("asdf")?;

    print_separator("asdf");
    let exit_status = run_type.execute(&asdf).arg("update").spawn()?.wait()?;

    if let ExecutorExitStatus::Wet(e) = exit_status {
        if !(e.success() || e.code().map(|c| c == 42).unwrap_or(false)) {
            return Err(TopgradeError::ProcessFailed(e).into());
        }
    }
    run_type.execute(&asdf).args(&["plugin", "update", "--all"]).check_run()
}

pub fn run_home_manager(run_type: RunType) -> Result<()> {
    let home_manager = require("home-manager")?;

    print_separator("home-manager");
    run_type.execute(&home_manager).arg("switch").check_run()
}

pub fn run_tldr(run_type: RunType) -> Result<()> {
    let tldr = require("tldr")?;

    print_separator("TLDR");
    run_type.execute(&tldr).arg("--update").check_run()
}

pub fn run_pearl(run_type: RunType) -> Result<()> {
    let pearl = require("pearl")?;
    print_separator("pearl");

    run_type.execute(&pearl).arg("update").check_run()
}

pub fn run_sdkman(base_dirs: &BaseDirs, cleanup: bool, run_type: RunType) -> Result<()> {
    let bash = require("bash")?;

    let sdkman_init_path = env::var("SDKMAN_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".sdkman"))
        .join("bin")
        .join("sdkman-init.sh")
        .require()
        .map(|p| format!("{}", &p.display()))?;

    print_separator("SDKMAN!");

    let cmd_selfupdate = format!("source {} && sdk selfupdate", &sdkman_init_path);
    run_type
        .execute(&bash)
        .args(&["-c", cmd_selfupdate.as_str()])
        .check_run()?;

    let cmd_update = format!("source {} && sdk update", &sdkman_init_path);
    run_type.execute(&bash).args(&["-c", cmd_update.as_str()]).check_run()?;

    let cmd_upgrade = format!("source {} && sdk upgrade", &sdkman_init_path);
    run_type
        .execute(&bash)
        .args(&["-c", cmd_upgrade.as_str()])
        .check_run()?;

    if cleanup {
        let cmd_flush_archives = format!("source {} && sdk flush archives", &sdkman_init_path);
        run_type
            .execute(&bash)
            .args(&["-c", cmd_flush_archives.as_str()])
            .check_run()?;

        let cmd_flush_temp = format!("source {} && sdk flush temp", &sdkman_init_path);
        run_type
            .execute(&bash)
            .args(&["-c", cmd_flush_temp.as_str()])
            .check_run()?;
    }

    Ok(())
}

pub fn reboot() {
    print!("Rebooting...");
    Command::new("sudo").arg("reboot").spawn().unwrap().wait().unwrap();
}
