use super::error::{Error, ErrorKind};
use custom_derive::custom_derive;
use directories::BaseDirs;
use enum_derive::{enum_derive_util, EnumFromStr};
use failure::ResultExt;
use serde::Deserialize;
use shellexpand;
use std::collections::BTreeMap;
use std::fs;
use structopt::StructOpt;
use toml;

type Commands = BTreeMap<String, String>;

custom_derive! {
    #[derive(Debug, EnumFromStr, PartialEq)]
    #[allow(non_camel_case_types)]
    pub enum Group {
        /// Don't perform system upgrade
        system,
        /// Don't perform updates on configured git repos
        git_repos,
        /// Don't upgrade Vim packages or configuration files
        vim,
        /// Don't upgrade Emacs packages or configuration files
        emacs,
        /// Don't upgrade ruby gems
        gem,
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

    /// Do not perform upgrades for the given groups. Allowed options: system, git_repos, vim, emacs
    #[structopt(long = "no")]
    pub no: Vec<Group>,
}
