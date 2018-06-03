extern crate os_type;
extern crate which;
#[macro_use]
extern crate error_chain;
extern crate termion;

mod error {
    error_chain!{
        foreign_links {
            Io(::std::io::Error);
        }

    }
}

mod git;
mod terminal;

use error::*;
use git::Git;
use os_type::OSType;
use std::collections::HashSet;
use std::env::home_dir;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use terminal::Terminal;
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

const EMACS_UPGRADE: &str = "(progn (let ((package-menu-async nil))
         (package-list-packages))
       (package-menu-mark-upgrades)
       (package-menu-execute 'noquery))";

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
    let terminal = Terminal::new();

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
        terminal.print_separator(format!("Pulling {}", repo));
        git.pull(repo)?;
    }

    if cfg!(unix) {
        if let Ok(zsh) = which("zsh") {
            terminal.print_separator("zplug");
            if home_path(".zplug").exists() {
                Command::new(&zsh)
                    .arg("-c")
                    .arg("source ~/.zshrc && zplug update")
                    .spawn()?
                    .wait()?;
            }
        }

        if let Some(tpm) = tpm() {
            terminal.print_separator("tmux plugins");
            Command::new(&tpm).arg("all").spawn()?.wait()?;
        }
    }

    let cargo_upgrade = home_path(".cargo/bin/cargo-install-update");
    if cargo_upgrade.exists() {
        terminal.print_separator("Cargo");
        Command::new(&cargo_upgrade)
            .arg("install-update")
            .arg("--all")
            .spawn()?
            .wait()?;
    }

    if let Ok(emacs) = which("emacs") {
        if home_path(".emacs.d").exists() {
            terminal.print_separator("Emacs");
            Command::new(&emacs)
                .arg("--batch")
                .arg("-l")
                .arg(home_path(".emacs.d/init.el"))
                .arg("--eval")
                .arg(EMACS_UPGRADE)
                .spawn()?
                .wait()?;
        }
    }

    if cfg!(target_os = "linux") {
        let sudo = which("sudo");

        match os_type::current_platform().os_type {
            OSType::Arch => {
                terminal.print_separator("System update");
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

            OSType::CentOS | OSType::Redhat => {
                if let Ok(sudo) = &sudo {
                    Command::new(&sudo)
                        .arg("yum")
                        .arg("upgrade")
                        .spawn()?
                        .wait()?;
                }
            }

            OSType::Ubuntu | OSType::Debian => {
                if let Ok(sudo) = &sudo {
                    Command::new(&sudo)
                        .arg("apt")
                        .arg("update")
                        .spawn()?
                        .wait()?
                        .and_then(|| {
                            Command::new(&sudo)
                                .arg("apt")
                                .arg("dist-upgrade")
                                .spawn()?
                                .wait()
                        })?;
                }
            }

            OSType::Unknown => {
                println!(
                    "Could not detect your Linux distribution. Do you have lsb-release installed?"
                );
            }

            _ => (),
        }

        if let Ok(fwupdmgr) = which("fwupdmgr") {
            terminal.print_separator("Firmware upgrades");
            Command::new(&fwupdmgr)
                .arg("refresh")
                .spawn()?
                .wait()?
                .and_then(|| Command::new(&fwupdmgr).arg("get-updates").spawn()?.wait())?;
        }

        if let Ok(sudo) = &sudo {
            if let Ok(needrestart) = which("needrestart") {
                terminal.print_separator("Check for needed restarts");
                Command::new(&sudo).arg(&needrestart).spawn()?.wait()?;
            }
        }
    }

    if cfg!(target_os = "macos") {
        if let Ok(brew) = which("brew") {
            terminal.print_separator("Homebrew");
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

        terminal.print_separator("System update");
        Command::new("softwareupdate")
            .arg("--install")
            .arg("--all")
            .spawn()?
            .wait()?;
    }

    Ok(())
}

quick_main!(run);
