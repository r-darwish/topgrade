extern crate directories;
extern crate failure;
extern crate which;
#[macro_use]
extern crate failure_derive;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate clap;
extern crate serde;
extern crate shellexpand;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate term_size;
extern crate termcolor;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

mod config;
mod git;
mod npm;
mod report;
mod steps;
mod terminal;
mod utils;
mod vim;

use clap::{App, Arg};
use config::Config;
use failure::Error;
use git::{Git, Repositories};
use report::{Report, Reporter};
use std::env;
use std::env::home_dir;
use std::process::exit;
use steps::*;
use terminal::Terminal;
use utils::{home_path, is_ancestor};

#[derive(Fail, Debug)]
#[fail(display = "A step failed")]
struct StepFailed;

fn run() -> Result<(), Error> {
    let matches = App::new("Topgrade")
        .version(crate_version!())
        .about("Upgrade all the things")
        .arg(
            Arg::with_name("tmux")
                .help("Invoke inside tmux")
                .short("t")
                .long("tmux"),
        )
        .get_matches();

    if matches.is_present("tmux") && !env::var("TMUX").is_ok() {
        #[cfg(unix)]
        {
            unix::run_in_tmux();
        }
    }

    env_logger::init();
    let git = Git::new();
    let mut git_repos = Repositories::new(&git);
    let mut terminal = Terminal::new();
    let config = Config::read()?;
    let mut reports = Report::new();

    #[cfg(target_os = "linux")]
    let sudo = utils::which("sudo");

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            terminal.print_separator(name);
            run_custom_command(&command)?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        terminal.print_separator("System update");
        match linux::Distribution::detect() {
            Ok(distribution) => {
                match distribution {
                    linux::Distribution::Arch => linux::upgrade_arch_linux(&sudo, &mut terminal),
                    linux::Distribution::CentOS => linux::upgrade_redhat(&sudo, &mut terminal),
                    linux::Distribution::Fedora => linux::upgrade_fedora(&sudo, &mut terminal),
                    linux::Distribution::Ubuntu | linux::Distribution::Debian => {
                        linux::upgrade_debian(&sudo, &mut terminal)
                    }
                }.report("System upgrade", &mut reports);
            }

            Err(e) => {
                println!("Error detecting current distribution: {}", e);
            }
        }
    }

    if let Some(brew) = utils::which("brew") {
        terminal.print_separator("Brew");
        run_homebrew(&brew).report("Brew", &mut reports);
    }

    #[cfg(windows)]
    {
        if let Some(choco) = utils::which("choco") {
            terminal.print_separator("Chocolatey");
            windows::run_chocolatey(&choco).report("Chocolatey", &mut reports);
        }
    }

    git_repos.insert(home_path(".emacs.d"));

    #[cfg(unix)]
    {
        git_repos.insert(home_path(".zshrc"));
        git_repos.insert(home_path(".oh-my-zsh"));
        git_repos.insert(home_path(".tmux"));
        git_repos.insert(home_path(".config/fish/config.fish"));
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

    #[cfg(unix)]
    {
        if let Some(zsh) = utils::which("zsh") {
            if home_path(".zplug").exists() {
                terminal.print_separator("zplug");
                unix::run_zplug(&zsh).report("zplug", &mut reports);
            }
        }

        if let Some(tpm) = unix::tpm_path() {
            terminal.print_separator("tmux plugins");
            unix::run_tpm(&tpm).report("tmux", &mut reports);
        }
    }

    if let Some(rustup) = utils::which("rustup") {
        terminal.print_separator("rustup");
        run_rustup(&rustup).report("rustup", &mut reports);
    }

    let cargo_upgrade = home_path(".cargo/bin/cargo-install-update");
    if cargo_upgrade.exists() {
        terminal.print_separator("Cargo");
        run_cargo_update(&cargo_upgrade).report("Cargo", &mut reports);
    }

    if let Some(emacs) = utils::which("emacs") {
        let init_file = home_path(".emacs.d/init.el");
        if init_file.exists() {
            terminal.print_separator("Emacs");
            run_emacs(&emacs, &init_file).report("Emacs", &mut reports);
        }
    }

    if let Some(vim) = utils::which("vim") {
        if let Some(vimrc) = vim::vimrc() {
            if let Some(plugin_framework) = vim::PluginFramework::detect(&vimrc) {
                terminal.print_separator(&format!("vim ({:?})", plugin_framework));
                run_vim(&vim, &vimrc, plugin_framework.upgrade_command())
                    .report("Vim", &mut reports);
            }
        }
    }

    if let Some(npm) = utils::which("npm").map(npm::NPM::new) {
        if let Ok(npm_root) = npm.root() {
            if is_ancestor(&home_dir().unwrap(), &npm_root) {
                terminal.print_separator("Node Package Manager");
                npm.upgrade().report("Node Package Manager", &mut reports);
            }
        }
    }

    if let Some(apm) = utils::which("apm") {
        terminal.print_separator("Atom Package Manager");
        run_apm(&apm).report("Atom Package Manager", &mut reports);
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(flatpak) = utils::which("flatpak") {
            terminal.print_separator("Flatpak");
            linux::run_flatpak(&flatpak).report("Flatpak", &mut reports);
        }

        if let Some(sudo) = &sudo {
            if let Some(snap) = utils::which("snap") {
                terminal.print_separator("snap");
                linux::run_snap(&sudo, &snap).report("snap", &mut reports);
            }
        }
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            terminal.print_separator(name);
            run_custom_command(&command).report(name.as_str(), &mut reports);
        }
    }

    #[cfg(target_os = "linux")]
    {
        if let Some(fwupdmgr) = utils::which("fwupdmgr") {
            terminal.print_separator("Firmware upgrades");
            linux::run_fwupdmgr(&fwupdmgr).report("Firmware upgrade", &mut reports);
        }

        if let Some(sudo) = &sudo {
            if let Some(_) = utils::which("needrestart") {
                terminal.print_separator("Check for needed restarts");
                linux::run_needrestart(&sudo).report("Restarts", &mut reports);
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        terminal.print_separator("App Store");
        macos::upgrade_macos().report("App Store", &mut reports);
    }

    if !reports.is_empty() {
        terminal.print_separator("Summary");

        for (key, succeeded) in &reports {
            terminal.print_result(key, *succeeded);
        }
    }

    if reports.iter().all(|(_, succeeded)| *succeeded) {
        Ok(())
    } else {
        Err(StepFailed.into())
    }
}

fn main() {
    match run() {
        Ok(()) => {
            exit(0);
        }
        Err(error) => {
            match error.downcast::<StepFailed>() {
                Ok(_) => (),
                Err(error) => println!("ERROR: {}", error),
            };
            exit(1);
        }
    }
}
