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
mod generic;
mod git;
mod node;
mod report;
mod terminal;
mod utils;
mod vim;

use self::config::Config;
use self::git::{Git, Repositories};
use self::report::{report, Report};
use self::terminal::Terminal;
use clap::{App, Arg};
use failure::Error;
use std::env;
use std::process::exit;

#[derive(Fail, Debug)]
#[fail(display = "A step failed")]
struct StepFailed;

#[derive(Fail, Debug)]
#[fail(display = "Cannot find the user base directories")]
struct NoBaseDirectories;

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
        .arg(
            Arg::with_name("no_system")
                .help("Don't perform system upgrade")
                .long("no-system"),
        )
        .get_matches();

    if matches.is_present("tmux") && env::var("TMUX").is_err() {
        #[cfg(unix)]
        {
            unix::run_in_tmux();
        }
    }

    env_logger::init();
    let base_dirs = directories::BaseDirs::new().ok_or(NoBaseDirectories)?;
    let git = Git::new();
    let mut git_repos = Repositories::new(&git);
    let mut terminal = Terminal::new();
    let config = Config::read(&base_dirs)?;
    let mut reports = Report::new();

    #[cfg(target_os = "linux")]
    let sudo = utils::which("sudo");

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, &mut terminal)?;
        }
    }

    if !(matches.is_present("no_system")) {
        #[cfg(target_os = "linux")]
        report(&mut reports, linux::upgrade(&sudo, &mut terminal));

        #[cfg(windows)]
        report(&mut reports, windows::run_chocolatey(&mut terminal));
    }

    #[cfg(unix)]
    report(&mut reports, unix::run_homebrew(&mut terminal));

    git_repos.insert(base_dirs.home_dir().join(".emacs.d"));
    git_repos.insert(base_dirs.home_dir().join(".vim"));
    git_repos.insert(base_dirs.home_dir().join(".config/nvim"));

    #[cfg(unix)]
    {
        git_repos.insert(base_dirs.home_dir().join(".zshrc"));
        git_repos.insert(base_dirs.home_dir().join(".oh-my-zsh"));
        git_repos.insert(base_dirs.home_dir().join(".tmux"));
        git_repos.insert(base_dirs.home_dir().join(".config/fish"));
    }

    if let Some(custom_git_repos) = config.git_repos() {
        for git_repo in custom_git_repos {
            git_repos.insert(git_repo);
        }
    }

    for repo in git_repos.repositories() {
        report(&mut reports, git.pull(&repo, &mut terminal));
    }

    #[cfg(unix)]
    {
        report(&mut reports, unix::run_zplug(&base_dirs, &mut terminal));
        report(&mut reports, unix::run_fisherman(&base_dirs, &mut terminal));
        report(&mut reports, unix::run_tpm(&base_dirs, &mut terminal));
    }

    report(&mut reports, generic::run_rustup(&base_dirs, &mut terminal));
    report(&mut reports, generic::run_cargo_update(&base_dirs, &mut terminal));
    report(&mut reports, generic::run_emacs(&base_dirs, &mut terminal));
    report(&mut reports, vim::upgrade_vim(&base_dirs, &mut terminal));
    report(&mut reports, vim::upgrade_neovim(&base_dirs, &mut terminal));
    report(&mut reports, node::run_npm_upgrade(&base_dirs, &mut terminal));
    report(&mut reports, node::yarn_global_update(&mut terminal));
    report(&mut reports, generic::run_apm(&mut terminal));

    #[cfg(target_os = "linux")]
    {
        report(&mut reports, linux::run_flatpak(&mut terminal));
        report(&mut reports, linux::run_snap(&sudo, &mut terminal));
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            report(
                &mut reports,
                Some((
                    name,
                    generic::run_custom_command(&name, &command, &mut terminal).is_ok(),
                )),
            );
        }
    }

    #[cfg(target_os = "linux")]
    {
        report(&mut reports, linux::run_fwupdmgr(&mut terminal));
        report(&mut reports, linux::run_needrestart(&sudo, &mut terminal));
    }

    #[cfg(target_os = "macos")]
    {
        if !(matches.is_present("no_system")) {
            macos::upgrade_macos(&mut terminal).report("App Store", &mut reports);
        }
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
