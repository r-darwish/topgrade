use crate::error::Error;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{which, HumanizedPath};
use log::{debug, error};
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug)]
pub struct Git {
    git: Option<PathBuf>,
}

#[derive(Debug)]
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
            Ok(mut path) => {
                debug_assert!(path.exists());

                if path.is_file() {
                    debug!("{} is a file. Checking {}", path.display(), path.parent()?.display());
                    path = path.parent()?.to_path_buf();
                }

                debug!("Checking if {} is a git repository", path.display());

                if let Some(git) = &self.git {
                    let output = Command::new(&git)
                        .args(&["rev-parse", "--show-toplevel"])
                        .current_dir(path)
                        .check_output()
                        .ok()
                        .map(|output| output.trim().to_string());
                    return output;
                }
            }
            Err(e) => match e.kind() {
                io::ErrorKind::NotFound => debug!("{} does not exists", path.as_ref().display()),
                _ => error!("Error looking for {}: {}", path.as_ref().display(), e),
            },
        }

        None
    }

    pub fn pull<P: AsRef<Path>>(&self, path: P, run_type: RunType) -> Option<(String, bool)> {
        let path = path.as_ref();

        print_separator(format!("Pulling {}", HumanizedPath::from(path)));

        let git = self.git.as_ref().unwrap();

        let success = || -> Result<(), Error> {
            run_type
                .execute(git)
                .args(&["pull", "--rebase", "--autostash"])
                .current_dir(&path)
                .check_run()?;

            Ok(())
        }()
        .is_ok();

        Some((format!("git: {}", HumanizedPath::from(path)), success))
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
