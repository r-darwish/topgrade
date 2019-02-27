use crate::error::Error;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{require, which, PathExt};
use directories::BaseDirs;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn zplug_exists(base_dirs: &BaseDirs) -> bool {
    env::var("ZPLUG_HOME")
        .map(|ref zplug_home| Path::new(zplug_home).exists())
        .unwrap_or(false)
        || base_dirs.home_dir().join("zplug").exists()
}

fn get_zshrc(base_dirs: &BaseDirs) -> PathBuf {
    env::var("ZDOTDIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| base_dirs.home_dir().join(".zshrc"))
}

pub fn run_zplug(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(zsh) = which("zsh") {
        if zplug_exists(base_dirs) {
            print_separator("zplug");

            let success = || -> Result<(), Error> {
                let zshrc = get_zshrc(base_dirs).require()?;
                let cmd = format!("source {} && zplug update", zshrc.display());
                run_type.execute(zsh).args(&["-c", cmd.as_str()]).check_run()?;
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
                    .check_run()?;
                run_type.execute(&fish).args(&["-c", "fisher"]).check_run()?;
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
            run_type.execute(&brew).arg("update").check_run()?;
            run_type.execute(&brew).arg("upgrade").check_run()?;

            let cask_upgrade_exists = Command::new(&brew)
                .args(&["--repository", "buo/cask-upgrade"])
                .check_output()
                .map(|p| Path::new(p.trim()).exists())?;

            if cask_upgrade_exists {
                run_type.execute(&brew).args(&["cu", "-a"]).check_run()?;
            } else {
                run_type.execute(&brew).args(&["cask", "upgrade"]).check_run()?;
            }

            if cleanup {
                run_type.execute(&brew).arg("cleanup").check_run()?;
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
                run_type.execute(&nix).arg("upgrade-nix").check_run()?;
                run_type.execute(&nix_env).arg("--upgrade").check_run()?;
                Ok(())
            };

            return Some(("Nix", inner().is_ok()));
        }
    }

    None
}

pub fn run_pearl(run_type: RunType) -> Result<(), Error> {
    let pearl = require("pearl")?;
    print_separator("pearl");

    run_type.execute(&pearl).arg("update").check_run()
}
