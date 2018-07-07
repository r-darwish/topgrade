use super::utils::{which, Check, PathExt};
use directories::BaseDirs;
use failure;
use std::env;
use std::os::unix::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;

pub fn run_zplug(zsh: &PathBuf) -> Result<(), failure::Error> {
    Command::new(zsh)
        .args(&["-c", "source ~/.zshrc && zplug update"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_fisherman(fish: &PathBuf) -> Result<(), failure::Error> {
    Command::new(fish)
        .args(&["-c", "fisher update"])
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

pub fn run_tpm(tpm: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&tpm).arg("all").spawn()?.wait()?.check()?;

    Ok(())
}

pub fn tpm_path(base_dirs: &BaseDirs) -> Option<PathBuf> {
    base_dirs
        .home_dir()
        .join(".tmux/plugins/tpm/bin/update_plugins")
        .if_exists()
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
