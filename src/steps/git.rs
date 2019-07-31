use crate::error::{Error, ErrorKind};
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{which, HumanizedPath};
use console::style;
use futures::future::{join_all, Future};
use log::{debug, error};
use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::runtime::Runtime;
use tokio_process::CommandExt as TokioCommandExt;

#[derive(Debug)]
pub struct Git {
    git: Option<PathBuf>,
}

#[derive(Debug)]
pub struct Repositories<'a> {
    git: &'a Git,
    repositories: HashSet<String>,
}

/// Checks whether the latest pull command actually pulled something
fn did_pull_change(git: &Path, repo: &str) -> bool {
    Command::new(git)
        .args(&["rev-parse", "HEAD", "ORIG_HEAD"])
        .current_dir(repo)
        .check_output()
        .map(|output| {
            let mut lines = output.trim().split('\n');
            lines.next().unwrap() != lines.next().unwrap()
        })
        .map_err(|e| {
            error!("Error getting revision for {}: {}", repo, e);

            e
        })
        .unwrap_or(false)
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

    pub fn multi_pull(&self, repositories: &Repositories, run_type: RunType) -> Result<(), Error> {
        if repositories.repositories.is_empty() {
            return Ok(());
        }

        let git = self.git.as_ref().unwrap();

        print_separator("Git repositories");

        if let RunType::Dry = run_type {
            repositories
                .repositories
                .iter()
                .for_each(|repo| println!("Would pull {}", HumanizedPath::from(std::path::Path::new(&repo))));

            return Ok(());
        }

        let futures: Vec<_> = repositories
            .repositories
            .iter()
            .map(|repo| {
                let repo = repo.clone();
                let path = format!("{}", HumanizedPath::from(std::path::Path::new(&repo)));
                let cloned_git = git.to_owned();

                println!("{} {}", style("Pulling").cyan().bold(), path);

                Command::new(git)
                    .args(&["pull", "--ff-only"])
                    .current_dir(&repo)
                    .output_async()
                    .then(move |result| match result {
                        Ok(output) => {
                            if output.status.success() {
                                if did_pull_change(&cloned_git, &repo) {
                                    println!("{} {}:", style("Changed").yellow().bold(), path);
                                    Command::new(&cloned_git)
                                        .current_dir(&repo)
                                        .args(&["log", "--no-decorate", "--oneline", "ORIG_HEAD.."])
                                        .spawn()
                                        .unwrap()
                                        .wait()
                                        .unwrap();
                                    println!();
                                } else {
                                    println!("{} {}", style("Up-to-date").green().bold(), path);
                                }

                                Ok(true) as Result<bool, Error>
                            } else {
                                println!("{} pulling {}", style("Failed").red().bold(), path);
                                if let Ok(text) = std::str::from_utf8(&output.stderr) {
                                    print!("{}", text);
                                }
                                Ok(false)
                            }
                        }
                        Err(e) => {
                            println!("{} pulling {}: {}", style("Failed").red().bold(), path, e);
                            Ok(false)
                        }
                    })
            })
            .collect();

        let mut runtime = Runtime::new().unwrap();
        let results: Vec<bool> = runtime.block_on(join_all(futures))?;
        if results.into_iter().any(|success| !success) {
            Err(ErrorKind::StepFailed.into())
        } else {
            Ok(())
        }
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
}
