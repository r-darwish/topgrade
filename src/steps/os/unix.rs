use crate::error::Error;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use directories::BaseDirs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

pub fn run_zplug(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let zsh = require("zsh")?;

    env::var("ZPLUG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join("zplug"))
        .require()?;

    let zshrc = env::var("ZDOTDIR")
        .map(|p| Path::new(&p).join(".zshrc"))
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zshrc"))
        .require()?;

    print_separator("zplug");

    let cmd = format!("source {} && zplug update", zshrc.display());
    run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()
}

pub fn run_fisher(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let fish = require("fish")?;
    base_dirs
        .home_dir()
        .join(".config/fish/functions/fisher.fish")
        .require()?;
    run_type
        .execute(&fish)
        .args(&["-c", "fisher self-update"])
        .check_run()?;
    run_type.execute(&fish).args(&["-c", "fisher"]).check_run()
}

#[must_use]
pub fn run_homebrew(cleanup: bool, run_type: RunType) -> Result<(), Error> {
    let brew = require("brew")?;
    print_separator("Brew");

    run_type.execute(&brew).arg("update").check_run()?;
    run_type.execute(&brew).arg("upgrade").check_run()?;

    let cask_upgrade_exists = Command::new(&brew)
        .args(&["--repository", "buo/cask-upgrade"])
        .check_output()
        .map(|p| Path::new(p.trim()).exists())?;

    if cask_upgrade_exists {
        run_type.execute(&brew).args(&["cu", "-ay"]).check_run()?;
    } else {
        run_type.execute(&brew).args(&["cask", "upgrade"]).check_run()?;
    }

    if cleanup {
        run_type.execute(&brew).arg("cleanup").check_run()?;
    }

    Ok(())
}

#[must_use]
pub fn run_nix(run_type: RunType) -> Result<(), Error> {
    let nix = require("nix")?;
    let nix_env = require("nix_env")?;

    print_separator("Nix");
    run_type.execute(&nix).arg("upgrade-nix").check_run()?;
    run_type.execute(&nix_env).arg("--upgrade").check_run()
}

pub fn run_pearl(run_type: RunType) -> Result<(), Error> {
    let pearl = require("pearl")?;
    print_separator("pearl");

    run_type.execute(&pearl).arg("update").check_run()
}
