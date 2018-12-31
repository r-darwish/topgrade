use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{which, Check};
use directories::BaseDirs;

pub fn run_zplug(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(zsh) = which("zsh") {
        if base_dirs.home_dir().join(".zplug").exists() {
            print_separator("zplug");

            let success = || -> Result<(), Error> {
                run_type
                    .execute(zsh)
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

pub fn run_fisher(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(fish) = which("fish") {
        if base_dirs.home_dir().join(".config/fish/functions/fisher.fish").exists() {
            print_separator("fisher");

            let success = || -> Result<(), Error> {
                run_type
                    .execute(&fish)
                    .args(&["-c", "fisher self-update"])
                    .spawn()?
                    .wait()?
                    .check()?;
                run_type
                    .execute(&fish)
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
pub fn run_homebrew(cleanup: bool, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(brew) = which("brew") {
        print_separator("Brew");

        let inner = || -> Result<(), Error> {
            run_type.execute(&brew).arg("update").spawn()?.wait()?;
            run_type.execute(&brew).arg("upgrade").spawn()?.wait()?;
            run_type
                .execute(&brew)
                .args(&["cask", "upgrade"])
                .spawn()?
                .wait()?
                .check()?;
            if cleanup {
                run_type.execute(&brew).arg("cleanup").spawn()?.wait()?;
            }
            Ok(())
        };

        return Some(("Brew", inner().is_ok()));
    }

    None
}

#[must_use]
pub fn run_nix(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(nix) = which("nix") {
        if let Some(nix_env) = which("nix-env") {
            print_separator("Nix");

            let inner = || -> Result<(), Error> {
                run_type.execute(&nix).arg("upgrade-nix").spawn()?.wait()?.check()?;
                run_type.execute(&nix_env).arg("--upgrade").spawn()?.wait()?.check()?;
                Ok(())
            };

            return Some(("Nix", inner().is_ok()));
        }
    }

    None
}
