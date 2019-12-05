use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
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
    run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()
}

pub fn zshrc(base_dirs: &BaseDirs) -> PathBuf {
    env::var("ZDOTDIR")
        .map(|p| Path::new(&p).join(".zshrc"))
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zshrc"))
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
    run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()
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
    run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()
}

pub fn run_oh_my_zsh(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;
    let zshrc = zshrc(base_dirs).require()?;
    base_dirs.home_dir().join(".oh-my-zsh").require()?;

    print_separator("oh-my-zsh");

    let cmd = format!("source {} && upgrade_oh_my_zsh", zshrc.display());
    run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()
}
