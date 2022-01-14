#![allow(unused_imports)]

#[cfg(unix)]
use std::os::unix::prelude::MetadataExt;
use std::path::PathBuf;
use std::process::Command;

use anyhow::Result;
use directories::BaseDirs;
use log::debug;
#[cfg(unix)]
use nix::unistd::Uid;

use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use crate::{error::SkipStep, execution_context::ExecutionContext};

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
        print_separator("Node Package Manager");

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

    #[cfg(target_os = "linux")]
    pub fn should_use_sudo(&self) -> Result<bool> {
        let npm_root = self.root()?;
        if !npm_root.exists() {
            return Err(SkipStep(format!("NPM root at {} doesn't exist", npm_root.display(),)).into());
        }

        let metadata = std::fs::metadata(&npm_root)?;
        let uid = Uid::effective();

        Ok(metadata.uid() != uid.as_raw() && metadata.uid() == 0)
    }
}

#[cfg(target_os = "linux")]
fn should_use_sudo(npm: &NPM, ctx: &ExecutionContext) -> Result<bool> {
    if npm.should_use_sudo()? {
        if ctx.config().npm_use_sudo() {
            Ok(true)
        } else {
            Err(SkipStep("NPM root is owned by another user which is not the current user. Set use_sudo = true under the NPM section in your configuration to run NPM as sudo".to_string())
                .into())
        }
    } else {
        Ok(false)
    }
}

pub fn run_npm_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let npm = require("npm").map(NPM::new)?;

    #[cfg(target_os = "linux")]
    {
        npm.upgrade(ctx.run_type(), should_use_sudo(&npm, ctx)?)
    }

    #[cfg(not(target_os = "linux"))]
    {
        npm.upgrade(ctx.run_type(), false)
    }
}

pub fn pnpm_global_update(ctx: &ExecutionContext) -> Result<()> {
    let pnpm = require("pnpm")?;

    print_separator("Performant Node Package Manager");
    #[cfg(target_os = "linux")]
    if should_use_sudo(&require("npm").map(NPM::new)?, ctx)? {
        ctx.run_type()
            .execute("sudo")
            .arg(pnpm)
            .args(&["update", "-g"])
            .check_run()
    } else {
        ctx.run_type().execute(&pnpm).args(&["update", "-g"]).check_run()
    }

    #[cfg(not(target_os = "linux"))]
    ctx.run_type().execute(&pnpm).args(&["update", "-g"]).check_run()
}

pub fn deno_upgrade(ctx: &ExecutionContext) -> Result<()> {
    let deno = require("deno")?;
    let deno_dir = ctx.base_dirs().home_dir().join(".deno");

    if !deno.canonicalize()?.is_descendant_of(&deno_dir) {
        let skip_reason = SkipStep("Deno installed outside of .deno directory".to_string());
        return Err(skip_reason.into());
    }

    print_separator("Deno");
    ctx.run_type().execute(&deno).arg("upgrade").check_run()
}
