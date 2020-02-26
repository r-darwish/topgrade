use crate::execution_context::ExecutionContext;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::require;
use anyhow::Result;

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

pub fn upgrade_macos(run_type: RunType) -> Result<()> {
    print_separator("macOS system update");

    run_type
        .execute("softwareupdate")
        .args(&["--install", "--all"])
        .check_run()
}
