use crate::error::Error;
use crate::executor::{RunType};
use crate::terminal::print_separator;
use crate::utils::{which, Check, PathExt};
use directories::BaseDirs;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy)]
pub enum PluginFramework {
    Plug,
    Vundle,
    NeoBundle,
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
        } else {
            None
        }
    }

    pub fn upgrade_command(self) -> &'static str {
        match self {
            PluginFramework::NeoBundle => "NeoBundleUpdate",
            PluginFramework::Vundle => "PluginUpdate",
            PluginFramework::Plug => "PlugUpgrade | PlugUpdate",
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
    run_type.execute(&vim)
        .args(&[
            "-N",
            "-u",
            vimrc.to_str().unwrap(),
            "-c",
            plugin_framework.upgrade_command(),
            "-c",
            "quitall",
            "-e",
            "-s",
            "-V1",
        ])
        .spawn()?
        .wait()?
        .check()?;

    println!();

    Ok(())
}

#[must_use]
pub fn upgrade_vim(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(vim) = which("vim") {
        if let Some(vimrc) = vimrc(&base_dirs) {
            if let Some(plugin_framework) = PluginFramework::detect(&vimrc) {
                print_separator(&format!("Vim ({:?})", plugin_framework));
                let success = upgrade(&vim, &vimrc, plugin_framework, run_type).is_ok();
                return Some(("vim", success));
            }
        }
    }

    None
}

#[must_use]
pub fn upgrade_neovim(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(nvim) = which("nvim") {
        if let Some(nvimrc) = nvimrc(&base_dirs) {
            if let Some(plugin_framework) = PluginFramework::detect(&nvimrc) {
                print_separator(&format!("Neovim ({:?})", plugin_framework));
                let success = upgrade(&nvim, &nvimrc, plugin_framework, run_type).is_ok();
                return Some(("Neovim", success));
            }
        }
    }

    None
}
