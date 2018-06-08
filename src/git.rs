use super::Check;
use failure::Error;
use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

pub struct Git {
    git: Option<PathBuf>,
}

impl Git {
    pub fn new() -> Self {
        Self {
            git: which("git").ok(),
        }
    }

    pub fn get_repo_root<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        if !path.as_ref().exists() {
            return None;
        }

        if let Some(git) = &self.git {
            let output = Command::new(&git)
                .arg("rev-parse")
                .arg("--show-toplevel")
                .current_dir(path)
                .output();

            if let Ok(output) = output {
                if !output.status.success() {
                    return None;
                }

                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        None
    }

    pub fn pull<P: AsRef<Path>>(&self, path: P) -> Result<Option<()>, Error> {
        if let Some(git) = &self.git {
            Command::new(&git)
                .arg("pull")
                .arg("--rebase")
                .arg("--autostash")
                .current_dir(path)
                .spawn()?
                .wait()?
                .check()?;

            return Ok(Some(()));
        }

        Ok(None)
    }

    pub fn insert_if_valid<P: AsRef<Path>>(&self, git_repos: &mut HashSet<String>, path: P) {
        if let Some(repo) = self.get_repo_root(path) {
            git_repos.insert(repo);
        }
    }
}
