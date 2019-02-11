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
use log::debug;
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::io;
#[cfg(windows)]
use std::path::PathBuf;
use std::process::exit;

fn execute_legacy<'a, F, M>(func: F, no_retry: bool) -> Result<Option<(M, bool)>, Error>
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

fn execute<'a, F, M>(report: &mut Report<'a>, key: M, func: F, no_retry: bool) -> Result<(), Error>
where
    F: Fn() -> Result<(), Error>,
    M: Into<Cow<'a, str>> + Debug,
{
    debug!("Executing {:?}", key);

    loop {
        match func() {
            Ok(()) => {
                report.push_result(Some((key, true)));
                break;
            }
            Err(ref e) if e.kind() == ErrorKind::SkipStep => {
                break;
            }
            Err(_) => {
                let interrupted = ctrlc::interrupted();
                if interrupted {
                    ctrlc::unset_interrupted();
                }

                let should_ask = interrupted || !no_retry;
                let should_retry = should_ask && should_retry(interrupted).context(ErrorKind::Retry)?;

                if !should_retry {
                    report.push_result(Some((key, false)));
                    break;
                }
            }
        }
    }

    Ok(())
}

fn run() -> Result<(), Error> {
    ctrlc::set_handler();

    let base_dirs = directories::BaseDirs::new().ok_or(ErrorKind::NoBaseDirectories)?;
    let config = Config::load(&base_dirs)?;

    if config.run_in_tmux() && env::var("TMUX").is_err() {
        #[cfg(unix)]
        {
            tmux::run_in_tmux();
        }
    }

    env_logger::init();

    let git = git::Git::new();
    let mut git_repos = git::Repositories::new(&git);

    let mut report = Report::new();

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let sudo = utils::which("sudo");
    let run_type = executor::RunType::new(config.dry_run());

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
    {
        if powershell.profile().is_some() && config.should_run(Step::Powershell) {
            report.push_result(execute_legacy(
                || powershell.update_modules(run_type),
                config.no_retry(),
            )?);
        }
    }

    #[cfg(target_os = "linux")]
    let distribution = linux::Distribution::detect();

    #[cfg(target_os = "linux")]
    {
        if config.should_run(Step::System) {
            match &distribution {
                Ok(distribution) => {
                    execute(
                        &mut report,
                        "System update",
                        || distribution.upgrade(&sudo, config.cleanup(), run_type),
                        config.no_retry(),
                    )?;
                }
                Err(e) => {
                    println!("Error detecting current distribution: {}", e);
                }
            }
            report.push_result(execute_legacy(
                || linux::run_etc_update(&sudo, run_type),
                config.no_retry(),
            )?);
        }
    }

    #[cfg(windows)]
    report.push_result(execute_legacy(|| windows::run_chocolatey(run_type), config.no_retry())?);

    #[cfg(windows)]
    report.push_result(execute_legacy(|| windows::run_scoop(run_type), config.no_retry())?);

    #[cfg(unix)]
    report.push_result(execute_legacy(
        || unix::run_homebrew(config.cleanup(), run_type),
        config.no_retry(),
    )?);
    #[cfg(target_os = "freebsd")]
    report.push_result(execute_legacy(
        || freebsd::upgrade_packages(&sudo, run_type),
        config.no_retry(),
    )?);
    #[cfg(unix)]
    report.push_result(execute_legacy(|| unix::run_nix(run_type), config.no_retry())?);

    if config.should_run(Step::Emacs) {
        #[cfg(unix)]
        git_repos.insert(base_dirs.home_dir().join(".emacs.d"));

        #[cfg(windows)]
        {
            git_repos.insert(base_dirs.data_dir().join(".emacs.d"));
            if let Ok(home) = env::var("HOME") {
                git_repos.insert(PathBuf::from(home).join(".emacs.d"));
            }
        }
    }

    if config.should_run(Step::Vim) {
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

    if config.should_run(Step::GitRepos) {
        if let Some(custom_git_repos) = config.git_repos() {
            for git_repo in custom_git_repos {
                git_repos.insert(git_repo);
            }
        }
    }
    for repo in git_repos.repositories() {
        report.push_result(execute_legacy(|| git.pull(&repo, run_type), config.no_retry())?);
    }

    #[cfg(unix)]
    {
        report.push_result(execute_legacy(
            || unix::run_zplug(&base_dirs, run_type),
            config.no_retry(),
        )?);
        report.push_result(execute_legacy(
            || unix::run_fisher(&base_dirs, run_type),
            config.no_retry(),
        )?);
        report.push_result(execute_legacy(
            || tmux::run_tpm(&base_dirs, run_type),
            config.no_retry(),
        )?);
    }

    execute(
        &mut report,
        "rustup",
        || generic::run_rustup(&base_dirs, run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "cargo",
        || generic::run_cargo_update(run_type),
        config.no_retry(),
    )?;

    if config.should_run(Step::Emacs) {
        execute(
            &mut report,
            "Emacs",
            || generic::run_emacs(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    execute(
        &mut report,
        "opam",
        || generic::run_opam_update(run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "vcpkg",
        || generic::run_vcpkg_update(run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "pipx",
        || generic::run_pipx_update(run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "jetpak",
        || generic::run_jetpack(run_type),
        config.no_retry(),
    )?;

    if config.should_run(Step::Vim) {
        report.push_result(execute_legacy(
            || vim::upgrade_vim(&base_dirs, run_type),
            config.no_retry(),
        )?);
        report.push_result(execute_legacy(
            || vim::upgrade_neovim(&base_dirs, run_type),
            config.no_retry(),
        )?);
    }

    report.push_result(execute_legacy(
        || node::run_npm_upgrade(&base_dirs, run_type),
        config.no_retry(),
    )?);
    execute(
        &mut report,
        "composer",
        || generic::run_composer_update(&base_dirs, run_type),
        config.no_retry(),
    )?;
    report.push_result(execute_legacy(
        || node::yarn_global_update(run_type),
        config.no_retry(),
    )?);

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    execute(&mut report, "apm", || generic::run_apm(run_type), config.no_retry())?;

    if config.should_run(Step::Gem) {
        execute(
            &mut report,
            "gem",
            || generic::run_gem(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute_legacy(|| linux::flatpak_update(run_type), config.no_retry())?);
        report.push_result(execute_legacy(|| linux::run_snap(&sudo, run_type), config.no_retry())?);
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            report.push_result(execute_legacy(
                || Some((name, generic::run_custom_command(&name, &command, run_type).is_ok())),
                config.no_retry(),
            )?);
        }
    }

    #[cfg(target_os = "linux")]
    {
        report.push_result(execute_legacy(|| linux::run_fwupdmgr(run_type), config.no_retry())?);
        execute(
            &mut report,
            "Restarts",
            || linux::run_needrestart(sudo.as_ref(), run_type),
            config.no_retry(),
        )?;
    }

    #[cfg(target_os = "macos")]
    {
        if config.should_run(Step::System) {
            report.push_result(execute_legacy(|| macos::upgrade_macos(run_type), config.no_retry())?);
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if config.should_run(Step::System) {
            report.push_result(execute_legacy(
                || freebsd::upgrade_freebsd(&sudo, run_type),
                config.no_retry(),
            )?);
        }
    }

    #[cfg(windows)]
    {
        if config.should_run(Step::System) {
            report.push_result(execute_legacy(
                || powershell.windows_update(run_type),
                config.no_retry(),
            )?);
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
