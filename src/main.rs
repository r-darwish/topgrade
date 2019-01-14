mod config;
mod ctrlc;
mod error;
mod executor;
mod report;
#[cfg(feature = "self-update")]
mod self_update;
mod steps;
mod terminal;
mod utils;

use self::config::{Config, Step};
use self::error::{Error, ErrorKind};
use self::report::Report;
use self::steps::*;
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

        let interrupted = ctrlc::interrupted();
        if interrupted {
            ctrlc::unset_interrupted();
        }

        let should_ask = interrupted || !no_retry;
        let should_retry = should_ask && should_retry(interrupted).context(ErrorKind::Retry)?;

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
    let git = git::Git::new();
    let mut git_repos = git::Repositories::new(&git);

    let config = Config::read(&base_dirs)?;
    let mut report = Report::new();

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let sudo = utils::which("sudo");
    let run_type = executor::RunType::new(opt.dry_run);

    #[cfg(feature = "self-update")]
    {
        if !run_type.dry() && env::var("TOPGRADE_NO_SELF_UPGRADE").is_err() {
            if let Err(e) = self_update::self_update() {
                print_warning(format!("Self update error: {}", e));
                if let Some(cause) = e.cause() {
                    print_warning(format!("Caused by: {}", cause));
                }
            }
        }
    }

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, run_type).context(ErrorKind::PreCommand)?;
        }
    }

    #[cfg(windows)]
    let powershell = windows::Powershell::new();

    #[cfg(windows)]
    report.push_result(execute(|| powershell.update_modules(run_type), opt.no_retry)?);

    #[cfg(target_os = "linux")]
    let distribution = linux::Distribution::detect();

    #[cfg(target_os = "linux")]
    {
        if !opt.disable.contains(&Step::System) {
            match &distribution {
                Ok(distribution) => {
                    report.push_result(execute(
                        || distribution.upgrade(&sudo, opt.cleanup, run_type),
                        opt.no_retry,
                    )?);
                }
                Err(e) => {
                    println!("Error detecting current distribution: {}", e);
                }
            }
            report.push_result(execute(|| linux::run_etc_update(&sudo, run_type), opt.no_retry)?);
        }
    }

    #[cfg(windows)]
    report.push_result(execute(|| windows::run_chocolatey(run_type), opt.no_retry)?);

    #[cfg(windows)]
    report.push_result(execute(|| windows::run_scoop(run_type), opt.no_retry)?);

    #[cfg(unix)]
    report.push_result(execute(|| unix::run_homebrew(opt.cleanup, run_type), opt.no_retry)?);
    #[cfg(target_os = "freebsd")]
    report.push_result(execute(|| freebsd::upgrade_packages(&sudo, run_type), opt.no_retry)?);
    #[cfg(unix)]
    report.push_result(execute(|| unix::run_nix(run_type), opt.no_retry)?);

    if !opt.disable.contains(&Step::Emacs) {
        git_repos.insert(base_dirs.home_dir().join(".emacs.d"));
    }

    if !opt.disable.contains(&Step::Vim) {
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

    if !opt.disable.contains(&Step::GitRepos) {
        if let Some(custom_git_repos) = config.git_repos() {
            for git_repo in custom_git_repos {
                git_repos.insert(git_repo);
            }
        }
    }
    for repo in git_repos.repositories() {
        report.push_result(execute(|| git.pull(&repo, run_type), opt.no_retry)?);
    }

    #[cfg(unix)]
    {
        report.push_result(execute(|| unix::run_zplug(&base_dirs, run_type), opt.no_retry)?);
        report.push_result(execute(|| unix::run_fisher(&base_dirs, run_type), opt.no_retry)?);
        report.push_result(execute(|| tmux::run_tpm(&base_dirs, run_type), opt.no_retry)?);
    }

    report.push_result(execute(|| generic::run_rustup(&base_dirs, run_type), opt.no_retry)?);
    report.push_result(execute(|| generic::run_cargo_update(run_type), opt.no_retry)?);

    if !opt.disable.contains(&Step::Emacs) {
        report.push_result(execute(|| generic::run_emacs(&base_dirs, run_type), opt.no_retry)?);
    }

    report.push_result(execute(|| generic::run_opam_update(run_type), opt.no_retry)?);
    report.push_result(execute(|| generic::run_vcpkg_update(run_type), opt.no_retry)?);
    report.push_result(execute(|| generic::run_pipx_update(run_type), opt.no_retry)?);
    report.push_result(execute(|| generic::run_jetpack(run_type), opt.no_retry)?);

    if !opt.disable.contains(&Step::Vim) {
        report.push_result(execute(|| vim::upgrade_vim(&base_dirs, run_type), opt.no_retry)?);
        report.push_result(execute(|| vim::upgrade_neovim(&base_dirs, run_type), opt.no_retry)?);
    }

    report.push_result(execute(|| node::run_npm_upgrade(&base_dirs, run_type), opt.no_retry)?);
    report.push_result(execute(
        || generic::run_composer_update(&base_dirs, run_type),
        opt.no_retry,
    )?);
    report.push_result(execute(|| node::yarn_global_update(run_type), opt.no_retry)?);

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    report.push_result(execute(|| generic::run_apm(run_type), opt.no_retry)?);

    if !opt.disable.contains(&Step::Gem) {
        report.push_result(execute(|| generic::run_gem(&base_dirs, run_type), opt.no_retry)?);
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(|| linux::flatpak_user_update(run_type), opt.no_retry)?);
        report.push_result(execute(|| linux::flatpak_global_update(&sudo, run_type), opt.no_retry)?);
        report.push_result(execute(|| linux::run_snap(&sudo, run_type), opt.no_retry)?);
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            report.push_result(execute(
                || Some((name, generic::run_custom_command(&name, &command, run_type).is_ok())),
                opt.no_retry,
            )?);
        }
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute(|| linux::run_fwupdmgr(run_type), opt.no_retry)?);
        report.push_result(execute(|| linux::run_needrestart(&sudo, run_type), opt.no_retry)?);
    }

    #[cfg(target_os = "macos")]
    {
        if !opt.disable.contains(&Step::System) {
            report.push_result(execute(|| macos::upgrade_macos(run_type), opt.no_retry)?);
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if !opt.disable.contains(&Step::System) {
            report.push_result(execute(|| freebsd::upgrade_freebsd(&sudo, run_type), opt.no_retry)?);
        }
    }

    #[cfg(windows)]
    {
        if !opt.disable.contains(&Step::System) {
            report.push_result(execute(|| powershell.windows_update(run_type), opt.no_retry)?);
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
