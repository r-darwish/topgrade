use crate::error::{Error, ErrorKind};
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{which, HumanizedPath};
use console::style;
use futures::future::{join_all, Future};
use glob::{glob_with, MatchOptions};
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

pub struct Repositories<'a> {
    git: &'a Git,
    repositories: HashSet<String>,
    glob_match_options: MatchOptions,
}

fn get_head_revision(git: &Path, repo: &str) -> Option<String> {
    Command::new(git)
        .args(&["rev-parse", "HEAD"])
        .current_dir(repo)
        .check_output()
        .map(|output| output.trim().to_string())
        .map_err(|e| {
            error!("Error getting revision for {}: {}", repo, e);

            e
        })
        .ok()
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

    pub fn multi_pull(
        &self,
        repositories: &Repositories,
        run_type: RunType,
        extra_arguments: &Option<String>,
    ) -> Result<(), Error> {
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
                let before_revision = get_head_revision(git, &repo);
                let cloned_git = git.to_owned();

                println!("{} {}", style("Pulling").cyan().bold(), path);

                let mut command = Command::new(git);

                command.args(&["pull", "--ff-only"]).current_dir(&repo);

                if let Some(extra_arguments) = extra_arguments {
                    command.args(extra_arguments.split_whitespace());
                }

                command.output_async().then(move |result| match result {
                    Ok(output) => {
                        if output.status.success() {
                            let after_revision = get_head_revision(&cloned_git, &repo);

                            match (&before_revision, &after_revision) {
                                (Some(before), Some(after)) if before != after => {
                                    println!("{} {}:", style("Changed").yellow().bold(), path);
                                    Command::new(&cloned_git)
                                        .current_dir(&repo)
                                        .args(&[
                                            "--no-pager",
                                            "log",
                                            "--no-decorate",
                                            "--oneline",
                                            &format!("{}..{}", before, after),
                                        ])
                                        .spawn()
                                        .unwrap()
                                        .wait()
                                        .unwrap();
                                    println!();
                                }
                                _ => {
                                    println!("{} {}", style("Up-to-date").green().bold(), path);
                                }
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
        let mut glob_match_options = MatchOptions::new();

        if cfg!(windows) {
            glob_match_options.case_sensitive = false;
        }

        Self {
            git,
            repositories: HashSet::new(),
            glob_match_options,
        }
    }

    pub fn insert<P: AsRef<Path>>(&mut self, path: P) {
        if let Some(repo) = self.git.get_repo_root(path) {
            self.repositories.insert(repo);
        }
    }

    pub fn glob_insert(&mut self, pattern: &str) {
        if let Ok(glob) = glob_with(pattern, self.glob_match_options) {
            for entry in glob {
                match entry {
                    Ok(path) => self.insert(path),
                    Err(e) => {
                        error!("Error in path {}", e);
                    }
                }
            }
        } else {
            error!("Bad glob pattern: {}", pattern);
        }
    }
}
