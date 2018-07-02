use super::utils::{which, Check};
use failure::Error;
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub struct Git {
    git: Option<PathBuf>,
}

pub struct Repositories<'a> {
    git: &'a Git,
    repositories: HashSet<String>,
}

impl Git {
    pub fn new() -> Self {
        Self { git: which("git") }
    }

    pub fn get_repo_root<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        match path.as_ref().canonicalize() {
            Ok(path) => {
                debug_assert!(path.exists());

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
            }
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => debug!("{} does not exists", path.as_ref().display()),
                _ => error!("Error looking for {}: {}", path.as_ref().display(), e),
            },
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
}

impl<'a> Repositories<'a> {
    pub fn new(git: &'a Git) -> Self {
        Self {
            git,
            repositories: HashSet::new(),
        }
    }

    pub fn insert<P: AsRef<Path>>(&mut self, path: P) {
        if let Some(repo) = self.git.get_repo_root(path) {
            self.repositories.insert(repo);
        }
    }

    pub fn repositories(&self) -> &HashSet<String> {
        &self.repositories
    }
}
