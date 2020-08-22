#[cfg(unix)]
use crate::error::SkipStep;
use crate::{execution_context::ExecutionContext, terminal::print_separator, utils};
use anyhow::Result;

pub fn ssh_step(ctx: &ExecutionContext, hostname: &str) -> Result<()> {
    let ssh = utils::require("ssh")?;

    let topgrade = ctx.config().remote_topgrade_path();

    if ctx.config().run_in_tmux() && !ctx.run_type().dry() {
        #[cfg(unix)]
        {
            let command = format!(
                "{ssh} -t {hostname} env TOPGRADE_PREFIX={hostname} TOPGRADE_KEEP_END=1 {topgrade}",
                ssh = ssh.display(),
                hostname = hostname,
                topgrade = topgrade
            );

            crate::tmux::run_command(ctx, &command)?;
            Err(SkipStep(String::from("Remote Topgrade launched in Tmux")).into())
        }

        #[cfg(not(unix))]
        unreachable!("Tmux execution is only implemented in Unix");
    } else {
        let mut args = vec!["-t", hostname];

        if let Some(ssh_arguments) = ctx.config().ssh_arguments() {
            args.extend(ssh_arguments.split_whitespace());
        }

        let env = format!("TOPGRADE_PREFIX={}", hostname);
        args.extend(&["env", &env, topgrade]);

        if ctx.config().yes() {
            args.push("-y");
        }

        print_separator(format!("Remote ({})", hostname));
        println!("Connecting to {}...", hostname);

        ctx.run_type().execute(&ssh).args(&args).check_run()
    }
}
