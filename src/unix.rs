use super::executor::Executor;
use super::terminal::print_separator;
use super::utils::{which, Check};
use directories::BaseDirs;
use Error;

pub fn run_zplug(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(zsh) = which("zsh") {
        if base_dirs.home_dir().join(".zplug").exists() {
            print_separator("zplug");

            let success = || -> Result<(), Error> {
                Executor::new(zsh, dry_run)
                    .args(&["-c", "source ~/.zshrc && zplug update"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            }()
            .is_ok();

            return Some(("zplug", success));
        }
    }

    None
}

pub fn run_fisher(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(fish) = which("fish") {
        if base_dirs.home_dir().join(".config/fish/functions/fisher.fish").exists() {
            print_separator("fisher");

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
            }()
            .is_ok();

            return Some(("fisher", success));
        }
    }

    None
}

#[must_use]
pub fn run_homebrew(cleanup: bool, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(brew) = which("brew") {
        print_separator("Brew");

        let inner = || -> Result<(), Error> {
            Executor::new(&brew, dry_run).arg("update").spawn()?.wait()?.check()?;
            Executor::new(&brew, dry_run).arg("upgrade").spawn()?.wait()?.check()?;
            if cleanup {
                Executor::new(&brew, dry_run).arg("cleanup").spawn()?.wait()?.check()?;
            }
            Ok(())
        };

        return Some(("Brew", inner().is_ok()));
    }

    None
}

#[must_use]
pub fn run_nix(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(nix) = which("nix") {
        if let Some(nix_env) = which("nix-env") {
            print_separator("Nix");

            let inner = || -> Result<(), Error> {
                Executor::new(&nix, dry_run)
                    .arg("upgrade-nix")
                    .spawn()?
                    .wait()?
                    .check()?;
                Executor::new(&nix_env, dry_run)
                    .arg("--upgrade")
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            };

            return Some(("Nix", inner().is_ok()));
        }
    }

    None
}
