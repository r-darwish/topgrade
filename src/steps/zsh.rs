use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use std::env;
use std::path::{Path, PathBuf};

pub fn run_zr(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;

    env::var("ZR_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zr"))
        .require()?;

    print_separator("zr");

    let cmd = format!("source {} && zr update", zshrc(base_dirs).display());
    run_type.execute(zsh).args(&["-l", "-c", cmd.as_str()]).check_run()
}

pub fn zshrc(base_dirs: &BaseDirs) -> PathBuf {
    env::var("ZDOTDIR")
        .map(|p| Path::new(&p).join(".zshrc"))
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zshrc"))
}

pub fn run_antibody(run_type: RunType) -> Result<()> {
    require("zsh")?;
    let antibody = require("antibody")?;

    print_separator("antibody");

    run_type.execute(antibody).arg("update").check_run()
}

pub fn run_antigen(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;
    let zshrc = zshrc(base_dirs).require()?;
    env::var("ADOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join("antigen.zsh"))
        .require()?;

    print_separator("antigen");

    let cmd = format!("source {} && antigen selfupdate && antigen update", zshrc.display());
    run_type.execute(zsh).args(&["-l", "-c", cmd.as_str()]).check_run()
}

pub fn run_zplug(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;
    let zshrc = zshrc(base_dirs).require()?;

    env::var("ZPLUG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join("zplug"))
        .require()?;

    print_separator("zplug");

    let cmd = format!("source {} && zplug update", zshrc.display());
    run_type.execute(zsh).args(&["-l", "-c", cmd.as_str()]).check_run()
}

pub fn run_zplugin(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;
    let zshrc = zshrc(base_dirs).require()?;

    env::var("ZPLGM[HOME_DIR]")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zplugin"))
        .require()?;

    print_separator("zplugin");

    let cmd = format!(
        "source {} && zplugin self-update && zplugin update --all",
        zshrc.display()
    );
    run_type.execute(zsh).args(&["-l", "-c", cmd.as_str()]).check_run()
}

pub fn run_oh_my_zsh(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    require("zsh")?;
    let oh_my_zsh = base_dirs.home_dir().join(".oh-my-zsh").require()?;

    print_separator("oh-my-zsh");

    run_type
        .execute("sh")
        .env("ZSH", &oh_my_zsh)
        .arg(&oh_my_zsh.join("tools/upgrade.sh"))
        .check_run()
}
