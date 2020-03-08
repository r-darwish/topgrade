#![allow(dead_code)]
use crate::config::Config;
use crate::executor::RunType;
use crate::git::Git;
use directories::BaseDirs;
#[cfg(unix)]
use std::path::PathBuf;

pub struct ExecutionContext<'a> {
    run_type: RunType,
    #[cfg(unix)]
    sudo: &'a Option<PathBuf>,
    git: &'a Git,
    config: &'a Config,
    base_dirs: &'a BaseDirs,
}

impl<'a> ExecutionContext<'a> {
    #[cfg(unix)]
    pub fn new(
        run_type: RunType,
        sudo: &'a Option<PathBuf>,
        git: &'a Git,
        config: &'a Config,
        base_dirs: &'a BaseDirs,
    ) -> ExecutionContext<'a> {
        ExecutionContext {
            run_type,
            sudo,
            git,
            config,
            base_dirs,
        }
    }

    #[cfg(not(unix))]
    pub fn new(run_type: RunType, git: &'a Git, config: &'a Config, base_dirs: &'a BaseDirs) -> ExecutionContext<'a> {
        ExecutionContext {
            run_type,
            config,
            git,
            base_dirs,
        }
    }

    pub fn run_type(&self) -> RunType {
        self.run_type
    }

    pub fn git(&self) -> &Git {
        &self.git
    }

    #[cfg(unix)]
    pub fn sudo(&self) -> &Option<PathBuf> {
        &self.sudo
    }

    pub fn config(&self) -> &Config {
        &self.config
    }

    pub fn base_dirs(&self) -> &BaseDirs {
        &self.base_dirs
    }
}
