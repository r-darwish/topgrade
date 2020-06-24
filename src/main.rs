#![allow(clippy::cognitive_complexity)]
mod config;
mod ctrlc;
mod error;
mod execution_context;
mod executor;
mod report;
mod runner;
#[cfg(feature = "self-update")]
mod self_update;
mod steps;
mod terminal;
mod utils;

use self::config::{CommandLineArgs, Config, Step};
use self::error::StepFailed;
#[cfg(all(windows, feature = "self-update"))]
use self::error::Upgraded;

use self::steps::*;
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

    #[cfg(unix)]
    let sudo = utils::sudo();
    let run_type = executor::RunType::new(config.dry_run());

    #[cfg(unix)]
    let ctx = execution_context::ExecutionContext::new(run_type, &sudo, &git, &config, &base_dirs);

    #[cfg(not(unix))]
    let ctx = execution_context::ExecutionContext::new(run_type, &git, &config, &base_dirs);

    let mut runner = runner::Runner::new(&ctx);

    #[cfg(feature = "self-update")]
    {
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
            generic::run_custom_command(&name, &command, &ctx)?;
        }
    }

    let powershell = powershell::Powershell::new();
    let should_run_powershell = powershell.profile().is_some() && config.should_run(Step::Shell);

    #[cfg(windows)]
    {
        if config.should_run(Step::Wsl) {
            runner.execute("WSL", || windows::run_wsl_topgrade(run_type))?;
        }
    }

    if let Some(topgrades) = config.remote_topgrades() {
        if config.should_run(Step::Remotes) {
            for remote_topgrade in topgrades {
                runner.execute(format!("Remote ({})", remote_topgrade), || {
                    generic::run_remote_topgrade(&ctx, remote_topgrade)
                })?;
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
                    runner.execute("System update", || distribution.upgrade(&ctx))?;
                }
                Err(e) => {
                    println!("Error detecting current distribution: {}", e);
                }
            }
            runner.execute("etc-update", || linux::run_etc_update(sudo.as_ref(), run_type))?;
        }
    }

    #[cfg(windows)]
    {
        if config.should_run(Step::PackageManagers) {
            runner.execute("Chocolatey", || windows::run_chocolatey(run_type))?;

            runner.execute("Scoop", || windows::run_scoop(config.cleanup(), run_type))?;
        }
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Brew) {
            runner.execute("brew", || unix::run_homebrew(&ctx))?;
        }
        #[cfg(target_os = "macos")]
        {
            if config.should_run(Step::MacPorts) {
                runner.execute("MacPorts", || macos::run_macports(&ctx))?;
            }
            if config.should_run(Step::MicrosoftAutoUpdate) {
                runner.execute("Microsoft AutoUpdate", || macos::run_msupdate(&ctx))?;
            }
        }
        if config.should_run(Step::Nix) {
            runner.execute("nix", || unix::run_nix(&ctx))?;
        }

        if config.should_run(Step::HomeManager) {
            runner.execute("home-manager", || unix::run_home_manager(run_type))?;
        }

        if config.should_run(Step::Asdf) {
            runner.execute("asdf", || unix::run_asdf(run_type))?;
        }
    }

    #[cfg(target_os = "dragonfly")]
    {
        if config.should_run(Step::PackageManagers) {
            runner.execute("DragonFly BSD Packages", || {
                dragonfly::upgrade_packages(sudo.as_ref(), run_type)
            })?;
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if config.should_run(Step::PackageManagers) {
            runner.execute("FreeBSD Packages", || {
                freebsd::upgrade_packages(sudo.as_ref(), run_type)
            })?;
        }
    }

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
        runner.execute("Git repositories", || git.multi_pull_step(&git_repos, &ctx))?;
    }

    if should_run_powershell {
        runner.execute("Powershell Modules Update", || powershell.update_modules(run_type))?;
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Shell) {
            runner.execute("zr", || zsh::run_zr(&base_dirs, run_type))?;
            runner.execute("antibody", || zsh::run_antibody(run_type))?;
            runner.execute("antigen", || zsh::run_antigen(&base_dirs, run_type))?;
            runner.execute("zplug", || zsh::run_zplug(&base_dirs, run_type))?;
            runner.execute("zinit", || zsh::run_zinit(&base_dirs, run_type))?;
            runner.execute("oh-my-zsh", || zsh::run_oh_my_zsh(&ctx))?;
            runner.execute("fisher", || unix::run_fisher(&base_dirs, run_type))?;
        }

        if config.should_run(Step::Tmux) {
            runner.execute("tmux", || tmux::run_tpm(&base_dirs, run_type))?;
        }

        if config.should_run(Step::Tldr) {
            runner.execute("TLDR", || unix::run_tldr(run_type))?;
        }
    }

    if config.should_run(Step::Rustup) {
        runner.execute("rustup", || generic::run_rustup(&base_dirs, run_type))?;
    }

    if config.should_run(Step::Cargo) {
        runner.execute("cargo", || generic::run_cargo_update(run_type))?;
    }

    if config.should_run(Step::Flutter) {
        runner.execute("Flutter", || generic::run_flutter_upgrade(run_type))?;
    }

    if config.should_run(Step::Go) {
        runner.execute("Go", || generic::run_go(&base_dirs, run_type))?;
    }

    if config.should_run(Step::Emacs) {
        runner.execute("Emacs", || emacs.upgrade(run_type))?;
    }

    if config.should_run(Step::Opam) {
        runner.execute("opam", || generic::run_opam_update(run_type))?;
    }

    if config.should_run(Step::Vcpkg) {
        runner.execute("vcpkg", || generic::run_vcpkg_update(run_type))?;
    }

    if config.should_run(Step::Pipx) {
        runner.execute("pipx", || generic::run_pipx_update(run_type))?;
    }

    if config.should_run(Step::Stack) {
        runner.execute("stack", || generic::run_stack_update(run_type))?;
    }

    if config.should_run(Step::Tlmgr) {
        runner.execute("tlmgr", || generic::run_tlmgr_update(&ctx))?;
    }

    if config.should_run(Step::Myrepos) {
        runner.execute("myrepos", || generic::run_myrepos_update(&base_dirs, run_type))?;
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Pearl) {
            runner.execute("pearl", || unix::run_pearl(run_type))?;
        }
    }

    if config.should_run(Step::Jetpack) {
        runner.execute("jetpak", || generic::run_jetpack(run_type))?;
    }

    if config.should_run(Step::Vim) {
        runner.execute("vim", || vim::upgrade_vim(&base_dirs, run_type, config.cleanup()))?;
        runner.execute("Neovim", || vim::upgrade_neovim(&base_dirs, run_type, config.cleanup()))?;
        runner.execute("voom", || vim::run_voom(&base_dirs, run_type))?;
    }

    if config.should_run(Step::Node) {
        runner.execute("npm", || node::run_npm_upgrade(&base_dirs, run_type))?;
        runner.execute("yarn", || node::yarn_global_update(run_type))?;
    }

    if config.should_run(Step::Composer) {
        runner.execute("composer", || generic::run_composer_update(&ctx))?;
    }

    #[cfg(not(any(
        target_os = "freebsd",
        target_os = "openbsd",
        target_os = "netbsd",
        target_os = "dragonfly"
    )))]
    {
        if config.should_run(Step::Atom) {
            runner.execute("apm", || generic::run_apm(run_type))?;
        }
    }

    if config.should_run(Step::Gem) {
        runner.execute("gem", || generic::run_gem(&base_dirs, run_type))?;
    }

    #[cfg(target_os = "linux")]
    {
        if config.should_run(Step::PackageManagers) {
            runner.execute("Flatpak", || linux::flatpak_update(run_type))?;
            runner.execute("snap", || linux::run_snap(sudo.as_ref(), run_type))?;
        }
    }

    if let Some(commands) = config.commands() {
        for (name, command) in commands {
            runner.execute(name, || generic::run_custom_command(&name, &command, &ctx))?;
        }
    }

    #[cfg(target_os = "linux")]
    {
        if config.should_run(Step::System) {
            runner.execute("pihole", || linux::run_pihole_update(sudo.as_ref(), run_type))?;
        }

        if config.should_run(Step::Firmware) {
            runner.execute("Firmware upgrades", || linux::run_fwupdmgr(run_type))?;
        }

        if config.should_run(Step::Restarts) {
            runner.execute("Restarts", || linux::run_needrestart(sudo.as_ref(), run_type))?;
        }
    }

    #[cfg(target_os = "macos")]
    {
        if config.should_run(Step::System) {
            runner.execute("App Store", || macos::run_mas(run_type))?;
            runner.execute("System upgrade", || macos::upgrade_macos(&ctx))?;
        }
    }

    #[cfg(target_os = "freebsd")]
    {
        if config.should_run(Step::System) {
            runner.execute("FreeBSD Upgrade", || freebsd::upgrade_freebsd(sudo.as_ref(), run_type))?;
        }
    }

    #[cfg(windows)]
    {
        if config.should_run(Step::System) {
            runner.execute("Windows update", || windows::windows_update(&ctx))?;
        }
    }

    #[cfg(unix)]
    {
        if config.should_run(Step::Sdkman) {
            runner.execute("SDKMAN!", || unix::run_sdkman(&base_dirs, config.cleanup(), run_type))?;
        }
    }

    if config.should_run(Step::Vagrant) {
        if let Ok(boxes) = vagrant::collect_boxes(&ctx) {
            for vagrant_box in boxes {
                runner.execute(format!("Vagrant ({})", vagrant_box.smart_name()), || {
                    vagrant::topgrade_vagrant_box(&ctx, &vagrant_box)
                })?;
            }
        }
    }

    if !runner.report().data().is_empty() {
        print_separator("Summary");

        for (key, succeeded) in runner.report().data() {
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

    if runner.report().data().iter().all(|(_, succeeded)| *succeeded) {
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
