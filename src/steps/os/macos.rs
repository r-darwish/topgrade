use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::terminal::{print_separator, prompt_yesno};
use crate::{
    error::{SkipStep, TopgradeError},
    utils::{require, PathExt},
};
use anyhow::Result;
use log::debug;
use std::{path::Path, process::Command};

pub fn run_msupdate(ctx: &ExecutionContext) -> Result<()> {
    let msupdate =
        Path::new("/Library/Application Support/Microsoft/MAU2.0/Microsoft AutoUpdate.app/Contents/MacOS/msupdate")
            .require()?;
    print_separator("Microsoft AutoUpdate");

    ctx.run_type().execute(msupdate).arg("--list").check_run()?;
    ctx.run_type().execute(msupdate).arg("--install").check_run()
}

pub fn run_brew_cask(ctx: &ExecutionContext) -> Result<()> {
    let brew = require("brew")?;
    print_separator("Brew Cask");

    let config = ctx.config();
    let run_type = ctx.run_type();

    let cask_upgrade_exists = Command::new(&brew)
        .args(&["--repository", "buo/cask-upgrade"])
        .check_output()
        .map(|p| Path::new(p.trim()).exists())?;

    let cask_args = if cask_upgrade_exists {
        let mut args = vec!["cu", "-y"];
        if config.brew_cask_greedy() {
            args.push("-a");
        }
        args
    } else {
        let mut args = vec!["cask", "upgrade"];
        if config.brew_cask_greedy() {
            args.push("--greedy");
        }
        args
    };
    run_type.execute(&brew).args(&cask_args).check_run()?;

    if ctx.config().cleanup() {
        run_type.execute(&brew).arg("cleanup").check_run()?;
    }

    Ok(())
}

pub fn run_macports(ctx: &ExecutionContext) -> Result<()> {
    require("port")?;
    let sudo = ctx.sudo().as_ref().unwrap();
    print_separator("MacPorts");
    ctx.run_type().execute(sudo).args(&["port", "selfupdate"]).check_run()?;
    ctx.run_type()
        .execute(sudo)
        .args(&["port", "-u", "upgrade", "outdated"])
        .check_run()?;
    if ctx.config().cleanup() {
        ctx.run_type()
            .execute(sudo)
            .args(&["port", "-N", "reclaim"])
            .check_run()?;
    }

    Ok(())
}

pub fn run_mas(run_type: RunType) -> Result<()> {
    let mas = require("mas")?;
    print_separator("macOS App Store");

    run_type.execute(mas).arg("upgrade").check_run()
}

pub fn upgrade_macos(ctx: &ExecutionContext) -> Result<()> {
    print_separator("macOS system update");

    let should_ask = !(ctx.config().yes()) || (ctx.config().dry_run());
    if should_ask {
        println!("Finding available software");
        if system_update_available()? {
            let answer = prompt_yesno("A system update is available. Do you wish to install it?")?;
            if !answer {
                return Err(SkipStep.into());
            }
            println!();
        } else {
            println!("No new software available.");
            return Err(SkipStep.into());
        }
    }

    let mut command = ctx.run_type().execute("softwareupdate");
    command.args(&["--install", "--all"]);

    if should_ask {
        command.arg("--no-scan");
    }

    command.check_run()
}

fn system_update_available() -> Result<bool> {
    let output = Command::new("softwareupdate").arg("--list").output()?;
    debug!("{:?}", output);

    let status = output.status;
    if !status.success() {
        return Err(TopgradeError::ProcessFailed(status).into());
    }
    let string_output = String::from_utf8(output.stderr)?;
    debug!("{:?}", string_output);
    Ok(!string_output.contains("No new software available"))
}
