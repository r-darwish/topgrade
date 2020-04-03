#![allow(unused_imports)]
use crate::error::SkipStep;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use log::debug;
use anyhow::Result;

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

    #[cfg(not(target_os = "macos"))]
    fn root(&self) -> Result<PathBuf> {
        Command::new(&self.command)
            .args(&["root", "-g"])
            .check_output()
            .map(PathBuf::from)
    }

    fn upgrade(&self, run_type: RunType) -> Result<()> {
        run_type.execute(&self.command).args(&["update", "-g"]).check_run()?;

        Ok(())
    }
}

pub fn run_npm_upgrade(_base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let npm = require("npm").map(NPM::new)?;

    #[cfg(not(target_os = "macos"))]
    {
        let npm_root = npm.root()?;
        if !npm_root.is_descendant_of(_base_dirs.home_dir()) {
            return Err(SkipStep.into());
        }
    }

    print_separator("Node Package Manager");
    npm.upgrade(run_type)
}

pub fn yarn_global_update(run_type: RunType) -> Result<()> {
    let yarn = require("yarn")?;

    let output = Command::new(&yarn).arg("version").check_output()?;
    if output.contains("Hadoop") {
        debug!("Yarn is Hadoop yarn");
        return Err(SkipStep.into());
    }

    print_separator("Yarn");
    run_type.execute(&yarn).args(&["global", "upgrade", "-s"]).check_run()
}
