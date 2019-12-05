use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{require, require_option, PathExt};
use anyhow::Result;
use directories::BaseDirs;
#[cfg(windows)]
use std::env;
use std::path::PathBuf;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

pub struct Emacs {
    directory: Option<PathBuf>,
}

impl Emacs {
    fn directory_path(base_dirs: &BaseDirs) -> Option<PathBuf> {
        #[cfg(unix)]
        return base_dirs.home_dir().join(".emacs.d").if_exists();

        #[cfg(windows)]
        return env::var("HOME")
            .ok()
            .and_then(|home| PathBuf::from(home).join(".emacs.d").if_exists())
            .or_else(|| base_dirs.data_dir().join(".emacs.d").if_exists());
    }

    pub fn new(base_dirs: &BaseDirs) -> Self {
        Self {
            directory: Emacs::directory_path(base_dirs),
        }
    }

    pub fn directory(&self) -> Option<&PathBuf> {
        self.directory.as_ref()
    }

    pub fn upgrade(&self, run_type: RunType) -> Result<()> {
        let emacs = require("emacs")?;
        let init_file = require_option(self.directory.as_ref())?.join("init.el").require()?;

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
