#![allow(unused_imports)]

use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use crate::{error::SkipStep, execution_context::ExecutionContext};
use anyhow::Result;
use directories::BaseDirs;
use log::debug;
#[cfg(unix)]
use nix::unistd::Uid;
#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;
use std::process::Command;

struct NPM {
    command: PathBuf,
}

impl NPM {
    fn new(command: PathBuf) -> Self {
        Self { command }
    }

    #[cfg(target_os = "linux")]
    fn root(&self) -> Result<PathBuf> {
        Command::new(&self.command)
            .args(&["root", "-g"])
            .check_output()
            .map(|s| PathBuf::from(s.trim()))
    }

    fn upgrade(&self, run_type: RunType) -> Result<()> {
        run_type.execute(&self.command).args(&["update", "-g"]).check_run()?;

        Ok(())
    }
}

pub fn run_npm_upgrade(_base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let npm = require("npm").map(NPM::new)?;

    #[cfg(target_os = "linux")]
    {
        let npm_root = npm.root()?;
        if !npm_root.exists() {
            return Err(SkipStep(format!("NPM root at {} doesn't exist", npm_root.display(),)).into());
        }

        let metadata = std::fs::metadata(&npm_root)?;
        let uid = Uid::effective();

        if metadata.uid() != uid.as_raw() {
            return Err(SkipStep(format!(
                "NPM root at {} is owned by {} which is not the current user",
                npm_root.display(),
                metadata.uid()
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
