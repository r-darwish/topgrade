#[cfg(target_os = "linux")]
use crate::error::SkipStep;
use crate::error::TopgradeError;
use crate::execution_context::ExecutionContext;
#[cfg(target_os = "macos")]
use crate::executor::CommandExt;
use crate::executor::{Executor, ExecutorExitStatus, RunType};
use crate::terminal::{print_separator, print_warning};
use crate::utils::{require, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use log::debug;
use std::fs;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;
use std::process::Command;
use std::{env, path::Path};

const INTEL_BREW: &str = "/usr/local/bin/brew";
const ARM_BREW: &str = "/opt/homebrew/bin/brew";
#[derive(Copy, Clone, Debug)]
#[allow(dead_code)]
pub enum BrewVariant {
    Linux,
    MacIntel,
    MacArm,
}

impl BrewVariant {
    fn binary_name(self) -> &'static str {
        match self {
            BrewVariant::Linux => "/home/linuxbrew/.linuxbrew/bin/brew",
            BrewVariant::MacIntel => INTEL_BREW,
            BrewVariant::MacArm => ARM_BREW,
        }
    }

    fn both_both_exist() -> bool {
        Path::new(INTEL_BREW).exists() && Path::new(ARM_BREW).exists()
    }

    pub fn step_title(self) -> &'static str {
        let both_exists = Self::both_both_exist();
        match self {
            BrewVariant::MacArm if both_exists => "Brew (ARM)",
            BrewVariant::MacIntel if both_exists => "Brew (Intel)",
            _ => "Brew",
        }
    }

    fn execute(self, run_type: RunType) -> Executor {
        match self {
            BrewVariant::MacIntel if cfg!(target_arch = "aarch64") => {
                let mut command = run_type.execute("arch");
                command.arg("-x86_64").arg(self.binary_name());
                command
            }
            BrewVariant::MacArm if cfg!(target_arch = "x86_64") => {
                let mut command = run_type.execute("arch");
                command.arg("-arm64e").arg(self.binary_name());
                command
            }
            _ => run_type.execute(self.binary_name()),
        }
    }
}

pub fn run_fisher(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let fish = require("fish")?;
    base_dirs
        .home_dir()
        .join(".config/fish/functions/fisher.fish")
        .require()?;

    print_separator("Fisher");

    run_type.execute(&fish).args(&["-c", "fisher update"]).check_run()
}

pub fn run_oh_my_fish(ctx: &ExecutionContext) -> Result<()> {
    let fish = require("fish")?;
    ctx.base_dirs()
        .home_dir()
        .join(".local/share/omf/pkg/omf/functions/omf.fish")
        .require()?;

    print_separator("oh-my-fish");

    ctx.run_type().execute(&fish).args(&["-c", "omf update"]).check_run()
}

pub fn run_brew_formula(ctx: &ExecutionContext, variant: BrewVariant) -> Result<()> {
    require(variant.binary_name())?;
    print_separator(variant.step_title());
    let run_type = ctx.run_type();

    variant.execute(run_type).arg("update").check_run()?;
    variant
        .execute(run_type)
        .args(&["upgrade", "--ignore-pinned", "--formula"])
        .check_run()?;

    if ctx.config().cleanup() {
        variant.execute(run_type).arg("cleanup").check_run()?;
    }

    Ok(())
}

#[cfg(target_os = "macos")]
pub fn run_brew_cask(ctx: &ExecutionContext, variant: BrewVariant) -> Result<()> {
    require(variant.binary_name())?;
    print_separator(format!("{} - Cask", variant.step_title()));
    let run_type = ctx.run_type();

    let cask_upgrade_exists = variant
        .execute(RunType::Wet)
        .args(&["--repository", "buo/cask-upgrade"])
        .check_output()
        .map(|p| Path::new(p.trim()).exists())?;

    let mut brew_args = vec![];

    if cask_upgrade_exists {
        brew_args.extend(&["cu", "-y"]);
        if ctx.config().brew_cask_greedy() {
            brew_args.push("-a");
        }
    } else {
        brew_args.extend(&["upgrade", "--cask"]);
        if ctx.config().brew_cask_greedy() {
            brew_args.push("--greedy");
        }
    }

    variant.execute(run_type).args(&brew_args).check_run()?;

    if ctx.config().cleanup() {
        variant.execute(run_type).arg("cleanup").check_run()?;
    }

    Ok(())
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
            return Err(SkipStep(String::from("Nix on NixOS must be upgraded via nixos-rebuild switch")).into());
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
