use directories;
use failure;
use std::collections::BTreeMap;
use std::fs;
use toml;

#[derive(Deserialize, Default)]
pub struct Config {
    commands: Option<BTreeMap<String, String>>,
    git_repos: Option<Vec<String>>,
}

impl Config {
    pub fn read() -> Result<Config, failure::Error> {
        let base_dirs = directories::BaseDirs::new();
        let config_path = base_dirs.config_dir().join("topgrade.toml");
        if !config_path.exists() {
            return Ok(Default::default());
        }

        Ok(toml::from_str(&fs::read_to_string(config_path)?)?)
    }

    pub fn commands(&self) -> &Option<BTreeMap<String, String>> {
        &self.commands
    }

    pub fn git_repos(&self) -> &Option<Vec<String>> {
        &self.git_repos
    }
}
