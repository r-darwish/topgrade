use super::utils;
use super::utils::Check;
use failure;
use std::env;
use std::env::home_dir;
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

pub fn run_tpm(tpm: &PathBuf) -> Result<(), failure::Error> {
    Command::new(&tpm).arg("all").spawn()?.wait()?.check()?;

    Ok(())
}

pub fn tpm_path() -> Option<PathBuf> {
    let mut path = home_dir().unwrap();
    path.push(".tmux/plugins/tpm/bin/update_plugins");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

pub fn run_in_tmux() -> ! {
    let tmux = utils::which("tmux").expect("Could not find tmux");
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
