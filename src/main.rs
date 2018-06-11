extern crate directories;
extern crate failure;
extern crate which;
#[macro_use]
extern crate failure_derive;
extern crate termion;
extern crate toml;
#[macro_use]
extern crate serde_derive;
extern crate serde;

mod config;
mod git;
mod linux;
mod npm;
mod report;
mod steps;
mod terminal;
mod vim;

use config::Config;
use failure::Error;
use git::{Git, Repositories};
use report::{Report, Reporter};
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use steps::*;
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

fn home_path(p: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(p);
    path
}

fn is_ancestor(ancestor: &Path, path: &Path) -> bool {
    let mut p = path;
    while let Some(parent) = p.parent() {
        if parent == ancestor {
            return true;
        }

        p = parent;
    }

    false
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
    let mut git_repos = Repositories::new(&git);
    let terminal = Terminal::new();
    let mut reports = Report::new();
    let config = Config::read()?;

    let sudo = if cfg!(target_os = "linux") {
        which("sudo").ok()
    } else {
        None
    };

    if cfg!(target_os = "linux") {
        terminal.print_separator("System update");
        match linux::Distribution::detect() {
            Ok(distribution) => {
                match distribution {
                    linux::Distribution::Arch => upgrade_arch_linux(&sudo, &terminal),
                    linux::Distribution::CentOS => upgrade_redhat(&sudo, &terminal),
                    linux::Distribution::Fedora => upgrade_fedora(&sudo, &terminal),
                    linux::Distribution::Ubuntu | linux::Distribution::Debian => {
                        upgrade_debian(&sudo, &terminal)
                    }
                }.report("System upgrade", &mut reports);
            }

            Err(e) => {
                println!("Error detecting current distribution: {}", e);
            }
        }
    }

    if cfg!(target_os = "macos") {
        if let Ok(brew) = which("brew") {
            terminal.print_separator("Homebrew");
            run_homebrew(&brew).report("Homebrew", &mut reports);
        }
    }

    git_repos.insert(home_path(".emacs.d"));

    if cfg!(unix) {
        git_repos.insert(home_path(".zshrc"));
        git_repos.insert(home_path(".oh-my-zsh"));
        git_repos.insert(home_path(".tmux"));
    }

    if let Some(custom_git_repos) = config.git_repos() {
        for git_repo in custom_git_repos {
            git_repos.insert(git_repo);
        }
    }

    for repo in git_repos.repositories() {
        terminal.print_separator(format!("Pulling {}", repo));
        if let Some(success) = git.pull(&repo).ok().and_then(|i| i) {
            success.report(format!("git: {}", repo), &mut reports);
        }
    }

    if cfg!(unix) {
        if let Ok(zsh) = which("zsh") {
            if home_path(".zplug").exists() {
                terminal.print_separator("zplug");
                run_zplug(&zsh).report("zplug", &mut reports);
            }
        }

        if let Some(tpm) = tpm() {
            terminal.print_separator("tmux plugins");
            run_tpm(&tpm).report("tmux", &mut reports);
        }
    }

    if let Ok(rustup) = which("rustup") {
        terminal.print_separator("rustup");
        run_rustup(&rustup).report("rustup", &mut reports);
    }

    let cargo_upgrade = home_path(".cargo/bin/cargo-install-update");
    if cargo_upgrade.exists() {
        terminal.print_separator("Cargo");
        run_cargo_update(&cargo_upgrade).report("Cargo", &mut reports);
    }

    if let Ok(emacs) = which("emacs") {
        let init_file = home_path(".emacs.d/init.el");
        if init_file.exists() {
            terminal.print_separator("Emacs");
            run_emacs(&emacs, &init_file).report("Emacs", &mut reports);
        }
    }

    if let Ok(vim) = which("vim") {
        if let Some(vimrc) = vim::vimrc() {
            if let Some(plugin_framework) = vim::PluginFramework::detect(&vimrc) {
                terminal.print_separator(&format!("vim ({:?})", plugin_framework));
                run_vim(&vim, &vimrc, plugin_framework.upgrade_command())
                    .report("Vim", &mut reports);
            }
        }
    }

    if let Ok(npm) = which("npm").map(npm::NPM::new) {
        if let Ok(npm_root) = npm.root() {
            if is_ancestor(&home_dir().unwrap(), &npm_root) {
                terminal.print_separator("Node Package Manager");
                npm.upgrade().report("Node Package Manager", &mut reports);
            }
        }
    }

    if let Ok(apm) = which("apm") {
        terminal.print_separator("Atom Package Manager");
        run_apm(&apm).report("Atom Package Manager", &mut reports);
    }

    if cfg!(target_os = "linux") {
        if let Ok(fwupdmgr) = which("fwupdmgr") {
            terminal.print_separator("Firmware upgrades");
            run_fwupdmgr(&fwupdmgr).report("Firmware upgrade", &mut reports);
        }

        if let Some(sudo) = &sudo {
            if let Ok(_) = which("needrestart") {
                terminal.print_separator("Check for needed restarts");
                run_needrestart(&sudo).report("Restarts", &mut reports);
            }
        }
    }

    if cfg!(target_os = "macos") {
        terminal.print_separator("App Store");
        upgrade_macos().report("App Store", &mut reports);;
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            terminal.print_separator(name);
            run_custom_command(&command).report(name.as_ref(), &mut reports);
        }
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
