use super::terminal::Terminal;
use super::utils::{which, Check, PathExt};
use directories::BaseDirs;
use failure;
use std::path::PathBuf;
use std::process::Command;

struct NPM {
    command: PathBuf,
}

impl NPM {
    fn new(command: PathBuf) -> Self {
        Self { command }
    }

    fn root(&self) -> Result<PathBuf, failure::Error> {
        let output = Command::new(&self.command).args(&["root", "-g"]).output()?;

        output.status.check()?;

        Ok(PathBuf::from(&String::from_utf8(output.stdout)?))
    }

    fn upgrade(&self) -> Result<(), failure::Error> {
        Command::new(&self.command)
            .args(&["update", "-g"])
            .spawn()?
            .wait()?
            .check()?;

        Ok(())
    }
}

#[must_use]
pub fn run_npm_upgrade(base_dirs: &BaseDirs, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(npm) = which("npm").map(NPM::new) {
        if let Ok(npm_root) = npm.root() {
            if npm_root.is_descendant_of(base_dirs.home_dir()) {
                terminal.print_separator("Node Package Manager");
                let success = npm.upgrade().is_ok();
                return Some(("NPM", success));
            }
        }
    }
    None
}

#[must_use]
pub fn yarn_global_update(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(yarn) = which("yarn") {
        terminal.print_separator("Yarn");

        let success = || -> Result<(), failure::Error> {
            Command::new(&yarn)
                .args(&["global", "upgrade", "-s"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }().is_ok();

        return Some(("yarn", success));
    }

    None
}
