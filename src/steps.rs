use super::Check;
use failure;
use std::path::PathBuf;
use std::process::Command;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

pub fn run_zplug(zsh: &PathBuf) -> Result<(), failure::Error> {
    Command::new(zsh)
        .args(&["-c", "source ~/.zshrc && zplug update"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_tpm(tpm: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&tpm).arg("all").spawn()?.wait()?.check()?;

    Ok(())
}

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
        .args(&[
            "--batch",
            "-l",
            init.to_str().unwrap(),
            "--eval",
            EMACS_UPGRADE,
        ])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_vim(
    vim: &PathBuf,
    vimrc: &PathBuf,
    upgrade_command: &str,
) -> Result<(), failure::Error> {
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

pub fn run_needrestart(sudo: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&sudo)
        .arg("needrestart")
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_fwupdmgr(fwupdmgr: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&fwupdmgr)
        .arg("refresh")
        .spawn()?
        .wait()?
        .check()?;

    Command::new(&fwupdmgr)
        .arg("get-updates")
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_rustup(rustup: &PathBuf) -> Result<(), failure::Error> {
    Command::new(rustup).arg("update").spawn()?.wait()?.check()?;

    Ok(())
}

pub fn upgrade_macos() -> Result<(), failure::Error> {
    Command::new("softwareupdate")
        .args(&["--install", "--all"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_homebrew(homebrew: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&homebrew)
        .arg("update")
        .spawn()?
        .wait()?
        .check()?;

    Command::new(&homebrew)
        .arg("upgrade")
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_custom_command(command: &str) -> Result<(), failure::Error> {
    Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}
