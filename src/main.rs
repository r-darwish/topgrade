extern crate which;
#[macro_use]
extern crate error_chain;

mod errors {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }

    }
}

use errors::*;
use std::env::home_dir;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use which::which;

trait Chain
where
    Self: std::marker::Sized,
{
    fn and_then<F>(self, f: F) -> ::std::io::Result<Self>
    where
        F: FnOnce() -> ::std::io::Result<Self>;
}

impl Chain for ExitStatus {
    fn and_then<F>(self, f: F) -> ::std::io::Result<Self>
    where
        F: FnOnce() -> ::std::io::Result<Self>,
    {
        if !self.success() {
            Ok(self)
        } else {
            f()
        }
    }
}

#[cfg(unix)]
fn zplug_exists() -> bool {
    let mut path = home_dir().unwrap();
    path.push(".zplug");
    path.exists()
}

#[cfg(unix)]
fn tpm() -> Option<PathBuf> {
    let mut path = home_dir().unwrap();
    path.push(".tmux/plugins/tpm/bin/update_plugins");
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

fn run() -> Result<()> {
    if cfg!(unix) {
        if let Ok(zsh) = which("zsh") {
            if zplug_exists() {
                Command::new(&zsh)
                    .arg("-ic")
                    .arg("zplug update")
                    .spawn()?
                    .wait()?;
            }
        }

        if let Some(tpm) = tpm() {
            if zplug_exists() {
                Command::new(&tpm).arg("all").spawn()?.wait()?;
            }
        }
    }

    if cfg!(target_os = "macos") {
        if let Ok(brew) = which("brew") {
            Command::new(&brew)
                .arg("update")
                .spawn()?
                .wait()?
                .and_then(|| Command::new(&brew).arg("upgrade").spawn()?.wait())?
                .and_then(|| {
                    Command::new(&brew)
                        .arg("cleanup")
                        .arg("-sbr")
                        .spawn()?
                        .wait()
                })?;
        }
    }

    if cfg!(target_os = "macos") {
        Command::new("softwareupdate")
            .arg("--install")
            .arg("--all")
            .spawn()?
            .wait()?;
    }

    Ok(())
}

quick_main!(run);
