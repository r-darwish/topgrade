use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::{which, Check, PathExt};
use directories::BaseDirs;
use failure::Error;
use std::env;
use std::os::unix::process::CommandExt;
use std::process::Command;

pub fn run_zplug(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(zsh) = which("zsh") {
        if base_dirs.home_dir().join(".zplug").exists() {
            terminal.print_separator("zplug");

            let success = || -> Result<(), Error> {
                Executor::new(zsh, dry_run)
                    .args(&["-c", "source ~/.zshrc && zplug update"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            }().is_ok();

            return Some(("zplug", success));
        }
    }

    None
}

pub fn run_fisherman(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(fish) = which("fish") {
        if base_dirs.home_dir().join(".config/fish/functions/fisher.fish").exists() {
            terminal.print_separator("fisherman");

            let success = || -> Result<(), Error> {
                Executor::new(fish, dry_run)
                    .args(&["-c", "fisher update"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            }().is_ok();

            return Some(("fisherman", success));
        }
    }

    None
}

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

#[must_use]
pub fn run_homebrew(terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(brew) = which("brew") {
        terminal.print_separator("Brew");

        let inner = || -> Result<(), Error> {
            Executor::new(&brew, dry_run).arg("update").spawn()?.wait()?.check()?;
            Executor::new(&brew, dry_run).arg("upgrade").spawn()?.wait()?.check()?;
            Ok(())
        };

        return Some(("Brew", inner().is_ok()));
    }

    None
}
