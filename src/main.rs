#![allow(clippy::cognitive_complexity)]
mod config;
mod ctrlc;
mod error;
mod execution_context;
mod executor;
mod report;
mod runner;
#[cfg(windows)]
mod self_renamer;
#[cfg(feature = "self-update")]
mod self_update;
mod steps;
mod terminal;
mod utils;

use self::config::{CommandLineArgs, Config, Step};
use self::error::StepFailed;
#[cfg(all(windows, feature = "self-update"))]
use self::error::Upgraded;

use self::steps::{remote::*, *};
use self::terminal::*;
use anyhow::{anyhow, Result};
use log::debug;

use std::env;
use std::io;
use std::process::exit;
use structopt::clap::crate_version;
use structopt::StructOpt;

fn run() -> Result<()> {
    ctrlc::set_handler();

    let base_dirs = directories::BaseDirs::new().ok_or_else(|| anyhow!("No base directories"))?;

    let opt = CommandLineArgs::from_args();
    if opt.edit_config() {
        Config::edit(&base_dirs)?;
        return Ok(());
    };

    if opt.show_config_reference() {
        print!("{}", crate::config::EXAMPLE_CONFIG);
        return Ok(());
    }

    let config = Config::load(&base_dirs, opt)?;
    terminal::set_title(config.set_title());
    terminal::set_desktop_notifications(config.notify_each_step());

    debug!("Version: {}", crate_version!());
    debug!("OS: {}", env!("TARGET"));
    debug!("{:?}", std::env::args());
    debug!("Binary path: {:?}", std::env::current_exe());
    debug!("Self Update: {:?}", cfg!(feature = "self-update"));

    if config.run_in_tmux() && env::var("TOPGRADE_INSIDE_TMUX").is_err() {
        #[cfg(unix)]
        {
            tmux::run_in_tmux(config.tmux_arguments());
        }
    }

    let git = git::Git::new();
    let mut git_repos = git::Repositories::new(&git);

    let sudo = utils::sudo();
    let run_type = executor::RunType::new(config.dry_run());

    let ctx = execution_context::ExecutionContext::new(run_type, &sudo, &git, &config, &base_dirs);

    let mut runner = runner::Runner::new(&ctx);

    #[cfg(feature = "self-update")]
    {
        #[cfg(target_os = "linux")]
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

    #[cfg(windows)]
    let _self_rename = if config.self_rename() {
        Some(crate::self_renamer::SelfRenamer::create()?)
    } else {
        None
    };

    if let Some(commands) = config.pre_commands() {
        for (name, command) in commands {
            generic::run_custom_command(&name, &command, &ctx)?;
        }
    }

    let powershell = powershell::Powershell::new();
    let should_run_powershell = powershell.profile().is_some() && config.should_run(Step::Powershell);

    #[cfg(windows)]
    runner.execute(Step::Wsl, "WSL", || windows::run_wsl_topgrade(&ctx))?;

    if let Some(topgrades) = config.remote_topgrades() {
        for remote_topgrade in topgrades.iter().filter(|t| config.should_execute_remote(t)) {
            runner.execute(Step::Remotes, format!("Remote ({})", remote_topgrade), || {
                remote::ssh::ssh_step(&ctx, remote_topgrade)
            })?;
        }
    }

    #[cfg(target_os = "linux")]
    let distribution = linux::Distribution::detect();

    #[cfg(target_os = "linux")]
    {
        match &distribution {
            Ok(distribution) => {
                runner.execute(Step::System, "System update", || distribution.upgrade(&ctx))?;
            }
            Err(e) => {
                println!("Error detecting current distribution: {}", e);
            }
        }
        runner.execute(Step::System, "etc-update", || {
            linux::run_etc_update(sudo.as_ref(), run_type)
        })?;
    }

    #[cfg(windows)]
    {
        runner.execute(Step::Chocolatey, "Chocolatey", || windows::run_chocolatey(&ctx))?;
        runner.execute(Step::Scoop, "Scoop", || windows::run_scoop(config.cleanup(), run_type))?;
    }

    #[cfg(unix)]
    {
        runner.execute(Step::Brew, "Brew", || unix::run_brew(&ctx))?;

        #[cfg(target_os = "macos")]
        {
            runner.execute(Step::MacPorts, "MacPorts", || macos::run_macports(&ctx))?;
            runner.execute(Step::MicrosoftAutoUpdate, "Microsoft AutoUpdate", || {
                macos::run_msupdate(&ctx)
            })?;
        }

        runner.execute(Step::Yadm, "yadm", || unix::run_yadm(&ctx))?;
        runner.execute(Step::Nix, "nix", || unix::run_nix(&ctx))?;
        runner.execute(Step::HomeManager, "home-manager", || unix::run_home_manager(run_type))?;
        runner.execute(Step::Asdf, "asdf", || unix::run_asdf(run_type))?;
    }

    #[cfg(target_os = "dragonfly")]
    runner.execute(Step::Pkg, "DragonFly BSD Packages", || {
        dragonfly::upgrade_packages(sudo.as_ref(), run_type)
    })?;

    #[cfg(target_os = "freebsd")]
    runner.execute(Step::Pkg, "FreeBSD Packages", || {
        freebsd::upgrade_packages(sudo.as_ref(), run_type)
    })?;

    let emacs = emacs::Emacs::new(&base_dirs);
    if config.use_predefined_git_repos() {
        if config.should_run(Step::Emacs) {
            if !emacs.is_doom() {
                if let Some(directory) = emacs.directory() {
                    git_repos.insert_if_repo(directory);
                }
            }
            git_repos.insert_if_repo(base_dirs.home_dir().join(".doom.d"));
        }

        if config.should_run(Step::Vim) {
            git_repos.insert_if_repo(base_dirs.home_dir().join(".vim"));
            git_repos.insert_if_repo(base_dirs.home_dir().join(".config/nvim"));
        }

        #[cfg(unix)]
        {
            git_repos.insert_if_repo(zsh::zshrc(&base_dirs));
            if config.should_run(Step::Tmux) {
                git_repos.insert_if_repo(base_dirs.home_dir().join(".tmux"));
            }
            git_repos.insert_if_repo(base_dirs.home_dir().join(".config/fish"));
            git_repos.insert_if_repo(base_dirs.config_dir().join("openbox"));
            git_repos.insert_if_repo(base_dirs.config_dir().join("bspwm"));
            git_repos.insert_if_repo(base_dirs.config_dir().join("i3"));
            git_repos.insert_if_repo(base_dirs.config_dir().join("sway"));
        }

        #[cfg(windows)]
        git_repos.insert_if_repo(
            base_dirs
                .data_local_dir()
                .join("Packages/Microsoft.WindowsTerminal_8wekyb3d8bbwe/LocalState"),
        );

        if let Some(profile) = powershell.profile() {
            git_repos.insert_if_repo(profile);
        }
    }

    if config.should_run(Step::GitRepos) {
        if let Some(custom_git_repos) = config.git_repos() {
            for git_repo in custom_git_repos {
                git_repos.glob_insert(git_repo);
            }
        }
        runner.execute(Step::GitRepos, "Git repositories", || {
            git.multi_pull_step(&git_repos, &ctx)
        })?;
    }

    if should_run_powershell {
        runner.execute(Step::Powershell, "Powershell Modules Update", || {
            powershell.update_modules(&ctx)
        })?;
    }

    #[cfg(unix)]
    {
        runner.execute(Step::Shell, "zr", || zsh::run_zr(&base_dirs, run_type))?;
        runner.execute(Step::Shell, "antibody", || zsh::run_antibody(run_type))?;
        runner.execute(Step::Shell, "antigen", || zsh::run_antigen(&base_dirs, run_type))?;
        runner.execute(Step::Shell, "zplug", || zsh::run_zplug(&base_dirs, run_type))?;
        runner.execute(Step::Shell, "zinit", || zsh::run_zinit(&base_dirs, run_type))?;
        runner.execute(Step::Shell, "oh-my-zsh", || zsh::run_oh_my_zsh(&ctx))?;
        runner.execute(Step::Shell, "fisher", || unix::run_fisher(&base_dirs, run_type))?;
        runner.execute(Step::Shell, "oh-my-fish", || unix::run_oh_my_fish(&ctx))?;
        runner.execute(Step::Tmux, "tmux", || tmux::run_tpm(&base_dirs, run_type))?;
        runner.execute(Step::Tldr, "TLDR", || unix::run_tldr(run_type))?;
        runner.execute(Step::Pearl, "pearl", || unix::run_pearl(run_type))?;
        runner.execute(Step::Sdkman, "SDKMAN!", || {
            unix::run_sdkman(&base_dirs, config.cleanup(), run_type)
        })?;
    }

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    runner.execute(Step::Atom, "apm", || generic::run_apm(run_type))?;
    runner.execute(Step::Rustup, "rustup", || generic::run_rustup(&base_dirs, run_type))?;
    runner.execute(Step::Choosenim, "choosenim", || generic::run_choosenim(&ctx))?;
    runner.execute(Step::Cargo, "cargo", || generic::run_cargo_update(run_type))?;
    runner.execute(Step::Flutter, "Flutter", || generic::run_flutter_upgrade(run_type))?;
    runner.execute(Step::Go, "Go", || generic::run_go(&base_dirs, run_type))?;
    runner.execute(Step::Emacs, "Emacs", || emacs.upgrade(run_type))?;
    runner.execute(Step::Opam, "opam", || generic::run_opam_update(run_type))?;
    runner.execute(Step::Vcpkg, "vcpkg", || generic::run_vcpkg_update(run_type))?;
    runner.execute(Step::Pipx, "pipx", || generic::run_pipx_update(run_type))?;
    runner.execute(Step::Stack, "stack", || generic::run_stack_update(run_type))?;
    runner.execute(Step::Tlmgr, "tlmgr", || generic::run_tlmgr_update(&ctx))?;
    runner.execute(Step::Myrepos, "myrepos", || {
        generic::run_myrepos_update(&base_dirs, run_type)
    })?;
    runner.execute(Step::Jetpack, "jetpack", || generic::run_jetpack(run_type))?;
    runner.execute(Step::Vim, "vim", || vim::upgrade_vim(&base_dirs, &ctx))?;
    runner.execute(Step::Vim, "Neovim", || vim::upgrade_neovim(&base_dirs, &ctx))?;
    runner.execute(Step::Vim, "voom", || vim::run_voom(&base_dirs, run_type))?;
    runner.execute(Step::Node, "npm", || node::run_npm_upgrade(&base_dirs, run_type))?;
    runner.execute(Step::Node, "yarn", || node::yarn_global_update(run_type))?;
    runner.execute(Step::Deno, "deno", || node::deno_upgrade(&ctx))?;
    runner.execute(Step::Composer, "composer", || generic::run_composer_update(&ctx))?;
    runner.execute(Step::Krew, "krew", || generic::run_krew_upgrade(run_type))?;
    runner.execute(Step::Gem, "gem", || generic::run_gem(&base_dirs, run_type))?;
    runner.execute(Step::Sheldon, "sheldon", || generic::run_sheldon(&ctx))?;
    runner.execute(Step::Rtcl, "rtcl", || generic::run_rtcl(&ctx))?;

    #[cfg(target_os = "linux")]
    {
        runner.execute(Step::Flatpak, "Flatpak", || linux::flatpak_update(run_type))?;
        runner.execute(Step::Snap, "snap", || linux::run_snap(sudo.as_ref(), run_type))?;
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            runner.execute(Step::CustomCommands, name, || {
                generic::run_custom_command(&name, &command, &ctx)
            })?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        runner.execute(Step::System, "pihole", || {
            linux::run_pihole_update(sudo.as_ref(), run_type)
        })?;
        runner.execute(Step::Firmware, "Firmware upgrades", || linux::run_fwupdmgr(run_type))?;
        runner.execute(Step::Restarts, "Restarts", || {
            linux::run_needrestart(sudo.as_ref(), run_type)
        })?;
    }

    #[cfg(target_os = "macos")]
    {
        runner.execute(Step::System, "App Store", || macos::run_mas(run_type))?;
        runner.execute(Step::System, "System upgrade", || macos::upgrade_macos(&ctx))?;
    }

    #[cfg(target_os = "freebsd")]
    runner.execute(Step::System, "FreeBSD Upgrade", || {
        freebsd::upgrade_freebsd(sudo.as_ref(), run_type)
    })?;

    #[cfg(windows)]
    runner.execute(Step::System, "Windows update", || windows::windows_update(&ctx))?;

    if config.should_run(Step::Vagrant) {
        if let Ok(boxes) = vagrant::collect_boxes(&ctx) {
            for vagrant_box in boxes {
                runner.execute(Step::Vagrant, format!("Vagrant ({})", vagrant_box.smart_name()), || {
                    vagrant::topgrade_vagrant_box(&ctx, &vagrant_box)
                })?;
            }
        }
    }

    if !runner.report().data().is_empty() {
        print_separator("Summary");

        for (key, result) in runner.report().data() {
            print_result(key, result);
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

    let mut post_command_failed = false;
    if let Some(commands) = config.post_commands() {
        for (name, command) in commands {
            if generic::run_custom_command(&name, &command, &ctx).is_err() {
                post_command_failed = true;
            }
        }
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

    if post_command_failed || runner.report().data().iter().any(|(_, result)| result.failed()) {
        Err(StepFailed.into())
    } else {
        Ok(())
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
