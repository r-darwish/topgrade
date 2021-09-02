use crate::execution_context::ExecutionContext;
use crate::terminal::print_separator;
use crate::utils::require;
use anyhow::Result;

pub fn upgrade_packages(ctx: &ExecutionContext) -> Result<()> {
    let pkg = require("pkg")?;

    print_separator("Termux Packages");

    let mut command = ctx.run_type().execute(&pkg);
    command.arg("upgrade");
    if ctx.config().yes() {
        command.arg("-y");
    }
    command.check_run()?;

    if ctx.config().cleanup() {
        ctx.run_type().execute(&pkg).arg("clean").check_run()?;

        let apt = require("apt")?;
        let mut command = ctx.run_type().execute(&apt);
        command.arg("autoremove");
        if ctx.config().yes() {
            command.arg("-y");
        }
        command.check_run()?;
    }

    Ok(())
}
