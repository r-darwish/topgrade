use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::which;
use super::utils::{Check, PathExt};
use directories::BaseDirs;
use failure::Error;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run_tpm(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(tpm) = base_dirs
        .home_dir()
        .join(".tmux/plugins/tpm/bin/update_plugins")
        .if_exists()
    {
        terminal.print_separator("tmux plugins");

        let success = || -> Result<(), Error> {
            Executor::new(&tpm, dry_run).arg("all").spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("tmux", success));
    }

    None
}

pub fn run_in_tmux() -> ! {
    let tmux = which("tmux").expect("Could not find tmux");

    let err = Command::new(tmux)
        .args(&[
            "new-session",
            "-s",
            "topgrade",
            "-n",
            "topgrade",
            &env::args().collect::<Vec<String>>().join(" "),
            ";",
            "set",
            "remain-on-exit",
            "on",
        ])
        .exec();

    panic!("{:?}", err);
}
