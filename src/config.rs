use super::error::{Error, ErrorKind};
use super::utils::editor;
use directories::BaseDirs;
use failure::ResultExt;
use lazy_static::lazy_static;
use log::{debug, error, LevelFilter};
use pretty_env_logger::formatted_timed_builder;
use serde::Deserialize;
use shellexpand;
use std::collections::{BTreeMap, HashMap};
use std::fs::write;
use std::path::PathBuf;
use std::process::Command;
use std::{env, fs};
use structopt::StructOpt;
use toml;

type Commands = BTreeMap<String, String>;

lazy_static! {
    // While this is used to automatically generate possible value list everywhere in the code, the
    // README.md file still needs to be manually updated.
    static ref STEPS_MAPPING: HashMap<&'static str, Step> = {
        let mut m = HashMap::new();

        m.insert("system", Step::System);
        m.insert("git-repos", Step::GitRepos);
        m.insert("vim", Step::Vim);
        m.insert("emacs", Step::Emacs);
        m.insert("gem", Step::Gem);
        m.insert("sdkman", Step::Sdkman);
        m.insert("remotes", Step::Remotes);
        m.insert("rustup", Step::Rustup);
        m.insert("cargo", Step::Cargo);

        #[cfg(windows)]
        m.insert("powershell", Step::Powershell);

        m
    };
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Step {
    /// Don't perform system upgrade
    System,
    /// Don't pull git repositories
    GitRepos,
    /// Don't upgrade Vim packages or configuration files
    Vim,
    /// Don't upgrade Emacs packages or configuration files
    Emacs,
    /// Don't upgrade ruby gems
    Gem,
    /// Don't upgrade SDKMAN! and its packages
    Sdkman,
    /// Don't run remote Togprades
    Remotes,
    /// Don't run Rustup
    Rustup,
    /// Don't run Cargo
    Cargo,
    /// Don't update Powershell modules
    Powershell,
}

impl Step {
    fn possible_values() -> Vec<&'static str> {
        STEPS_MAPPING.keys().cloned().collect()
    }
}

impl std::str::FromStr for Step {
    type Err = structopt::clap::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(STEPS_MAPPING.get(s).unwrap().clone())
    }
}

#[derive(Deserialize, Default, Debug)]
#[serde(deny_unknown_fields)]
/// Configuration file
pub struct ConfigFile {
    pre_commands: Option<Commands>,
    commands: Option<Commands>,
    git_repos: Option<Vec<String>>,
    disable: Option<Vec<Step>>,
    remote_topgrades: Option<Vec<String>>,
    ssh_arguments: Option<String>,
}

impl ConfigFile {
    fn ensure(base_dirs: &BaseDirs) -> Result<PathBuf, Error> {
        let config_path = base_dirs.config_dir().join("topgrade.toml");
        if !config_path.exists() {
            write(&config_path, include_str!("../config.example.toml"))
                .map_err(|e| {
                    error!(
                        "Unable to write the example configuration file to {}: {}",
                        config_path.display(),
                        e
                    );
                    e
                })
                .context(ErrorKind::Configuration)?;
            debug!("No configuration exists");
        }

        Ok(config_path)
    }

    /// Read the configuration file.
    ///
    /// If the configuration file does not exist the function returns the default ConfigFile.
    fn read(base_dirs: &BaseDirs) -> Result<ConfigFile, Error> {
        let config_path = Self::ensure(base_dirs)?;
        let mut result: Self = toml::from_str(&fs::read_to_string(config_path).context(ErrorKind::Configuration)?)
            .context(ErrorKind::Configuration)?;

        if let Some(ref mut paths) = &mut result.git_repos {
            for path in paths.iter_mut() {
                *path = shellexpand::tilde::<&str>(&path.as_ref()).into_owned();
            }
        }

        debug!("Loaded configuration: {:?}", result);

        Ok(result)
    }

    fn edit(base_dirs: &BaseDirs) -> Result<(), Error> {
        let config_path = Self::ensure(base_dirs)?;
        let editor = editor();

        debug!("Editing {} with {}", config_path.display(), editor);
        Command::new(editor)
            .arg(config_path)
            .spawn()
            .and_then(|mut p| p.wait())
            .context(ErrorKind::Configuration)?;
        Ok(())
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Topgrade", raw(setting = "structopt::clap::AppSettings::ColoredHelp"))]
/// Command line arguments
pub struct CommandLineArgs {
    /// Edit the configuration file
    #[structopt(long = "edit-config")]
    edit_config: bool,

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
    #[structopt(long = "disable", raw(possible_values = "&Step::possible_values()"))]
    disable: Vec<Step>,

    /// Output logs
    #[structopt(short = "v", long = "verbose")]
    verbose: bool,

    /// Prompt for a key before exiting
    #[structopt(short = "k", long = "keep")]
    keep_at_end: bool,
}

/// Represents the application configuration
///
/// The struct holds the loaded configuration file, as well as the arguments parsed from the command line.
/// Its provided methods decide the appropriate options based on combining the configuraiton file and the
/// command line arguments.
pub struct Config {
    opt: CommandLineArgs,
    config_file: ConfigFile,
}

impl Config {
    /// Load the configuration.
    ///
    /// The function parses the command line arguments and reading the configuration file.
    pub fn load(base_dirs: &BaseDirs) -> Result<Self, Error> {
        let opt = CommandLineArgs::from_args();

        let mut builder = formatted_timed_builder();

        if opt.verbose {
            builder.filter(Some("topgrade"), LevelFilter::Trace);
        }

        builder.init();

        Ok(Self {
            opt,
            config_file: ConfigFile::read(base_dirs)?,
        })
    }

    /// Launch an editor to edit the configuration
    pub fn edit(base_dirs: &BaseDirs) -> Result<(), Error> {
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
        &self.config_file.git_repos
    }

    /// Tell whether the specified step should run.
    ///
    /// If the step appears either in the `--disable` command line argument
    /// or the `disable` option in the configuration, the function returns false.
    pub fn should_run(&self, step: Step) -> bool {
        !(self
            .config_file
            .disable
            .as_ref()
            .map(|d| d.contains(&step))
            .unwrap_or(false)
            || self.opt.disable.contains(&step))
    }

    /// Tell whether we should run in tmux.
    pub fn run_in_tmux(&self) -> bool {
        self.opt.run_in_tmux
    }

    /// Tell whether we should perform cleanup steps.
    #[cfg(not(windows))]
    pub fn cleanup(&self) -> bool {
        self.opt.cleanup
    }

    /// Tell whether we are dry-running.
    pub fn dry_run(&self) -> bool {
        self.opt.dry_run
    }

    /// Tell whether we should not attempt to retry anything.
    pub fn no_retry(&self) -> bool {
        self.opt.no_retry
    }

    /// List of remote hosts to run Topgrade in
    pub fn remote_topgrades(&self) -> &Option<Vec<String>> {
        &self.config_file.remote_topgrades
    }

    /// Extra SSH arguments
    pub fn ssh_arguments(&self) -> &Option<String> {
        &self.config_file.ssh_arguments
    }

    /// Prompt for a key before exiting
    pub fn keep_at_end(&self) -> bool {
        self.opt.keep_at_end || env::var("TOPGRADE_KEEP_END").is_ok()
    }

    /// Whether to edit the configuration file
    pub fn edit_config(&self) -> bool {
        self.opt.edit_config
    }
}
