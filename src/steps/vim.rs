use crate::error::{SkipStep, TopgradeError};
use anyhow::Result;

use crate::executor::{CommandExt, ExecutorOutput, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, require_option, PathExt};
use directories::BaseDirs;
use std::path::PathBuf;
use std::{
    io::{self, Write},
    process::Command,
};

const UPGRADE_VIM: &str = include_str!("upgrade.vim");

pub fn vimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    base_dirs
        .home_dir()
        .join(".vimrc")
        .if_exists()
        .or_else(|| base_dirs.home_dir().join(".vim/vimrc").if_exists())
}

fn nvimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    #[cfg(unix)]
    return base_dirs.home_dir().join(".config/nvim/init.vim").if_exists();

    #[cfg(windows)]
    return base_dirs.cache_dir().join("nvim/init.vim").if_exists();
}

fn upgrade(vim: &PathBuf, vimrc: &PathBuf, run_type: RunType) -> Result<()> {
    let mut tempfile = tempfile::NamedTempFile::new()?;
    tempfile.write_all(UPGRADE_VIM.as_bytes())?;

    let output = run_type
        .execute(&vim)
        .args(&["-u"])
        .arg(vimrc)
        .args(&["-U", "NONE", "-V1", "-nNesS"])
        .arg(tempfile.path())
        .output()?;

    if let ExecutorOutput::Wet(output) = output {
        let status = output.status;
        io::stdout().write(&output.stdout).ok();
        io::stderr().write(&output.stderr).ok();

        if !status.success() {
            return Err(TopgradeError::ProcessFailed(status).into());
        } else {
            println!("Plugins upgraded")
        }
    }

    Ok(())
}

pub fn upgrade_vim(base_dirs: &BaseDirs, run_type: RunType, _cleanup: bool) -> Result<()> {
    let vim = require("vim")?;

    let output = Command::new(&vim).arg("--version").check_output()?;
    if !output.starts_with("VIM") {
        return Err(SkipStep.into());
    }

    let vimrc = require_option(vimrc(&base_dirs))?;

    print_separator("Vim");
    upgrade(&vim, &vimrc, run_type)
}

pub fn upgrade_neovim(base_dirs: &BaseDirs, run_type: RunType, _cleanup: bool) -> Result<()> {
    let nvim = require("nvim")?;
    let nvimrc = require_option(nvimrc(&base_dirs))?;

    print_separator("Neovim");
    upgrade(&nvim, &nvimrc, run_type)
}

pub fn run_voom(_base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let voom = require("voom")?;

    print_separator("voom");

    run_type.execute(voom).arg("update").check_run()
}
