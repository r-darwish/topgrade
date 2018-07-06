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
            PluginFramework::Plug => "PlugUpdate",
        }
    }
}

pub fn vimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    {
        let vimrc = base_dirs.home_dir().join(".vimrc");
        if vimrc.exists() {
            return Some(vimrc);
        }
    }

    {
        let vimrc = base_dirs.home_dir().join(".vim/vimrc");
        if vimrc.exists() {
            return Some(vimrc);
        }
    }

    None
}

pub fn nvimrc(base_dirs: &BaseDirs) -> Option<PathBuf> {
    {
        let nvimrc = base_dirs.config_dir().join("nvim/init.vim");

        if nvimrc.exists() {
            return Some(nvimrc);
        }
    }

    {
        let nvimrc = base_dirs.cache_dir().join("nvim/init.vim");

        if nvimrc.exists() {
            return Some(nvimrc);
        }
    }

    None
}
