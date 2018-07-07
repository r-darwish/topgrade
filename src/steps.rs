use super::utils::{Check, PathExt};
use directories::BaseDirs;
use failure;
use std::path::PathBuf;
use std::process::Command;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

pub fn run_cargo_update(cargo_update: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&cargo_update)
        .args(&["install-update", "--git", "--all"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_emacs(emacs: &PathBuf, init: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&emacs)
        .args(&["--batch", "-l", init.to_str().unwrap(), "--eval", EMACS_UPGRADE])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_vim(vim: &PathBuf, vimrc: &PathBuf, upgrade_command: &str) -> Result<(), failure::Error> {
    Command::new(&vim)
        .args(&[
            "-N",
            "-u",
            vimrc.to_str().unwrap(),
            "-c",
            upgrade_command,
            "-c",
            "quitall",
            "-e",
            "-s",
            "-V1",
        ])
        .spawn()?
        .wait()?
        .check()?;

    println!("");

    Ok(())
}

pub fn run_apm(apm: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&apm)
        .args(&["upgrade", "--confirm=false"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_rustup(rustup: &PathBuf, base_dirs: &BaseDirs) -> Result<(), failure::Error> {
    if rustup.is_descendant_of(base_dirs.home_dir()) {
        Command::new(rustup).args(&["self", "update"]).spawn()?.wait()?.check()?;
    }

    Command::new(rustup).arg("update").spawn()?.wait()?.check()?;

    Ok(())
}

pub fn run_homebrew(homebrew: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&homebrew).arg("update").spawn()?.wait()?.check()?;

    Command::new(&homebrew).arg("upgrade").spawn()?.wait()?.check()?;

    Ok(())
}

pub fn run_custom_command(command: &str) -> Result<(), failure::Error> {
    Command::new("sh").arg("-c").arg(command).spawn()?.wait()?.check()?;

    Ok(())
}
