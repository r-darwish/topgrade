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

#[allow(clippy::upper_case_acronyms)]
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

    fn upgrade(&self, run_type: RunType, use_sudo: bool) -> Result<()> {
        if use_sudo {
            run_type
                .execute("sudo")
                .arg(&self.command)
                .args(&["update", "-g"])
                .check_run()?;
        } else {
            run_type.execute(&self.command).args(&["update", "-g"]).check_run()?;
        }

        Ok(())
    }
}

pub fn run_npm_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let npm = require("npm").map(NPM::new)?;
    #[allow(unused_mut)]
    let mut use_sudo = false;

    #[cfg(target_os = "linux")]
    {
        let npm_root = npm.root()?;
        if !npm_root.exists() {
            return Err(SkipStep(format!("NPM root at {} doesn't exist", npm_root.display(),)).into());
        }

        let metadata = std::fs::metadata(&npm_root)?;
        let uid = Uid::effective();

        if metadata.uid() != uid.as_raw() {
            if metadata.uid() == 0 && (ctx.config().npm_use_sudo()) {
                use_sudo = true;
            } else {
                return Err(SkipStep(format!(
                    "NPM root at {} is owned by {} which is not the current user. Set use_sudo = true under the NPM section in your configuration to run NPM as sudo",
                    npm_root.display(),
                    metadata.uid()
                ))
                    .into());
            }
        }
    }

    print_separator("Node Package Manager");
    npm.upgrade(ctx.run_type(), use_sudo)
}

pub fn pnpm_global_update(run_type: RunType) -> Result<()> {
    let pnpm = require("pnpm")?;

    print_separator("Performant Node Package Manager");
    run_type.execute(&pnpm).args(&["update", "-g"]).check_run()
}

pub fn deno_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let deno = require("deno")?;

    print_separator("Deno");
    ctx.run_type().execute(&deno).arg("upgrade").check_run()
}
