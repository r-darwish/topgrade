use super::terminal::Terminal;
use super::utils;
use super::utils::{Check, PathExt};
use directories::BaseDirs;
use failure::Error;
use std::process::Command;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

#[must_use]
pub fn run_cargo_update(base_dirs: &BaseDirs, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(cargo_update) = base_dirs.home_dir().join(".cargo/bin/cargo-install-update").if_exists() {
        terminal.print_separator("Cargo");

        let success = || -> Result<(), Error> {
            Command::new(cargo_update)
                .args(&["install-update", "--git", "--all"])
                .spawn()?
                .wait()?
                .check()?;

            Ok(())
        }().is_ok();

        return Some(("Cargo", success));
    }

    None
}

#[must_use]
pub fn run_emacs(base_dirs: &BaseDirs, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(emacs) = utils::which("emacs") {
        if let Some(init_file) = base_dirs.home_dir().join(".emacs.d/init.el").if_exists() {
            terminal.print_separator("Emacs");

            let success = || -> Result<(), Error> {
                Command::new(&emacs)
                    .args(&["--batch", "-l", init_file.to_str().unwrap(), "--eval", EMACS_UPGRADE])
                    .spawn()?
                    .wait()?
                    .check()?;

                Ok(())
            }().is_ok();

            return Some(("Emacs", success));
        }
    }
    None
}

#[must_use]
pub fn run_apm(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(apm) = utils::which("apm") {
        terminal.print_separator("Atom Package Manager");

        let success = || -> Result<(), Error> {
            Command::new(&apm)
                .args(&["upgrade", "--confirm=false"])
                .spawn()?
                .wait()?
                .check()?;

            Ok(())
        }().is_ok();

        return Some(("apm", success));
    }

    None
}

#[must_use]
pub fn run_rustup(base_dirs: &BaseDirs, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(rustup) = utils::which("rustup") {
        terminal.print_separator("rustup");

        let success = || -> Result<(), Error> {
            if rustup.is_descendant_of(base_dirs.home_dir()) {
                Command::new(&rustup)
                    .args(&["self", "update"])
                    .spawn()?
                    .wait()?
                    .check()?;
            }

            Command::new(&rustup).arg("update").spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("rustup", success));
    }

    None
}

#[must_use]
pub fn run_custom_command(name: &str, command: &str, terminal: &mut Terminal) -> Result<(), Error> {
    terminal.print_separator(name);
    Command::new("sh").arg("-c").arg(command).spawn()?.wait()?.check()?;

    Ok(())
}
