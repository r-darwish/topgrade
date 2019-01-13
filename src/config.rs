use super::error::{Error, ErrorKind};
use directories::BaseDirs;
use failure::ResultExt;
use serde::Deserialize;
use shellexpand;
use std::collections::BTreeMap;
use std::fs;
use structopt::StructOpt;
use toml;

type Commands = BTreeMap<String, String>;

#[derive(Debug, PartialEq)]
pub enum Group {
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

impl std::str::FromStr for Group {
    type Err = structopt::clap::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            "system" => Group::System,
            "git-repos" => Group::GitRepos,
            "vim" => Group::Vim,
            "emacs" => Group::Emacs,
            "gem" => Group::Gem,
            _ => {
                return Err(structopt::clap::Error::with_description(
                    "Allowed values: system, git-repos, vim, emacs, gem",
                    structopt::clap::ErrorKind::InvalidValue,
                ));
            }
        })
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

    /// Do not perform upgrades for the given groups. Allowed options: system, git-repos, vim, emacs
    #[structopt(long = "no")]
    pub no: Vec<Group>,
}
