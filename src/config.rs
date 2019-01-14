use super::error::{Error, ErrorKind};
use directories::BaseDirs;
use failure::ResultExt;
use lazy_static::lazy_static;
use serde::Deserialize;
use shellexpand;
use std::collections::{BTreeMap, HashMap};
use std::fs;
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

        m
    };
}

#[derive(Debug, Clone, PartialEq)]
pub enum Step {
    /// Don't perform system upgrade
    System,
    /// Don't perform updates on configured git repos
    GitRepos,
    /// Don't upgrade Vim packages or configuration files
    Vim,
    /// Don't upgrade Emacs packages or configuration files
    Emacs,
    /// Don't upgrade ruby gems
    Gem,
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

#[derive(Deserialize, Default)]
pub struct Config {
    pre_commands: Option<Commands>,
    commands: Option<Commands>,
    git_repos: Option<Vec<String>>,
}

impl Config {
    pub fn read(base_dirs: &BaseDirs) -> Result<Config, Error> {
        let config_path = base_dirs.config_dir().join("topgrade.toml");
        if !config_path.exists() {
            return Ok(Default::default());
        }

        let mut result: Self = toml::from_str(&fs::read_to_string(config_path).context(ErrorKind::Configuration)?)
            .context(ErrorKind::Configuration)?;

        if let Some(ref mut paths) = &mut result.git_repos {
            for path in paths.iter_mut() {
                *path = shellexpand::tilde::<&str>(&path.as_ref()).into_owned();
            }
        }

        Ok(result)
    }

    pub fn pre_commands(&self) -> &Option<Commands> {
        &self.pre_commands
    }

    pub fn commands(&self) -> &Option<Commands> {
        &self.commands
    }

    pub fn git_repos(&self) -> &Option<Vec<String>> {
        &self.git_repos
    }
}

#[derive(StructOpt, Debug)]
#[structopt(name = "Topgrade")]
pub struct Opt {
    /// Run inside tmux
    #[structopt(short = "t", long = "tmux")]
    pub run_in_tmux: bool,

    /// Cleanup temporary or old files
    #[structopt(short = "c", long = "cleanup")]
    pub cleanup: bool,

    /// Print what would be done
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,

    /// Do not ask to retry failed steps
    #[structopt(long = "no-retry")]
    pub no_retry: bool,

    /// Do not perform upgrades for the given steps
    #[structopt(long = "disable", raw(possible_values = "&Step::possible_values()"))]
    pub disable: Vec<Step>,
}
