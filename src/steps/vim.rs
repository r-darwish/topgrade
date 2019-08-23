use crate::error::Error;
use crate::executor::{CommandExt, ExecutorOutput, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, require_option, PathExt};
use directories::BaseDirs;
use std::path::PathBuf;
use std::{
    fs,
    io::{self, Write},
    process::Command,
};

#[derive(Debug, Clone, Copy)]
pub enum PluginFramework {
    Plug,
    Vundle,
    NeoBundle,
    Dein,
}

impl PluginFramework {
    pub fn detect(vimrc: &PathBuf) -> Option<PluginFramework> {
        let content = fs::read_to_string(vimrc).ok()?;

        if content.contains("NeoBundle") {
            Some(PluginFramework::NeoBundle)
        } else if content.contains("Vundle") {
            Some(PluginFramework::Vundle)
        } else if content.contains("plug#begin") {
            Some(PluginFramework::Plug)
        } else if content.contains("dein#begin") {
            Some(PluginFramework::Dein)
        } else {
            None
        }
    }

    pub fn upgrade_command(self) -> &'static str {
        match self {
            PluginFramework::NeoBundle => "NeoBundleUpdate",
            PluginFramework::Vundle => "PluginUpdate",
            PluginFramework::Plug => "PlugUpgrade | PlugUpdate",
            PluginFramework::Dein => "call dein#install() | call dein#update()",
        }
    }
}

pub fn vimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    base_dirs
        .home_dir()
        .join(".vimrc")
        .if_exists()
        .or_else(|| base_dirs.home_dir().join(".vim/vimrc").if_exists())
}

fn nvimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    #[cfg(unix)]
    return base_dirs.config_dir().join("nvim/init.vim").if_exists();

    #[cfg(windows)]
    return base_dirs.cache_dir().join("nvim/init.vim").if_exists();
}

#[must_use]
fn upgrade(vim: &PathBuf, vimrc: &PathBuf, plugin_framework: PluginFramework, run_type: RunType) -> Result<(), Error> {
    let output = run_type
        .execute(&vim)
        .args(&["-N", "-u"])
        .arg(vimrc)
        .args(&[
            "-c",
            plugin_framework.upgrade_command(),
            "-c",
            "quitall",
            "-e",
            "-s",
            "-V1",
        ])
        .output()?;

    if let ExecutorOutput::Wet(output) = output {
        let status = output.status;
        if !status.success() {
            io::stdout().write(&output.stdout).ok();
            io::stderr().write(&output.stderr).ok();
            Err(Error::ProcessFailed { status })?
        } else {
            println!("Plugins upgraded")
        }
    }

    Ok(())
}

#[must_use]
pub fn upgrade_vim(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let vim = require("vim")?;

    let output = Command::new(&vim).arg("--version").check_output()?;
    if !output.starts_with("VIM") {
        Err(Error::SkipStep)?;
    }

    let vimrc = require_option(vimrc(&base_dirs))?;
    let plugin_framework = require_option(PluginFramework::detect(&vimrc))?;

    print_separator(&format!("Vim ({:?})", plugin_framework));
    upgrade(&vim, &vimrc, plugin_framework, run_type)
}

#[must_use]
pub fn upgrade_neovim(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let nvim = require("nvim")?;
    let nvimrc = require_option(nvimrc(&base_dirs))?;
    let plugin_framework = require_option(PluginFramework::detect(&nvimrc))?;

    print_separator(&format!("Neovim ({:?})", plugin_framework));
    upgrade(&nvim, &nvimrc, plugin_framework, run_type)
}
