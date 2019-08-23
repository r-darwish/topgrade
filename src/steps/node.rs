use crate::error::Error;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use directories::BaseDirs;
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
        Command::new(&self.command)
            .args(&["root", "-g"])
            .check_output()
            .map(PathBuf::from)
    }

    fn upgrade(&self, run_type: RunType) -> Result<(), Error> {
        run_type.execute(&self.command).args(&["update", "-g"]).check_run()?;

        Ok(())
    }
}

pub fn run_npm_upgrade(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let npm = require("npm").map(NPM::new)?;
    let npm_root = npm.root()?;
    if !npm_root.is_descendant_of(base_dirs.home_dir()) {
        Err(Error::SkipStep)?;
    }

    print_separator("Node Package Manager");
    npm.upgrade(run_type)
}

pub fn yarn_global_update(run_type: RunType) -> Result<(), Error> {
    let yarn = require("yarn")?;

    print_separator("Yarn");
    run_type.execute(&yarn).args(&["global", "upgrade", "-s"]).check_run()
}
