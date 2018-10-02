extern crate directories;
extern crate failure;
extern crate which;
#[macro_use]
extern crate failure_derive;
extern crate toml;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate structopt;
extern crate serde;
extern crate shellexpand;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate term_size;
extern crate termcolor;
extern crate walkdir;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(unix)]
mod tmux;
#[cfg(unix)]
mod unix;
#[cfg(target_os = "windows")]
mod windows;

mod config;
mod executor;
mod generic;
mod git;
mod node;
mod report;
mod terminal;
mod utils;
mod vim;

use self::config::Config;
use self::git::{Git, Repositories};
use self::report::Report;
use self::terminal::Terminal;
use failure::Error;
use std::borrow::Cow;
use std::env;
use std::process::exit;
use structopt::StructOpt;

#[derive(Fail, Debug)]
#[fail(display = "A step failed")]
struct StepFailed;

#[derive(Fail, Debug)]
#[fail(display = "Cannot find the user base directories")]
struct NoBaseDirectories;

fn execute<'a, F, M>(func: F, terminal: &mut Terminal) -> Option<(M, bool)>
where
    M: Into<Cow<'a, str>>,
    F: Fn(&mut Terminal) -> Option<(M, bool)>,
{
    while let Some((key, success)) = func(terminal) {
        if success {
            return Some((key, success));
        }

        if !terminal.should_retry() {
            return Some((key, success));
        }
    }
    None
}

fn run() -> Result<(), Error> {
    let opt = config::Opt::from_args();

    if opt.run_in_tmux && env::var("TMUX").is_err() {
        #[cfg(unix)]
        {
            tmux::run_in_tmux();
        }
    }

    env_logger::init();
    let base_dirs = directories::BaseDirs::new().ok_or(NoBaseDirectories)?;
    let git = Git::new();
    let mut git_repos = Repositories::new(&git);
    let mut terminal = Terminal::new();
    let config = Config::read(&base_dirs)?;
    let mut report = Report::new();

    #[cfg(target_os = "linux")]
    let sudo = utils::which("sudo");

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, &mut terminal, opt.dry_run)?;
        }
    }

    #[cfg(windows)]
    let powershell = windows::Powershell::new();

    #[cfg(windows)]
    report.push_result(execute(
        |terminal| powershell.update_modules(terminal, opt.dry_run),
        &mut terminal,
    ));

    #[cfg(target_os = "linux")]
    let distribution = linux::Distribution::detect();

    #[cfg(target_os = "linux")]
    {
        if !opt.no_system {
            match &distribution {
                Ok(distribution) => {
                    report.push_result(execute(
                        |terminal| distribution.upgrade(&sudo, terminal, opt.dry_run),
                        &mut terminal,
                    ));
                }
                Err(e) => {
                    println!("Error detecting current distribution: {}", e);
                }
            }
        }
    }

    #[cfg(windows)]
    report.push_result(execute(
        |terminal| windows::run_chocolatey(terminal, opt.dry_run),
        &mut terminal,
    ));

    #[cfg(unix)]
    report.push_result(execute(
        |terminal| unix::run_homebrew(terminal, opt.dry_run),
        &mut terminal,
    ));

    if !opt.no_emacs {
        git_repos.insert(base_dirs.home_dir().join(".emacs.d"));
    }

    git_repos.insert(base_dirs.home_dir().join(".vim"));
    git_repos.insert(base_dirs.home_dir().join(".config/nvim"));

    #[cfg(unix)]
    {
        git_repos.insert(base_dirs.home_dir().join(".zshrc"));
        git_repos.insert(base_dirs.home_dir().join(".oh-my-zsh"));
        git_repos.insert(base_dirs.home_dir().join(".tmux"));
        git_repos.insert(base_dirs.home_dir().join(".config/fish"));
        git_repos.insert(base_dirs.config_dir().join("openbox"));
    }

    #[cfg(windows)]
    {
        if let Some(profile) = powershell.profile() {
            git_repos.insert(profile);
        }
    }

    if !opt.no_git_repos {
        if let Some(custom_git_repos) = config.git_repos() {
            for git_repo in custom_git_repos {
                git_repos.insert(git_repo);
            }
        }
    }
    for repo in git_repos.repositories() {
        report.push_result(execute(
            |terminal| git.pull(&repo, terminal, opt.dry_run),
            &mut terminal,
        ));
    }

    #[cfg(unix)]
    {
        report.push_result(execute(
            |terminal| unix::run_zplug(&base_dirs, terminal, opt.dry_run),
            &mut terminal,
        ));
        report.push_result(execute(
            |terminal| unix::run_fisherman(&base_dirs, terminal, opt.dry_run),
            &mut terminal,
        ));
        report.push_result(execute(
            |terminal| tmux::run_tpm(&base_dirs, terminal, opt.dry_run),
            &mut terminal,
        ));
    }

    report.push_result(execute(
        |terminal| generic::run_rustup(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| generic::run_cargo_update(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));

    if !opt.no_emacs {
        report.push_result(execute(
            |terminal| generic::run_emacs(&base_dirs, terminal, opt.dry_run),
            &mut terminal,
        ));
    }

    report.push_result(execute(
        |terminal| generic::run_opam_update(terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| vim::upgrade_vim(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| vim::upgrade_neovim(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| node::run_npm_upgrade(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| node::yarn_global_update(terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| generic::run_apm(terminal, opt.dry_run),
        &mut terminal,
    ));
    report.push_result(execute(
        |terminal| generic::run_gem(&base_dirs, terminal, opt.dry_run),
        &mut terminal,
    ));

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(
            |terminal| linux::run_flatpak(terminal, opt.dry_run),
            &mut terminal,
        ));
        report.push_result(execute(
            |terminal| linux::run_snap(&sudo, terminal, opt.dry_run),
            &mut terminal,
        ));
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            report.push_result(execute(
                |terminal| {
                    Some((
                        name,
                        generic::run_custom_command(&name, &command, terminal, opt.dry_run).is_ok(),
                    ))
                },
                &mut terminal,
            ));
        }
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(
            |terminal| linux::run_fwupdmgr(terminal, opt.dry_run),
            &mut terminal,
        ));
        report.push_result(execute(
            |terminal| linux::run_needrestart(&sudo, terminal, opt.dry_run),
            &mut terminal,
        ));
    }

    #[cfg(target_os = "macos")]
    {
        if !opt.no_system {
            report.push_result(execute(
                |terminal| macos::upgrade_macos(terminal, opt.dry_run),
                &mut terminal,
            ));
        }
    }

    #[cfg(windows)]
    {
        if !opt.no_system {
            report.push_result(execute(
                |terminal| powershell.windows_update(terminal, opt.dry_run),
                &mut terminal,
            ));
        }
    }

    if !report.data().is_empty() {
        terminal.print_separator("Summary");

        for (key, succeeded) in report.data() {
            terminal.print_result(key, *succeeded);
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(distribution) = &distribution {
                distribution.show_summary();
            }
        }
    }

    if report.data().iter().all(|(_, succeeded)| *succeeded) {
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
