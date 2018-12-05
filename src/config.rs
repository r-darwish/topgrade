use directories::BaseDirs;
use failure;
use serde_derive::Deserialize;
use shellexpand;
use std::collections::BTreeMap;
use std::fs;
use structopt::StructOpt;
use toml;

type Commands = BTreeMap<String, String>;

#[derive(Deserialize, Default)]
pub struct Config {
    pre_commands: Option<Commands>,
    commands: Option<Commands>,
    git_repos: Option<Vec<String>>,
}

impl Config {
    pub fn read(base_dirs: &BaseDirs) -> Result<Config, failure::Error> {
        let config_path = base_dirs.config_dir().join("topgrade.toml");
        if !config_path.exists() {
            return Ok(Default::default());
        }

        let mut result: Self = toml::from_str(&fs::read_to_string(config_path)?)?;

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

    /// Don't perform system upgrade
    #[structopt(long = "no-system")]
    pub no_system: bool,

    /// Don't perform updates on configured git repos
    #[structopt(long = "no-git-repos")]
    pub no_git_repos: bool,

    /// Don't upgrade Emacs packages or configuration files
    #[structopt(long = "no-emacs")]
    pub no_emacs: bool,

    /// Print what would be done
    #[structopt(short = "n", long = "dry-run")]
    pub dry_run: bool,
}
