#![allow(dead_code)]
use crate::config::Config;
use crate::executor::RunType;
use directories::BaseDirs;
#[cfg(unix)]
use std::path::PathBuf;

pub struct ExecutionContext<'a> {
    run_type: RunType,
    #[cfg(unix)]
    sudo: &'a Option<PathBuf>,
    config: &'a Config,
    base_dirs: &'a BaseDirs,
}

impl<'a> ExecutionContext<'a> {
    #[cfg(unix)]
    pub fn new(
        run_type: RunType,
        sudo: &'a Option<PathBuf>,
        config: &'a Config,
        base_dirs: &'a BaseDirs,
    ) -> ExecutionContext<'a> {
        ExecutionContext {
            run_type,
            sudo,
            config,
            base_dirs,
        }
    }

    #[cfg(not(unix))]
    pub fn new(run_type: RunType, config: &'a Config, base_dirs: &'a BaseDirs) -> ExecutionContext<'a> {
        ExecutionContext {
            run_type,
            config,
            base_dirs,
        }
    }

    pub fn run_type(&self) -> RunType {
        self.run_type
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
