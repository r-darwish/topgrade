#![allow(clippy::cognitive_complexity)]
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
use env_logger::Env;
use failure::{Fail, ResultExt};
use log::debug;
#[cfg(feature = "self-update")]
use openssl_probe;
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::io;
use std::process::exit;

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

    let mut env = Env::default();
    if config.verbose() {
        env = env.filter_or("LOG_LEVEL", "info");
    }
    env_logger::init_from_env(env);

    let git = git::Git::new();
    let mut git_repos = git::Repositories::new(&git);

    let mut report = Report::new();

    #[cfg(any(target_os = "freebsd", target_os = "linux"))]
    let sudo = utils::which("sudo");
    let run_type = executor::RunType::new(config.dry_run());

    #[cfg(feature = "self-update")]
    {
        openssl_probe::init_ssl_cert_env_vars();
        if !run_type.dry() && env::var("TOPGRADE_NO_SELF_UPGRADE").is_err() {
            let result = self_update::self_update();

            #[cfg(windows)]
            {
                let upgraded = match &result {
                    Ok(()) => false,
                    Err(e) => e.upgraded(),
                };
                if upgraded {
                    return result;
                }
            }

            if let Err(e) = result {
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
    let should_run_powershell = powershell.profile().is_some() && config.should_run(Step::Powershell);

    #[cfg(windows)]
    execute(&mut report, "WSL", || windows::run_wsl_topgrade(run_type), true)?;

    if let Some(topgrades) = config.remote_topgrades() {
        for remote_topgrade in topgrades {
            execute(
                &mut report,
                remote_topgrade,
                || generic::run_remote_topgrade(run_type, remote_topgrade),
                config.no_retry(),
            )?;
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
            execute(
                &mut report,
                "etc-update",
                || linux::run_etc_update(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(windows)]
    execute(
        &mut report,
        "Chocolatey",
        || windows::run_chocolatey(run_type),
        config.no_retry(),
    )?;

    #[cfg(windows)]
    execute(&mut report, "Scoop", || windows::run_scoop(run_type), config.no_retry())?;

    #[cfg(unix)]
    execute(
        &mut report,
        "brew",
        || unix::run_homebrew(config.cleanup(), run_type),
        config.no_retry(),
    )?;
    #[cfg(target_os = "freebsd")]
    execute(
        &mut report,
        "FreeBSD Packages",
        || freebsd::upgrade_packages(sudo.as_ref(), run_type),
        config.no_retry(),
    )?;
    #[cfg(unix)]
    execute(&mut report, "nix", || unix::run_nix(run_type), config.no_retry())?;

    let emacs = emacs::Emacs::new(&base_dirs);
    if config.should_run(Step::Emacs) {
        if let Some(directory) = emacs.directory() {
            git_repos.insert(directory);
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
        git_repos.insert(base_dirs.config_dir().join("bspwm"));
        git_repos.insert(base_dirs.config_dir().join("i3"));
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
    execute(
        &mut report,
        "Git repositories",
        || git.multi_pull(&git_repos, run_type),
        config.no_retry(),
    )?;

    #[cfg(windows)]
    {
        if should_run_powershell {
            execute(
                &mut report,
                "Powershell Modules Update",
                || powershell.update_modules(run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(unix)]
    {
        execute(
            &mut report,
            "zplug",
            || unix::run_zplug(&base_dirs, run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "fisher",
            || unix::run_fisher(&base_dirs, run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "tmux",
            || tmux::run_tpm(&base_dirs, run_type),
            config.no_retry(),
        )?;
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
        execute(&mut report, "Emacs", || emacs.upgrade(run_type), config.no_retry())?;
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
        "myrepos",
        || generic::run_myrepos_update(&base_dirs, run_type),
        config.no_retry(),
    )?;
    #[cfg(unix)]
    execute(&mut report, "pearl", || unix::run_pearl(run_type), config.no_retry())?;
    execute(
        &mut report,
        "jetpak",
        || generic::run_jetpack(run_type),
        config.no_retry(),
    )?;

    if config.should_run(Step::Vim) {
        execute(
            &mut report,
            "vim",
            || vim::upgrade_vim(&base_dirs, run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "Neovim",
            || vim::upgrade_neovim(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    execute(
        &mut report,
        "NPM",
        || node::run_npm_upgrade(&base_dirs, run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "composer",
        || generic::run_composer_update(&base_dirs, run_type),
        config.no_retry(),
    )?;
    execute(
        &mut report,
        "yarn",
        || node::yarn_global_update(run_type),
        config.no_retry(),
    )?;

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
        execute(
            &mut report,
            "Flatpak",
            || linux::flatpak_update(run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "snap",
            || linux::run_snap(sudo.as_ref(), run_type),
            config.no_retry(),
        )?;
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            execute(
                &mut report,
                name,
                || generic::run_custom_command(&name, &command, run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        execute(
            &mut report,
            "pihole",
            || linux::run_pihole_update(sudo.as_ref(), run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "rpi-update",
            || linux::run_rpi_update(sudo.as_ref(), run_type),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "Firmware upgrades",
            || linux::run_fwupdmgr(run_type),
            config.no_retry(),
        )?;
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
            execute(
                &mut report,
                "App Store",
                || macos::upgrade_macos(run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if config.should_run(Step::System) {
            execute(
                &mut report,
                "FreeBSD Upgrade",
                || freebsd::upgrade_freebsd(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(windows)]
    {
        if config.should_run(Step::System) {
            execute(
                &mut report,
                "Windows update",
                || powershell.windows_update(run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Sdkman) {
            execute(
                &mut report,
                "SDKMAN!",
                || unix::run_sdkman(&base_dirs, config.cleanup(), run_type),
                config.no_retry(),
            )?;
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
            #[cfg(all(windows, feature = "self-update"))]
            {
                if let ErrorKind::Upgraded(status) = error.kind() {
                    exit(status.code().unwrap());
                }
            }

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
