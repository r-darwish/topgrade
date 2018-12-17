#[cfg(target_os = "freebsd")]
mod freebsd;
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
mod ctrlc;
mod error;
mod executor;
mod generic;
mod git;
mod node;
mod report;
#[cfg(feature = "self-update")]
mod self_update;
mod terminal;
mod utils;
mod vim;

use self::config::Config;
use self::error::{Error, ErrorKind};
use self::git::{Git, Repositories};
use self::report::Report;
use self::terminal::*;
use failure::{Fail, ResultExt};
use std::borrow::Cow;
use std::env;
use std::io;
use std::process::exit;
use structopt::StructOpt;

fn execute<'a, F, M>(func: F, no_retry: bool) -> Result<Option<(M, bool)>, Error>
where
    M: Into<Cow<'a, str>>,
    F: Fn() -> Option<(M, bool)>,
{
    while let Some((key, success)) = func() {
        if success {
            return Ok(Some((key, success)));
        }

        let running = ctrlc::running();
        if !running {
            ctrlc::set_running(true);
        }

        let should_ask = !running || !no_retry;
        let should_retry = should_ask && should_retry(running).context(ErrorKind::Retry)?;

        if !should_retry {
            return Ok(Some((key, success)));
        }
    }

    Ok(None)
}

fn run() -> Result<(), Error> {
    ctrlc::set_handler();

    let opt = config::Opt::from_args();

    if opt.run_in_tmux && env::var("TMUX").is_err() {
        #[cfg(unix)]
        {
            tmux::run_in_tmux();
        }
    }

    env_logger::init();

    let base_dirs = directories::BaseDirs::new().ok_or(ErrorKind::NoBaseDirectories)?;
    let git = Git::new();
    let mut git_repos = Repositories::new(&git);

    let config = Config::read(&base_dirs)?;
    let mut report = Report::new();

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let sudo = utils::which("sudo");

    #[cfg(feature = "self-update")]
    {
        if !opt.dry_run {
            if let Err(e) = self_update::self_update() {
                print_warning(format!("Self update error: {}", e));
            }
        }
    }

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, opt.dry_run).context(ErrorKind::PreCommand)?;
        }
    }

    #[cfg(windows)]
    let powershell = windows::Powershell::new();

    #[cfg(windows)]
    report.push_result(execute(|| powershell.update_modules(opt.dry_run), opt.no_retry)?);

    #[cfg(target_os = "linux")]
    let distribution = linux::Distribution::detect();

    #[cfg(target_os = "linux")]
    {
        if !opt.no_system {
            match &distribution {
                Ok(distribution) => {
                    report.push_result(
                        execute(|| distribution.upgrade(&sudo, opt.cleanup, opt.dry_run), opt.no_retry)?
                    );
                }
                Err(e) => {
                    println!("Error detecting current distribution: {}", e);
                }
            }
            report.push_result(execute(|| linux::run_etc_update(&sudo, opt.dry_run), opt.no_retry)?);
        }
    }

    #[cfg(windows)]
    report.push_result(execute(|| windows::run_chocolatey(opt.dry_run), opt.no_retry)?);

    #[cfg(windows)]
    report.push_result(execute(|| windows::run_scoop(opt.dry_run), opt.no_retry)?);

    #[cfg(unix)]
    report.push_result(execute(|| unix::run_homebrew(opt.cleanup, opt.dry_run), opt.no_retry)?);
    #[cfg(target_os = "freebsd")]
    report.push_result(execute(|| freebsd::upgrade_packages(&sudo, opt.dry_run), opt.no_retry)?);
    #[cfg(unix)]
    report.push_result(execute(|| unix::run_nix(opt.dry_run), opt.no_retry)?);

    if !opt.no_emacs {
        git_repos.insert(base_dirs.home_dir().join(".emacs.d"));
    }

    if !opt.no_vim {
        git_repos.insert(base_dirs.home_dir().join(".vim"));
        git_repos.insert(base_dirs.home_dir().join(".config/nvim"));
    }

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
        report.push_result(execute(|| git.pull(&repo, opt.dry_run), opt.no_retry)?);
    }

    #[cfg(unix)]
    {
        report.push_result(execute(|| unix::run_zplug(&base_dirs, opt.dry_run), opt.no_retry)?);
        report.push_result(execute(|| unix::run_fisher(&base_dirs, opt.dry_run), opt.no_retry)?);
        report.push_result(execute(|| tmux::run_tpm(&base_dirs, opt.dry_run), opt.no_retry)?);
    }

    report.push_result(execute(|| generic::run_rustup(&base_dirs, opt.dry_run), opt.no_retry)?);
    report.push_result(execute(|| generic::run_cargo_update(opt.dry_run), opt.no_retry)?);

    if !opt.no_emacs {
        report.push_result(execute(|| generic::run_emacs(&base_dirs, opt.dry_run), opt.no_retry)?);
    }

    report.push_result(execute(|| generic::run_opam_update(opt.dry_run), opt.no_retry)?);
    report.push_result(execute(|| generic::run_vcpkg_update(opt.dry_run), opt.no_retry)?);
    report.push_result(execute(|| generic::run_pipx_update(opt.dry_run), opt.no_retry)?);
    report.push_result(execute(|| generic::run_jetpack(opt.dry_run), opt.no_retry)?);

    if !opt.no_vim {
        report.push_result(execute(|| vim::upgrade_vim(&base_dirs, opt.dry_run), opt.no_retry)?);
        report.push_result(execute(|| vim::upgrade_neovim(&base_dirs, opt.dry_run), opt.no_retry)?);
    }

    report.push_result(execute(
        || node::run_npm_upgrade(&base_dirs, opt.dry_run),
        opt.no_retry,
    )?);
    report.push_result(execute(
        || generic::run_composer_update(&base_dirs, opt.dry_run),
        opt.no_retry,
    )?);
    report.push_result(execute(|| node::yarn_global_update(opt.dry_run), opt.no_retry)?);

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    report.push_result(execute(|| generic::run_apm(opt.dry_run), opt.no_retry)?);
    report.push_result(execute(|| generic::run_gem(&base_dirs, opt.dry_run), opt.no_retry)?);

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(|| linux::flatpak_user_update(opt.dry_run), opt.no_retry)?);
        report.push_result(execute(
            || linux::flatpak_global_update(&sudo, opt.dry_run),
            opt.no_retry,
        )?);
        report.push_result(execute(|| linux::run_snap(&sudo, opt.dry_run), opt.no_retry)?);
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            report.push_result(execute(
                || Some((name, generic::run_custom_command(&name, &command, opt.dry_run).is_ok())),
                opt.no_retry,
            )?);
        }
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(|| linux::run_fwupdmgr(opt.dry_run), opt.no_retry)?);
        report.push_result(execute(|| linux::run_needrestart(&sudo, opt.dry_run), opt.no_retry)?);
    }

    #[cfg(target_os = "macos")]
    {
        if !opt.no_system {
            report.push_result(execute(|| macos::upgrade_macos(opt.dry_run), opt.no_retry)?);
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if !opt.no_system {
            report.push_result(execute(|| freebsd::upgrade_freebsd(&sudo, opt.dry_run), opt.no_retry)?);
        }
    }

    #[cfg(windows)]
    {
        if !opt.no_system {
            report.push_result(execute(|| powershell.windows_update(opt.dry_run), opt.no_retry)?);
        }
    }

    if !report.data().is_empty() {
        print_separator("Summary");

        for (key, succeeded) in report.data() {
            print_result(key, *succeeded);
        }

        #[cfg(target_os = "linux")]
        {
            if let Ok(distribution) = &distribution {
                distribution.show_summary();
            }
        }

        #[cfg(target_os = "freebsd")]
        freebsd::audit_packages(&sudo).ok();
    }

    if report.data().iter().all(|(_, succeeded)| *succeeded) {
        Ok(())
    } else {
        Err(ErrorKind::StepFailed)?
    }
}

fn main() {
    match run() {
        Ok(()) => {
            exit(0);
        }
        Err(error) => {
            let should_print = match error.kind() {
                ErrorKind::StepFailed => false,
                ErrorKind::Retry => error
                    .cause()
                    .and_then(|cause| cause.downcast_ref::<io::Error>())
                    .filter(|io_error| io_error.kind() == io::ErrorKind::Interrupted)
                    .is_none(),
                _ => true,
            };

            if should_print {
                println!("Error: {}", error);
                if let Some(cause) = error.cause() {
                    println!("Caused by: {}", cause);
                }
            }
            exit(1);
        }
    }
}
