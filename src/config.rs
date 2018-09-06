use directories::BaseDirs;
use failure;
use shellexpand;
use std::collections::BTreeMap;
use std::fs;
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
    #[structopt(short = "t", long = "tmux", help = "Run inside tmux")]
    pub run_in_tmux: bool,

    #[structopt(long = "no-system", help = "Don't perform system upgrade")]
    pub no_system: bool,

    #[structopt(long = "no-git-repos", help = "Don't perform updates on configured git repos")]
    pub no_git_repos: bool,

    #[structopt(long = "no-emacs", help = "Don't upgrade Emacs packages or configuration files")]
    pub no_emacs: bool,

    #[structopt(short = "n", long = "dry-run", help = "Print what would be done")]
    pub dry_run: bool,
}
