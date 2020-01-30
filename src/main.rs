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

use self::config::{CommandLineArgs, Config, Step};
#[cfg(all(windows, feature = "self-update"))]
use self::error::Upgraded;
use self::error::{SkipStep, StepFailed};
use self::report::Report;
use self::steps::*;
use self::terminal::*;
use anyhow::{anyhow, Result};
use log::debug;
#[cfg(feature = "self-update")]
use openssl_probe;
use std::borrow::Cow;
use std::env;
use std::fmt::Debug;
use std::io;
use std::process::exit;
use structopt::StructOpt;

fn execute<'a, F, M>(report: &mut Report<'a>, key: M, func: F, no_retry: bool) -> Result<()>
where
    F: Fn() -> Result<()>,
    M: Into<Cow<'a, str>> + Debug,
{
    let key = key.into();
    debug!("Step {:?}", key);

    loop {
        match func() {
            Ok(()) => {
                report.push_result(Some((key, true)));
                break;
            }
            Err(e) if e.downcast_ref::<SkipStep>().is_some() => {
                break;
            }
            Err(_) => {
                let interrupted = ctrlc::interrupted();
                if interrupted {
                    ctrlc::unset_interrupted();
                }

                let should_ask = interrupted || !no_retry;
                let should_retry = should_ask && should_retry(interrupted, key.as_ref())?;

                if !should_retry {
                    report.push_result(Some((key, false)));
                    break;
                }
            }
        }
    }

    Ok(())
}

fn run() -> Result<()> {
    ctrlc::set_handler();

    let base_dirs = directories::BaseDirs::new().ok_or_else(|| anyhow!("No base directories"))?;

    let opt = CommandLineArgs::from_args();
    if opt.edit_config() {
        Config::edit(&base_dirs)?;
        return Ok(());
    };

    let config = Config::load(&base_dirs, opt)?;
    terminal::set_title(config.set_title());

    if config.run_in_tmux() && env::var("TOPGRADE_INSIDE_TMUX").is_err() {
        #[cfg(unix)]
        {
            tmux::run_in_tmux(config.tmux_arguments());
        }
    }

    let git = git::Git::new();
    let mut git_repos = git::Repositories::new(&git);

    let mut report = Report::new();

    #[cfg(unix)]
    let sudo = utils::sudo();
    let run_type = executor::RunType::new(config.dry_run());

    #[cfg(feature = "self-update")]
    {
        openssl_probe::init_ssl_cert_env_vars();
        if !run_type.dry() && env::var("TOPGRADE_NO_SELF_UPGRADE").is_err() {
            let result = self_update::self_update();

            if let Err(e) = &result {
                #[cfg(windows)]
                {
                    if e.downcast_ref::<Upgraded>().is_some() {
                        return result;
                    }
                }
                print_warning(format!("Self update error: {}", e));
            }
        }
    }

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, run_type)?;
        }
    }

    let powershell = powershell::Powershell::new();
    let should_run_powershell = powershell.profile().is_some() && config.should_run(Step::Shell);

    #[cfg(windows)]
    execute(&mut report, "WSL", || windows::run_wsl_topgrade(run_type), true)?;

    if let Some(topgrades) = config.remote_topgrades() {
        if config.should_run(Step::Remotes) {
            for remote_topgrade in topgrades {
                execute(
                    &mut report,
                    remote_topgrade,
                    || {
                        generic::run_remote_topgrade(
                            run_type,
                            remote_topgrade,
                            config.ssh_arguments(),
                            config.run_in_tmux(),
                            config.tmux_arguments(),
                        )
                    },
                    config.no_retry(),
                )?;
            }
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
                        || distribution.upgrade(&sudo, run_type, &config),
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
    {
        if config.should_run(Step::PackageManagers) {
            execute(
                &mut report,
                "Chocolatey",
                || windows::run_chocolatey(run_type),
                config.no_retry(),
            )?;

            execute(
                &mut report,
                "Scoop",
                || windows::run_scoop(config.cleanup(), run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::PackageManagers) {
            execute(
                &mut report,
                "brew",
                || unix::run_homebrew(config.cleanup(), run_type),
                config.no_retry(),
            )?;

            execute(&mut report, "nix", || unix::run_nix(run_type), config.no_retry())?;
            execute(
                &mut report,
                "home-manager",
                || unix::run_home_manager(run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(target_os = "dragonfly")]
    {
        if config.should_run(Step::PackageManagers) {
            execute(
                &mut report,
                "DragonFly BSD Packages",
                || dragonfly::upgrade_packages(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if config.should_run(Step::PackageManagers) {
            execute(
                &mut report,
                "FreeBSD Packages",
                || freebsd::upgrade_packages(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }
    }

    let emacs = emacs::Emacs::new(&base_dirs);
    if config.use_predefined_git_repos() {
        if config.should_run(Step::Emacs) {
            if !emacs.is_doom() {
                if let Some(directory) = emacs.directory() {
                    git_repos.insert(directory);
                }
            }
            git_repos.insert(base_dirs.home_dir().join(".doom.d"));
        }

        if config.should_run(Step::Vim) {
            git_repos.insert(base_dirs.home_dir().join(".vim"));
            git_repos.insert(base_dirs.home_dir().join(".config/nvim"));
        }

        #[cfg(unix)]
        {
            git_repos.insert(zsh::zshrc(&base_dirs));
            git_repos.insert(base_dirs.home_dir().join(".tmux"));
            git_repos.insert(base_dirs.home_dir().join(".config/fish"));
            git_repos.insert(base_dirs.config_dir().join("openbox"));
            git_repos.insert(base_dirs.config_dir().join("bspwm"));
            git_repos.insert(base_dirs.config_dir().join("i3"));
            git_repos.insert(base_dirs.config_dir().join("sway"));
        }

        #[cfg(windows)]
        git_repos.insert(
            base_dirs
                .data_local_dir()
                .join("Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState"),
        );

        if let Some(profile) = powershell.profile() {
            git_repos.insert(profile);
        }
    }

    if config.should_run(Step::GitRepos) {
        if let Some(custom_git_repos) = config.git_repos() {
            for git_repo in custom_git_repos {
                git_repos.glob_insert(git_repo);
            }
        }
        execute(
            &mut report,
            "Git repositories",
            || git.multi_pull(&git_repos, run_type, config.git_arguments()),
            config.no_retry(),
        )?;
    }

    if should_run_powershell {
        execute(
            &mut report,
            "Powershell Modules Update",
            || powershell.update_modules(run_type),
            config.no_retry(),
        )?;
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Shell) {
            execute(
                &mut report,
                "zr",
                || zsh::run_zr(&base_dirs, run_type),
                config.no_retry(),
            )?;
            execute(
                &mut report,
                "antigen",
                || zsh::run_antigen(&base_dirs, run_type),
                config.no_retry(),
            )?;
            execute(
                &mut report,
                "zplug",
                || zsh::run_zplug(&base_dirs, run_type),
                config.no_retry(),
            )?;
            execute(
                &mut report,
                "zplugin",
                || zsh::run_zplugin(&base_dirs, run_type),
                config.no_retry(),
            )?;
            execute(
                &mut report,
                "oh-my-zsh",
                || zsh::run_oh_my_zsh(&base_dirs, run_type),
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

        if config.should_run(Step::Tldr) {
            execute(&mut report, "TLDR", || unix::run_tldr(run_type), config.no_retry())?;
        }
    }

    if config.should_run(Step::Rustup) {
        execute(
            &mut report,
            "rustup",
            || generic::run_rustup(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Cargo) {
        execute(
            &mut report,
            "cargo",
            || generic::run_cargo_update(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Flutter) {
        execute(
            &mut report,
            "Flutter",
            || generic::run_flutter_upgrade(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Go) {
        execute(
            &mut report,
            "Go",
            || generic::run_go(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Emacs) {
        execute(&mut report, "Emacs", || emacs.upgrade(run_type), config.no_retry())?;
    }

    if config.should_run(Step::Opam) {
        execute(
            &mut report,
            "opam",
            || generic::run_opam_update(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Vcpkg) {
        execute(
            &mut report,
            "vcpkg",
            || generic::run_vcpkg_update(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Pipx) {
        execute(
            &mut report,
            "pipx",
            || generic::run_pipx_update(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Stack) {
        execute(
            &mut report,
            "stack",
            || generic::run_stack_update(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Tlmgr) {
        execute(
            &mut report,
            "tlmgr",
            || {
                generic::run_tlmgr_update(
                    #[cfg(unix)]
                    &sudo,
                    #[cfg(windows)]
                    &None,
                    run_type,
                )
            },
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Myrepos) {
        execute(
            &mut report,
            "myrepos",
            || generic::run_myrepos_update(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Pearl) {
            execute(&mut report, "pearl", || unix::run_pearl(run_type), config.no_retry())?;
        }
    }

    if config.should_run(Step::Jetpack) {
        execute(
            &mut report,
            "jetpak",
            || generic::run_jetpack(run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Vim) {
        execute(
            &mut report,
            "vim",
            || vim::upgrade_vim(&base_dirs, run_type, config.cleanup()),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "Neovim",
            || vim::upgrade_neovim(&base_dirs, run_type, config.cleanup()),
            config.no_retry(),
        )?;
        execute(
            &mut report,
            "voom",
            || vim::run_voom(&base_dirs, run_type),
            config.no_retry(),
        )?;
    }

    if config.should_run(Step::Node) {
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
    }

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        if config.should_run(Step::Atom) {
            execute(&mut report, "apm", || generic::run_apm(run_type), config.no_retry())?;
        }
    }

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
        if config.should_run(Step::PackageManagers) {
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
        if config.should_run(Step::System) {
            execute(
                &mut report,
                "pihole",
                || linux::run_pihole_update(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }

        if config.should_run(Step::Firmware) {
            execute(
                &mut report,
                "Firmware upgrades",
                || linux::run_fwupdmgr(run_type),
                config.no_retry(),
            )?;
        }

        if config.should_run(Step::Restarts) {
            execute(
                &mut report,
                "Restarts",
                || linux::run_needrestart(sudo.as_ref(), run_type),
                config.no_retry(),
            )?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if config.should_run(Step::System) {
            execute(&mut report, "App Store", || macos::run_mas(run_type), config.no_retry())?;

            execute(
                &mut report,
                "System upgrade",
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
                || {
                    powershell::Powershell::windows_powershell()
                        .windows_update(run_type, config.accept_all_windows_updates())
                },
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

        #[cfg(target_os = "dragonfly")]
        dragonfly::audit_packages(&sudo).ok();
    }

    if config.keep_at_end() {
        print_info("\n(R)eboot\n(S)hell\n(Q)uit");
        loop {
            match get_char() {
                's' | 'S' => {
                    run_shell();
                }
                'r' | 'R' => {
                    reboot();
                }
                'q' | 'Q' => (),
                _ => {
                    continue;
                }
            }
            break;
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
            #[cfg(all(windows, feature = "self-update"))]
            {
                if let Some(Upgraded(status)) = error.downcast_ref::<Upgraded>() {
                    exit(status.code().unwrap());
                }
            }

            let skip_print = (error.downcast_ref::<StepFailed>().is_some())
                || (error
                    .downcast_ref::<io::Error>()
                    .filter(|io_error| io_error.kind() == io::ErrorKind::Interrupted)
                    .is_some());

            if !skip_print {
                println!("Error: {}", error);
            }
            exit(1);
        }
    }
}
