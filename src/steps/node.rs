use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{which, Check, PathExt};
use directories::BaseDirs;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

struct NPM {
    command: PathBuf,
}

impl NPM {
    fn new(command: PathBuf) -> Self {
        Self { command }
    }

    fn root(&self) -> Result<PathBuf, Error> {
        let output = Command::new(&self.command)
            .args(&["root", "-g"])
            .output()
            .context(ErrorKind::ProcessExecution)?;

        output.status.check()?;

        Ok(PathBuf::from(
            &String::from_utf8(output.stdout).context(ErrorKind::ProcessExecution)?,
        ))
    }

    fn upgrade(&self, run_type: RunType) -> Result<(), Error> {
        run_type.execute(&self.command).args(&["update", "-g"]).check_run()?;

        Ok(())
    }
}

#[must_use]
pub fn run_npm_upgrade(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(npm) = which("npm").map(NPM::new) {
        if let Ok(npm_root) = npm.root() {
            if npm_root.is_descendant_of(base_dirs.home_dir()) {
                print_separator("Node Package Manager");
                let success = npm.upgrade(run_type).is_ok();
                return Some(("NPM", success));
            }
        }
    }
    None
}

#[must_use]
pub fn yarn_global_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(yarn) = which("yarn") {
        print_separator("Yarn");

        let success = || -> Result<(), Error> {
            run_type.execute(&yarn).args(&["global", "upgrade", "-s"]).check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("yarn", success));
    }

    None
}
