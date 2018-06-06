extern crate failure;
extern crate os_type;
extern crate which;
#[macro_use]
extern crate failure_derive;
extern crate termion;

mod git;
mod report;
mod terminal;

use failure::Error;
use git::Git;
use os_type::OSType;
use report::{Report, Reporter};
use std::collections::HashSet;
use std::env::home_dir;
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use terminal::Terminal;
use which::which;

#[derive(Fail, Debug)]
#[fail(display = "Process failed")]
struct ProcessFailed;

trait Check {
    fn check(self) -> Result<(), Error>;
}

impl Check for ExitStatus {
    fn check(self) -> Result<(), Error> {
        if self.success() {
            Ok(())
        } else {
            Err(Error::from(ProcessFailed {}))
        }
    }
}

const EMACS_UPGRADE: &str = include_str!("emacs.el");

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

fn main() -> Result<(), Error> {
    let git = Git::new();
    let mut git_repos: HashSet<String> = HashSet::new();
    let terminal = Terminal::new();
    let mut reports = Report::new();

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
        if let Some(success) = git.pull(&repo)? {
            success.report(format!("git: {}", repo), &mut reports);
        }
    }

    if cfg!(unix) {
        if let Ok(zsh) = which("zsh") {
            terminal.print_separator("zplug");
            if home_path(".zplug").exists() {
                Command::new(&zsh)
                    .args(&["-c", "source ~/.zshrc && zplug update"])
                    .spawn()?
                    .wait()?
                    .report("zplug", &mut reports);
            }
        }

        if let Some(tpm) = tpm() {
            terminal.print_separator("tmux plugins");
            Command::new(&tpm)
                .arg("all")
                .spawn()?
                .wait()?
                .report("tmux", &mut reports);
        }
    }

    let cargo_upgrade = home_path(".cargo/bin/cargo-install-update");
    if cargo_upgrade.exists() {
        terminal.print_separator("Cargo");
        Command::new(&cargo_upgrade)
            .args(&["install-update", "--all"])
            .spawn()?
            .wait()?
            .report("Cargo", &mut reports);
    }

    if let Ok(emacs) = which("emacs") {
        if home_path(".emacs.d").exists() {
            terminal.print_separator("Emacs");
            Command::new(&emacs)
                .args(&[
                    "--batch",
                    "-l",
                    home_path(".emacs.d/init.el").to_str().unwrap(),
                    "--eval",
                    EMACS_UPGRADE,
                ])
                .spawn()?
                .wait()?
                .report("Emacs", &mut reports);
        }
    }

    if let Ok(npm) = which("npm") {
        terminal.print_separator("Node Package Manager");
        Command::new(&npm)
            .args(&["update", "-g"])
            .spawn()?
            .wait()?
            .report("Node Package Manager", &mut reports);
    }

    if let Ok(apm) = which("apm") {
        terminal.print_separator("Atom Package Manager");
        Command::new(&apm)
            .args(&["upgrade", "--confirm=false"])
            .spawn()?
            .wait()
            .map_err(Error::from)?
            .report("Atom Package Manager", &mut reports);
    }

    if cfg!(target_os = "linux") {
        let sudo = which("sudo");

        terminal.print_separator("System update");
        match os_type::current_platform().os_type {
            OSType::Arch => {
                if let Ok(yay) = which("yay") {
                    Command::new(yay)
                        .spawn()?
                        .wait()?
                        .report("System upgrade", &mut reports);
                } else {
                    if let Ok(sudo) = &sudo {
                        Command::new(&sudo)
                            .args(&["pacman", "-Syu"])
                            .spawn()?
                            .wait()?
                            .report("System upgrade", &mut reports);
                    } else {
                        terminal.print_warning("No sudo or yay detected. Skipping system upgrade");
                    }
                }
            }

            OSType::CentOS | OSType::Redhat => {
                if let Ok(sudo) = &sudo {
                    Command::new(&sudo)
                        .args(&["yum", "upgrade"])
                        .spawn()?
                        .wait()?
                        .report("System upgrade", &mut reports);;
                }
            }

            OSType::Ubuntu | OSType::Debian => {
                if let Ok(sudo) = &sudo {
                    Command::new(&sudo)
                        .args(&["apt", "update"])
                        .spawn()?
                        .wait()?
                        .check()
                        .and_then(|()| {
                            Command::new(&sudo)
                                .args(&["apt", "dist-upgrade"])
                                .spawn()?
                                .wait()
                                .map_err(Error::from)
                        })?
                        .report("System upgrade", &mut reports);;
                }
            }

            OSType::Unknown => {
                terminal.print_warning(
                    "Could not detect your Linux distribution. Do you have lsb-release installed?",
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
                .check()
                .and_then(|()| {
                    Command::new(&fwupdmgr)
                        .arg("get-updates")
                        .spawn()?
                        .wait()
                        .map_err(Error::from)
                })?
                .report("Firmware upgrade", &mut reports);
        }

        if let Ok(sudo) = &sudo {
            if let Ok(needrestart) = which("needrestart") {
                terminal.print_separator("Check for needed restarts");
                Command::new(&sudo)
                    .arg(&needrestart)
                    .spawn()?
                    .wait()?
                    .report("Restarts", &mut reports);
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
                .check()
                .and_then(|()| {
                    Command::new(&brew)
                        .arg("upgrade")
                        .spawn()?
                        .wait()
                        .map_err(Error::from)
                })?
                .report("Homebrew", &mut reports);
        }

        terminal.print_separator("System update");
        Command::new("softwareupdate")
            .args(&["--install", "--all"])
            .spawn()?
            .wait()?
            .report("System upgrade", &mut reports);;
    }

    let mut reports: Vec<_> = reports.into_iter().collect();
    reports.sort();

    if !reports.is_empty() {
        terminal.print_separator("Summary");

        for (key, succeeded) in reports {
            terminal.print_result(key, succeeded);
        }
    }

    Ok(())
}
