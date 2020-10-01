#![allow(unused_imports)]

use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use crate::{error::SkipStep, execution_context::ExecutionContext};
use anyhow::Result;
use log::debug;

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
            return Err(SkipStep(format!(
                "NPM root at {} isn't a decandent of the user's home directory",
                npm_root.display()
            ))
            .into());
        }
    }

    print_separator("Node Package Manager");
    npm.upgrade(run_type)
}

pub fn yarn_global_update(run_type: RunType) -> Result<()> {
    let yarn = require("yarn")?;

    let output = Command::new(&yarn).arg("--version").string_output()?;
    if output.contains("Hadoop") {
        return Err(SkipStep(String::from("Installed yarn is Hadoop's yarn")).into());
    }

    print_separator("Yarn");
    run_type.execute(&yarn).args(&["global", "upgrade", "-s"]).check_run()
}

pub fn deno_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let deno = require("deno")?;

    print_separator("Deno");
    ctx.run_type().execute(&deno).arg("upgrade").check_run()
}
