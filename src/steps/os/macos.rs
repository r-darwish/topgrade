use crate::execution_context::ExecutionContext;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{require, PathExt};
use anyhow::Result;
use std::path::Path;

pub fn run_msupdate(ctx: &ExecutionContext) -> Result<()> {
    let msupdate =
        Path::new("/Library/Application Support/Microsoft/MAU2.0/Microsoft AutoUpdate.app/Contents/MacOS/msupdate")
            .require()?;
    print_separator("Microsoft AutoUpdate");

    ctx.run_type().execute(msupdate).arg("--list").check_run()?;
    ctx.run_type().execute(msupdate).arg("--install").check_run()
}

pub fn run_mas(run_type: RunType) -> Result<()> {
    let mas = require("mas")?;
    print_separator("macOS App Store");

    run_type.execute(mas).arg("upgrade").check_run()
}

pub fn upgrade_macos(run_type: RunType) -> Result<()> {
    print_separator("macOS system update");

    run_type
        .execute("softwareupdate")
        .args(&["--install", "--all"])
        .check_run()
}
