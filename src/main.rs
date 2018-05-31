extern crate os_type;
extern crate which;
#[macro_use]
extern crate error_chain;

mod error {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }

    }
}

mod git;

use error::*;
use git::Git;
use os_type::OSType;
use std::collections::HashSet;
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

fn home_path(p: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(p);
    path
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
    let git = Git::new();
    let mut git_repos: HashSet<String> = HashSet::new();

    {
        let mut collect_repo = |path| {
            if let Some(repo) = git.get_repo_root(path) {
                git_repos.insert(repo);
            }
        };

        collect_repo(home_path(".emacs.d"));

        if cfg!(unix) {
            collect_repo(home_path(".zshrc"));
            collect_repo(home_path(".oh-my-zsh"));
            collect_repo(home_path(".tmux"));
        }
    }

    for repo in git_repos {
        terminal.print_seperator(format!("Pulling {}", repo));
        git.pull(repo)?;
    }

    if cfg!(unix) {
        if let Ok(zsh) = which("zsh") {
            if home_path(".zplug").exists() {
                Command::new(&zsh)
                    .arg("-c")
                    .arg("source ~/.zshrc && zplug update")
                    .spawn()?
                    .wait()?;
            }
        }

        if let Some(tpm) = tpm() {
            Command::new(&tpm).arg("all").spawn()?.wait()?;
        }
    }

    if cfg!(target_os = "linux") {
        let sudo = which("sudo");

        match os_type::current_platform().os_type {
            OSType::Arch => {
                if let Ok(yay) = which("yay") {
                    Command::new(yay).spawn()?.wait()?;
                } else {
                    if let Ok(sudo) = &sudo {
                        Command::new(&sudo)
                            .arg("pacman")
                            .arg("-Syu")
                            .spawn()?
                            .wait()?;
                    }
                }
            }
            _ => (),
        }

        if let Ok(sudo) = &sudo {
            if let Ok(needrestart) = which("needrestart") {
                Command::new(&sudo).arg(&needrestart).spawn()?.wait()?;
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

        Command::new("softwareupdate")
            .arg("--install")
            .arg("--all")
            .spawn()?
            .wait()?;
    }

    Ok(())
}

quick_main!(run);
