use super::utils::editor;
use anyhow::Result;
use directories::BaseDirs;
use log::{debug, LevelFilter};
use pretty_env_logger::formatted_timed_builder;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs::write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use structopt::StructOpt;
use strum::{EnumIter, EnumString, EnumVariantNames, IntoEnumIterator, VariantNames};
use which_crate::which;

pub static EXAMPLE_CONFIG: &str = include_str!("../config.example.toml");

#[allow(unused_macros)]
macro_rules! str_value {
    ($section:ident, $value:ident) => {
        pub fn $value(&self) -> Option<&str> {
            self.config_file
                .$section
                .as_ref()
                .and_then(|section| section.$value.as_deref())
        }
    };
}

macro_rules! check_deprecated {
    ($config:expr, $old:ident, $section:ident, $new:ident) => {
        if $config.$old.is_some() {
            println!(concat!(
                "'",
                stringify!($old),
                "' configuration option is deprecated. Rename it to '",
                stringify!($new),
                "' and put it under the section [",
                stringify!($section),
                "]",
            ));
        }
    };
}
macro_rules! get_deprecated {
    ($config:expr, $old:ident, $section:ident, $new:ident) => {
        if $config.$old.is_some() {
            &$config.$old
        } else {
            if let Some(section) = &$config.$section {
                &section.$new
            } else {
                &None
            }
        }
    };
}

type Commands = BTreeMap<String, String>;

#[derive(EnumString, EnumVariantNames, Debug, Clone, PartialEq, Deserialize, EnumIter, Copy)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Step {
    System,
    GitRepos,
    Vim,
    Emacs,
    Gem,
    Node,
    Composer,
    Krew,
    Sdkman,
    Remotes,
    Rustup,
    Cargo,
    Flutter,
    Go,
    Shell,
    Opam,
    Vcpkg,
    Pipx,
    Stack,
    Tlmgr,
    Myrepos,
    Pearl,
    Jetpack,
    Atom,
    Firmware,
    Restarts,
    Tldr,
    Wsl,
    Tmux,
    Vagrant,
    MicrosoftAutoUpdate,
    Brew,
    MacPorts,
    Nix,
    HomeManager,
    Asdf,
    Chocolatey,
    Scoop,
    Flatpak,
    Snap,
    Pkg,
    Powershell,
    CustomCommands,
    Yadm,
    Sheldon,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Git {
    max_concurrency: Option<usize>,
    arguments: Option<String>,
    repos: Option<Vec<String>>,
    pull_predefined: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Vagrant {
    directories: Option<Vec<String>>,
    power_on: Option<bool>,
    always_suspend: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Windows {
    accept_all_updates: Option<bool>,
    self_rename: Option<bool>,
    use_gsudo_with_choco: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Brew {
    greedy_cask: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Linux {
    yay_arguments: Option<String>,
    trizen_arguments: Option<String>,
    dnf_arguments: Option<String>,
    enable_tlmgr: Option<bool>,
    emerge_sync_flags: Option<String>,
    emerge_update_flags: Option<String>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
pub struct Composer {
    self_update: Option<bool>,
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
/// Configuration file
pub struct ConfigFile {
    pre_commands: Option<Commands>,
    commands: Option<Commands>,
    git_repos: Option<Vec<String>>,
    predefined_git_repos: Option<bool>,
    disable: Option<Vec<Step>>,
    ignore_failures: Option<Vec<Step>>,
    remote_topgrades: Option<Vec<String>>,
    remote_topgrade_path: Option<String>,
    ssh_arguments: Option<String>,
    git_arguments: Option<String>,
    tmux_arguments: Option<String>,
    set_title: Option<bool>,
    assume_yes: Option<bool>,
    yay_arguments: Option<String>,
    no_retry: Option<bool>,
    run_in_tmux: Option<bool>,
    cleanup: Option<bool>,
    notify_each_step: Option<bool>,
    accept_all_windows_updates: Option<bool>,
    only: Option<Vec<Step>>,
    composer: Option<Composer>,
    brew: Option<Brew>,
    linux: Option<Linux>,
    git: Option<Git>,
    windows: Option<Windows>,
    vagrant: Option<Vagrant>,
}

fn config_directory(base_dirs: &BaseDirs) -> PathBuf {
    #[cfg(not(target_os = "macos"))]
    return base_dirs.config_dir().to_owned();

    #[cfg(target_os = "macos")]
    return base_dirs.home_dir().join(".config");
}

impl ConfigFile {
    fn ensure(base_dirs: &BaseDirs) -> Result<PathBuf> {
        let config_directory = config_directory(base_dirs);

        let config_path = config_directory.join("topgrade.toml");

        if !config_path.exists() {
            debug!("No configuration exists");
            write(&config_path, EXAMPLE_CONFIG).map_err(|e| {
                debug!(
                    "Unable to write the example configuration file to {}: {}. Using blank config.",
                    config_path.display(),
                    e
                );
                e
            })?;
        } else {
            debug!("Configuration at {}", config_path.display());
        }

        Ok(config_path)
    }

    /// Read the configuration file.
    ///
    /// If the configuration file does not exist the function returns the default ConfigFile.
    fn read(base_dirs: &BaseDirs, config_path: Option<PathBuf>) -> Result<ConfigFile> {
        let config_path = if let Some(path) = config_path {
            path
        } else {
            Self::ensure(base_dirs)?
        };

        let contents = fs::read_to_string(&config_path).map_err(|e| {
            log::error!("Unable to read {}", config_path.display());
            e
        })?;

        let mut result: Self = toml::from_str(&contents).map_err(|e| {
            log::error!("Failed to deserialize {}", config_path.display());
            e
        })?;

        if let Some(ref mut paths) = &mut result.git_repos {
            for path in paths.iter_mut() {
                let expanded = shellexpand::tilde::<&str>(&path.as_ref()).into_owned();
                debug!("Path {} expanded to {}", path, expanded);
                *path = expanded;
            }
        }

        if let Some(paths) = result.git.as_mut().and_then(|git| git.repos.as_mut()) {
            for path in paths.iter_mut() {
                let expanded = shellexpand::tilde::<&str>(&path.as_ref()).into_owned();
                debug!("Path {} expanded to {}", path, expanded);
                *path = expanded;
            }
        }

        debug!("Loaded configuration: {:?}", result);

        Ok(result)
    }

    fn edit(base_dirs: &BaseDirs) -> Result<()> {
        let config_path = Self::ensure(base_dirs)?;
        let editor = editor();

        let command = which(&editor[0])?;
        let args: Vec<&String> = editor.iter().skip(1).collect();

        Command::new(command)
            .args(args)
            .arg(config_path)
            .spawn()
            .and_then(|mut p| p.wait())?;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Topgrade", setting = structopt::clap::AppSettings::ColoredHelp)]
/// Command line arguments
pub struct CommandLineArgs {
    /// Edit the configuration file
    #[structopt(long = "edit-config")]
    edit_config: bool,

    /// Show config reference
    #[structopt(long = "config-reference")]
    show_config_reference: bool,

    /// Run inside tmux
    #[structopt(short = "t", long = "tmux")]
    run_in_tmux: bool,

    /// Cleanup temporary or old files
    #[structopt(short = "c", long = "cleanup")]
    cleanup: bool,

    /// Print what would be done
    #[structopt(short = "n", long = "dry-run")]
    dry_run: bool,

    /// Do not ask to retry failed steps
    #[structopt(long = "no-retry")]
    no_retry: bool,

    /// Do not perform upgrades for the given steps
    #[structopt(long = "disable", possible_values = &Step::VARIANTS)]
    disable: Vec<Step>,

    /// Perform only the specified steps (experimental)
    #[structopt(long = "only", possible_values = &Step::VARIANTS)]
    only: Vec<Step>,

    /// Output logs
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Prompt for a key before exiting
    #[structopt(short = "k", long = "keep")]
    keep_at_end: bool,

    /// Say yes to package manager's prompt (experimental)
    #[structopt(short = "y", long = "yes")]
    yes: bool,

    /// Don't pull the predefined git repos
    #[structopt(long = "disable-predefined-git-repos")]
    disable_predefined_git_repos: bool,

    /// Alternative configuration file
    #[structopt(long = "config")]
    config: Option<PathBuf>,
}

impl CommandLineArgs {
    pub fn edit_config(&self) -> bool {
        self.edit_config
    }

    pub fn show_config_reference(&self) -> bool {
        self.show_config_reference
    }
}

/// Represents the application configuration
///
/// The struct holds the loaded configuration file, as well as the arguments parsed from the command line.
/// Its provided methods decide the appropriate options based on combining the configuraiton file and the
/// command line arguments.
pub struct Config {
    opt: CommandLineArgs,
    config_file: ConfigFile,
    allowed_steps: Vec<Step>,
}

impl Config {
    /// Load the configuration.
    ///
    /// The function parses the command line arguments and reading the configuration file.
    pub fn load(base_dirs: &BaseDirs, opt: CommandLineArgs) -> Result<Self> {
        let mut builder = formatted_timed_builder();

        if opt.verbose {
            builder.filter(Some("topgrade"), LevelFilter::Trace);
        }

        builder.init();

        let config_directory = config_directory(base_dirs);
        let config_file = if config_directory.is_dir() {
            ConfigFile::read(base_dirs, opt.config.clone()).unwrap_or_else(|e| {
                // Inform the user about errors when loading the configuration,
                // but fallback to the default config to at least attempt to do something
                log::error!("failed to load configuration: {}", e);
                ConfigFile::default()
            })
        } else {
            log::debug!("Configuration directory {} does not exist", config_directory.display());
            ConfigFile::default()
        };

        check_deprecated!(config_file, git_arguments, git, arguments);
        check_deprecated!(config_file, git_repos, git, repos);
        check_deprecated!(config_file, predefined_git_repos, git, pull_predefined);
        check_deprecated!(config_file, yay_arguments, linux, yay_arguments);
        check_deprecated!(config_file, accept_all_windows_updates, windows, accept_all_updates);

        if config_file
            .windows
            .as_ref()
            .map(|w| w.use_gsudo_with_choco.is_some())
            .unwrap_or(false)
        {
            println!("use_gsudo_with_choco is deprecated and will be removed in the future. Topgrade will not automatically detect and use gsudo");
        }

        let allowed_steps = Self::allowed_steps(&opt, &config_file);

        Ok(Self {
            opt,
            config_file,
            allowed_steps,
        })
    }

    /// Launch an editor to edit the configuration
    pub fn edit(base_dirs: &BaseDirs) -> Result<()> {
        ConfigFile::edit(base_dirs)
    }

    /// The list of commands to run before performing any step.
    pub fn pre_commands(&self) -> &Option<Commands> {
        &self.config_file.pre_commands
    }

    /// The list of custom steps.
    pub fn commands(&self) -> &Option<Commands> {
        &self.config_file.commands
    }

    /// The list of additional git repositories to pull.
    pub fn git_repos(&self) -> &Option<Vec<String>> {
        get_deprecated!(&self.config_file, git_repos, git, repos)
    }

    /// Tell whether the specified step should run.
    ///
    /// If the step appears either in the `--disable` command line argument
    /// or the `disable` option in the configuration, the function returns false.
    pub fn should_run(&self, step: Step) -> bool {
        self.allowed_steps.contains(&step)
    }

    fn allowed_steps(opt: &CommandLineArgs, config_file: &ConfigFile) -> Vec<Step> {
        let mut enabled_steps: Vec<Step> = if !opt.only.is_empty() {
            opt.only.clone()
        } else {
            config_file
                .only
                .as_ref()
                .map_or_else(|| Step::iter().collect(), |v| v.clone())
        };

        let disabled_steps: Vec<Step> = if !opt.disable.is_empty() {
            opt.disable.clone()
        } else {
            config_file.disable.as_ref().map_or_else(Vec::new, |v| v.clone())
        };

        enabled_steps.retain(|e| !disabled_steps.contains(e));
        enabled_steps
    }

    /// Tell whether we should run in tmux.
    pub fn run_in_tmux(&self) -> bool {
        self.opt.run_in_tmux || self.config_file.run_in_tmux.unwrap_or(false)
    }

    /// Tell whether we should perform cleanup steps.
    pub fn cleanup(&self) -> bool {
        self.opt.cleanup || self.config_file.cleanup.unwrap_or(false)
    }

    /// Tell whether we are dry-running.
    pub fn dry_run(&self) -> bool {
        self.opt.dry_run
    }

    /// Tell whether we should not attempt to retry anything.
    pub fn no_retry(&self) -> bool {
        self.opt.no_retry || self.config_file.no_retry.unwrap_or(false)
    }

    /// List of remote hosts to run Topgrade in
    pub fn remote_topgrades(&self) -> &Option<Vec<String>> {
        &self.config_file.remote_topgrades
    }

    /// Path to Topgrade executable used for all remote hosts
    pub fn remote_topgrade_path(&self) -> &str {
        self.config_file.remote_topgrade_path.as_deref().unwrap_or("topgrade")
    }

    /// Extra SSH arguments
    pub fn ssh_arguments(&self) -> &Option<String> {
        &self.config_file.ssh_arguments
    }

    /// Extra Git arguments
    pub fn git_arguments(&self) -> &Option<String> {
        get_deprecated!(&self.config_file, git_arguments, git, arguments)
    }

    /// Extra Tmux arguments
    #[allow(dead_code)]
    pub fn tmux_arguments(&self) -> &Option<String> {
        &self.config_file.tmux_arguments
    }

    /// Prompt for a key before exiting
    pub fn keep_at_end(&self) -> bool {
        self.opt.keep_at_end || env::var("TOPGRADE_KEEP_END").is_ok()
    }

    /// Whether to set the terminal title
    pub fn set_title(&self) -> bool {
        self.config_file.set_title.unwrap_or(true)
    }

    /// Whether to say yes to package managers
    #[allow(dead_code)]
    pub fn yes(&self) -> bool {
        self.config_file.assume_yes.unwrap_or(self.opt.yes)
    }

    /// Whether to accept all Windows updates
    #[allow(dead_code)]
    pub fn accept_all_windows_updates(&self) -> bool {
        get_deprecated!(
            self.config_file,
            accept_all_windows_updates,
            windows,
            accept_all_updates
        )
        .unwrap_or(true)
    }

    /// Whether to self rename the Topgrade executable during the run
    #[allow(dead_code)]
    pub fn self_rename(&self) -> bool {
        self.config_file
            .windows
            .as_ref()
            .and_then(|w| w.self_rename)
            .unwrap_or(false)
    }

    /// Whether Brew cask should be greedy
    #[allow(dead_code)]
    pub fn brew_cask_greedy(&self) -> bool {
        self.config_file
            .brew
            .as_ref()
            .and_then(|c| c.greedy_cask)
            .unwrap_or(false)
    }

    /// Whether Composer should update itself
    pub fn composer_self_update(&self) -> bool {
        self.config_file
            .composer
            .as_ref()
            .and_then(|c| c.self_update)
            .unwrap_or(false)
    }

    /// Whether to send a desktop notification at the beginning of every step
    #[allow(dead_code)]
    pub fn notify_each_step(&self) -> bool {
        self.config_file.notify_each_step.unwrap_or(false)
    }

    /// Extra trizen arguments
    #[allow(dead_code)]
    pub fn trizen_arguments(&self) -> &str {
        &self
            .config_file
            .linux
            .as_ref()
            .and_then(|s| s.trizen_arguments.as_deref())
            .unwrap_or("")
    }

    /// Extra yay arguments
    #[allow(dead_code)]
    pub fn yay_arguments(&self) -> &str {
        get_deprecated!(self.config_file, yay_arguments, linux, yay_arguments)
            .as_deref()
            .unwrap_or("--devel")
    }

    /// Extra yay arguments
    #[allow(dead_code)]
    pub fn dnf_arguments(&self) -> Option<&str> {
        self.config_file
            .linux
            .as_ref()
            .and_then(|linux| linux.dnf_arguments.as_deref())
    }

    /// Concurrency limit for git
    pub fn git_concurrency_limit(&self) -> Option<usize> {
        self.config_file.git.as_ref().and_then(|git| git.max_concurrency)
    }

    /// Should we power on vagrant boxes if needed
    pub fn vagrant_power_on(&self) -> Option<bool> {
        self.config_file.vagrant.as_ref().and_then(|vagrant| vagrant.power_on)
    }

    /// Vagrant directories
    pub fn vagrant_directories(&self) -> Option<&Vec<String>> {
        self.config_file
            .vagrant
            .as_ref()
            .and_then(|vagrant| vagrant.directories.as_ref())
    }

    /// Always suspend vagrant boxes instead of powering off
    pub fn vagrant_always_suspend(&self) -> Option<bool> {
        self.config_file
            .vagrant
            .as_ref()
            .and_then(|vagrant| vagrant.always_suspend)
    }

    /// Extra yay arguments
    #[allow(dead_code)]
    pub fn enable_tlmgr_linux(&self) -> bool {
        self.config_file
            .linux
            .as_ref()
            .and_then(|linux| linux.enable_tlmgr)
            .unwrap_or(false)
    }

    /// Should we ignore failures for this step
    pub fn ignore_failure(&self, step: Step) -> bool {
        self.config_file
            .ignore_failures
            .as_ref()
            .map(|v| v.contains(&step))
            .unwrap_or(false)
    }

    pub fn use_predefined_git_repos(&self) -> bool {
        !self.opt.disable_predefined_git_repos
            && get_deprecated!(&self.config_file, predefined_git_repos, git, pull_predefined).unwrap_or(true)
    }

    pub fn verbose(&self) -> bool {
        self.opt.verbose
    }

    #[cfg(target_os = "linux")]
    str_value!(linux, emerge_sync_flags);

    #[cfg(target_os = "linux")]
    str_value!(linux, emerge_update_flags);
}
