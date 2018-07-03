use super::home_path;
use std::fs;
use std::path::PathBuf;
use app_dirs::*;

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
            PluginFramework::Plug => "PlugUpdate",
        }
    }
}

pub fn vimrc() -> Option<PathBuf> {
    {
        let vimrc = home_path(".vimrc");
        if vimrc.exists() {
            return Some(vimrc);
        }
    }

    {
        let vimrc = home_path(".vim/vimrc");
        if vimrc.exists() {
            return Some(vimrc);
        }
    }

    None
}

pub fn nvimrc() -> Option<PathBuf> {
    {
        let mut nvimrc = get_data_root(AppDataType::UserConfig).unwrap();
        nvimrc.push("nvim/init.vim");

        if nvimrc.exists() {
            return Some(nvimrc);
        }
    }

    {
        let mut nvimrc = get_data_root(AppDataType::UserCache).unwrap();
        nvimrc.push("nvim/init.vim");

        if nvimrc.exists() {
            return Some(nvimrc);
        }
    }

    None
}
