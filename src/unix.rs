use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::{which, Check};
use directories::BaseDirs;
use failure::Error;

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

pub fn run_fisher(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(fish) = which("fish") {
        if base_dirs.home_dir().join(".config/fish/functions/fisher.fish").exists() {
            terminal.print_separator("fisher");

            let success = || -> Result<(), Error> {
                Executor::new(&fish, dry_run)
                    .args(&["-c", "fisher self-update"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Executor::new(&fish, dry_run)
                    .args(&["-c", "fisher"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            }().is_ok();

            return Some(("fisher", success));
        }
    }

    None
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
