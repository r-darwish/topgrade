use super::executor::Executor;
use super::terminal::Terminal;
use super::utils::{self, Check, PathExt};
use directories::BaseDirs;
use failure::Error;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

#[must_use]
pub fn run_cargo_update(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(cargo_update) = base_dirs.home_dir().join(".cargo/bin/cargo-install-update").if_exists() {
        terminal.print_separator("Cargo");

        let success = || -> Result<(), Error> {
            Executor::new(cargo_update, dry_run)
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
pub fn run_emacs(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(emacs) = utils::which("emacs") {
        if let Some(init_file) = base_dirs.home_dir().join(".emacs.d/init.el").if_exists() {
            terminal.print_separator("Emacs");

            let success = || -> Result<(), Error> {
                Executor::new(&emacs, dry_run)
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
pub fn run_apm(terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(apm) = utils::which("apm") {
        terminal.print_separator("Atom Package Manager");

        let success = || -> Result<(), Error> {
            Executor::new(&apm, dry_run)
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
pub fn run_rustup(base_dirs: &BaseDirs, terminal: &mut Terminal, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(rustup) = utils::which("rustup") {
        terminal.print_separator("rustup");

        let success = || -> Result<(), Error> {
            if rustup.is_descendant_of(base_dirs.home_dir()) {
                Executor::new(&rustup, dry_run)
                    .args(&["self", "update"])
                    .spawn()?
                    .wait()?
                    .check()?;
            }

            Executor::new(&rustup, dry_run).arg("update").spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("rustup", success));
    }

    None
}

#[must_use]
pub fn run_custom_command(name: &str, command: &str, terminal: &mut Terminal, dry_run: bool) -> Result<(), Error> {
    terminal.print_separator(name);
    Executor::new("sh", dry_run)
        .arg("-c")
        .arg(command)
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}
