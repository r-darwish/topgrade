use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{require, require_option, PathExt};
use anyhow::Result;
use directories::BaseDirs;
#[cfg(any(windows, target_os = "macos"))]
use std::env;
use std::path::{Path, PathBuf};

const EMACS_UPGRADE: &str = include_str!("emacs.el");
#[cfg(windows)]
const DOOM_PATH: &str = "bin/doom.cmd";
#[cfg(unix)]
const DOOM_PATH: &str = "bin/doom";

pub struct Emacs {
    directory: Option<PathBuf>,
    doom: Option<PathBuf>,
}

impl Emacs {
    fn directory_path(base_dirs: &BaseDirs) -> Option<PathBuf> {
        #[cfg(unix)]
        cfg_if::cfg_if! {
            if #[cfg(target_os = "macos")] {
                let emacs_xdg_dir = env::var("XDG_CONFIG_HOME")
                    .ok()
                    .and_then(|config| PathBuf::from(config).join("emacs").if_exists())
                    .or_else(|| base_dirs.home_dir().join(".config/emacs").if_exists());
            } else {
                let emacs_xdg_dir = base_dirs.config_dir().join("emacs").if_exists();
            }
        }
        #[cfg(unix)]
        return base_dirs
            .home_dir()
            .join(".emacs.d")
            .if_exists()
            .or_else(|| emacs_xdg_dir);

        #[cfg(windows)]
        return env::var("HOME")
            .ok()
            .and_then(|home| PathBuf::from(home).join(".emacs.d").if_exists())
            .or_else(|| base_dirs.data_dir().join(".emacs.d").if_exists());
    }

    pub fn new(base_dirs: &BaseDirs) -> Self {
        let directory = Emacs::directory_path(base_dirs);
        let doom = directory.as_ref().and_then(|d| d.join(DOOM_PATH).if_exists());
        Self { directory, doom }
    }

    pub fn is_doom(&self) -> bool {
        self.doom.is_some()
    }

    pub fn directory(&self) -> Option<&PathBuf> {
        self.directory.as_ref()
    }

    fn update_doom(doom: &Path, run_type: RunType) -> Result<()> {
        print_separator("Doom Emacs");

        run_type.execute(doom).args(&["-y", "upgrade"]).check_run()
    }

    pub fn upgrade(&self, run_type: RunType) -> Result<()> {
        let emacs = require("emacs")?;
        let init_file = require_option(self.directory.as_ref())?.join("init.el").require()?;

        if let Some(doom) = &self.doom {
            return Emacs::update_doom(doom, run_type);
        }

        print_separator("Emacs");

        let mut command = run_type.execute(&emacs);

        command
            .args(&["--batch", "--debug-init", "-l"])
            .arg(init_file)
            .arg("--eval");

        #[cfg(unix)]
        command.arg(
            EMACS_UPGRADE
                .chars()
                .map(|c| if c.is_whitespace() { '\u{00a0}' } else { c })
                .collect::<String>(),
        );

        #[cfg(not(unix))]
        command.arg(EMACS_UPGRADE);

        command.check_run()
    }
}
