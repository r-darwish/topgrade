use std::collections::HashSet;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use anyhow::Result;
use console::style;
use futures::stream::futures_unordered::FuturesUnordered;
use futures::StreamExt;
use glob::{glob_with, MatchOptions};
use log::{debug, error};
use tokio::process::Command as AsyncCommand;
use tokio::runtime;

use crate::error::SkipStep;
use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{which, PathExt};

#[cfg(windows)]
static PATH_PREFIX: &str = "\\\\?\\";

#[derive(Debug)]
pub struct Git {
    git: Option<PathBuf>,
}

pub struct Repositories<'a> {
    git: &'a Git,
    repositories: HashSet<String>,
    glob_match_options: MatchOptions,
}

fn check_output(output: Output) -> std::result::Result<(), String> {
    if !(output.status.success()) {
        let stderr = String::from_utf8(output.stderr).unwrap();
        Err(stderr)
    } else {
        Ok(())
    }
}

async fn pull_repository(repo: String, git: &PathBuf, ctx: &ExecutionContext<'_>) -> Result<()> {
    let path = repo.to_string();
    let before_revision = get_head_revision(git, &repo);

    println!("{} {}", style("Pulling").cyan().bold(), path);

    let mut command = AsyncCommand::new(git);

    command.args(&["pull", "--ff-only"]).current_dir(&repo);

    if let Some(extra_arguments) = ctx.config().git_arguments() {
        command.args(extra_arguments.split_whitespace());
    }

    let pull_output = command.output().await?;
    let submodule_output = AsyncCommand::new(git)
        .args(&["submodule", "update", "--recursive"])
        .current_dir(&repo)
        .output()
        .await?;
    let result = check_output(pull_output).and_then(|_| check_output(submodule_output));

    if let Err(message) = result {
        println!("{} pulling {}", style("Failed").red().bold(), &repo);
        print!("{}", message);
    } else {
        let after_revision = get_head_revision(&git, &repo);

        match (&before_revision, &after_revision) {
            (Some(before), Some(after)) if before != after => {
                println!("{} {}:", style("Changed").yellow().bold(), &repo);

                Command::new(&git)
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
                println!("{} {}", style("Up-to-date").green().bold(), &repo);
            }
        }
    }

    Ok(())
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

fn has_remotes(git: &Path, repo: &str) -> Option<bool> {
    Command::new(git)
        .args(&["remote", "show"])
        .current_dir(repo)
        .check_output()
        .map(|output| output.lines().count() > 0)
        .map_err(|e| {
            error!("Error getting remotes for {}: {}", repo, e);
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

                #[cfg(windows)]
                let path = {
                    let mut path_string = path.into_os_string().to_string_lossy().into_owned();
                    if path_string.starts_with(PATH_PREFIX) {
                        path_string.replace_range(0..PATH_PREFIX.len(), "");
                    }

                    debug!("Transformed path to {}", path_string);

                    path_string
                };

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
    pub fn multi_pull_step(&self, repositories: &Repositories, ctx: &ExecutionContext) -> Result<()> {
        if repositories.repositories.is_empty() {
            return Err(SkipStep.into());
        }

        print_separator("Git repositories");
        self.multi_pull(repositories, ctx)
    }

    pub fn multi_pull(&self, repositories: &Repositories, ctx: &ExecutionContext) -> Result<()> {
        let git = self.git.as_ref().unwrap();

        if let RunType::Dry = ctx.run_type() {
            repositories
                .repositories
                .iter()
                .for_each(|repo| println!("Would pull {}", &repo));

            return Ok(());
        }

        let mut stream = repositories
            .repositories
            .iter()
            .filter(|repo| match has_remotes(git, repo) {
                Some(false) => {
                    println!(
                        "{} {} because it has no remotes",
                        style("Skipping").yellow().bold(),
                        repo
                    );
                    false
                }
                _ => true, // repo has remotes or command to check for remotes has failed. proceed to pull anyway.
            })
            .map(|repo| pull_repository(repo.clone(), &git, ctx))
            .collect::<FuturesUnordered<_>>()
            .boxed();

        if let Some(limit) = ctx.config().git_concurrency_limit() {
            stream = stream.buffer_unordered(limit).boxed();
        }

        let mut basic_rt = runtime::Runtime::new()?;
        let results = basic_rt.block_on(async { stream.collect::<Vec<Result<()>>>().await });

        let error = results.into_iter().find(|r| r.is_err());
        error.unwrap_or(Ok(()))
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

    pub fn insert_if_repo<P: AsRef<Path>>(&mut self, path: P) -> bool {
        if let Some(repo) = self.git.get_repo_root(path) {
            self.repositories.insert(repo);
            true
        } else {
            false
        }
    }

    pub fn glob_insert(&mut self, pattern: &str) {
        if let Ok(glob) = glob_with(pattern, self.glob_match_options) {
            let mut last_git_repo: Option<PathBuf> = None;
            for entry in glob {
                match entry {
                    Ok(path) => {
                        if let Some(last_git_repo) = &last_git_repo {
                            if path.is_descendant_of(&last_git_repo) {
                                debug!(
                                    "Skipping {} because it's a decendant of last known repo {}",
                                    path.display(),
                                    last_git_repo.display()
                                );
                                continue;
                            }
                        }
                        if self.insert_if_repo(&path) {
                            last_git_repo = Some(path);
                        }
                    }
                    Err(e) => {
                        error!("Error in path {}", e);
                    }
                }
            }
        } else {
            error!("Bad glob pattern: {}", pattern);
        }
    }

    #[cfg(unix)]
    pub fn is_empty(&self) -> bool {
        self.repositories.is_empty()
    }

    #[cfg(unix)]
    pub fn remove(&mut self, path: &str) {
        let _removed = self.repositories.remove(path);
        debug_assert!(_removed);
    }
}
