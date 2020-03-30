use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::git::Repositories;
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use log::debug;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
    zshrc(base_dirs).require()?;

    env::var("ZPLUG_HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join("zplug"))
        .require()?;

    print_separator("zplug");

    run_type.execute(zsh).args(&["-i", "-c", "zplug update"]).check_run()
}

pub fn run_zinit(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let zsh = require("zsh")?;
    let zshrc = zshrc(base_dirs).require()?;

    env::var("ZPFX")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zinit"))
        .require()?;

    print_separator("zinit");

    let cmd = format!("source {} && zinit self-update && zinit update --all", zshrc.display(),);
    run_type.execute(zsh).args(&["-i", "-c", cmd.as_str()]).check_run()
}

pub fn run_oh_my_zsh(ctx: &ExecutionContext) -> Result<()> {
    require("zsh")?;
    let oh_my_zsh = ctx.base_dirs().home_dir().join(".oh-my-zsh").require()?;

    print_separator("oh-my-zsh");

    let mut custom_dir = PathBuf::from(
        Command::new("zsh")
            .args(&["-i", "-c", "echo -n $ZSH_CUSTOM"])
            .check_output()?,
    );
    custom_dir.push("plugins");

    debug!("oh-my-zsh custom dir: {}", custom_dir.display());

    if let Ok(custom_plugins_dir) = fs::read_dir(custom_dir) {
        let mut custom_plugins = Repositories::new(ctx.git());

        for entry in custom_plugins_dir? {
            let entry = entry?;
            custom_plugins.insert_if_repo(entry.path());
        }
        custom_plugins.remove(&oh_my_zsh.to_string_lossy());
        if !custom_plugins.is_empty() {
            println!("Pulling custom plugins");
            ctx.git().multi_pull(&custom_plugins, ctx)?;
        }
    }

    ctx.run_type()
        .execute("sh")
        .env("ZSH", &oh_my_zsh)
        .arg(&oh_my_zsh.join("tools/upgrade.sh"))
        .check_run()
}
