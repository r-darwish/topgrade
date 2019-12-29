use crate::error::{SkipStep, TopgradeError};
use anyhow::Result;

use log::{debug, error};
use std::env;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::{ExitStatus, Output};
use which_crate;

pub trait Check {
    fn check(self) -> Result<()>;
}

impl Check for ExitStatus {
    fn check(self) -> Result<()> {
        if self.success() {
            Ok(())
        } else {
            Err(TopgradeError::ProcessFailed(self).into())
        }
    }
}

impl Check for Output {
    fn check(self) -> Result<()> {
        self.status.check()
    }
}

pub trait PathExt
where
    Self: Sized,
{
    fn if_exists(self) -> Option<Self>;
    fn is_descendant_of(&self, ancestor: &Path) -> bool;

    /// Returns the path if it exists or ErrorKind::SkipStep otherwise
    fn require(self) -> Result<Self>;
}

impl<T> PathExt for T
where
    T: AsRef<Path>,
{
    fn if_exists(self) -> Option<Self> {
        if self.as_ref().exists() {
            Some(self)
        } else {
            None
        }
    }

    fn is_descendant_of(&self, ancestor: &Path) -> bool {
        self.as_ref().iter().zip(ancestor.iter()).all(|(a, b)| a == b)
    }

    fn require(self) -> Result<Self> {
        if self.as_ref().exists() {
            debug!("Path {:?} exists", self.as_ref());
            Ok(self)
        } else {
            debug!("Path {:?} doesn't exist", self.as_ref());
            Err(SkipStep.into())
        }
    }
}

pub fn which<T: AsRef<OsStr> + Debug>(binary_name: T) -> Option<PathBuf> {
    match which_crate::which(&binary_name) {
        Ok(path) => {
            debug!("Detected {:?} as {:?}", &path, &binary_name);
            Some(path)
        }
        Err(e) => {
            match e.kind() {
                which_crate::ErrorKind::CannotFindBinaryPath => {
                    debug!("Cannot find {:?}", &binary_name);
                }
                _ => {
                    error!("Detecting {:?} failed: {}", &binary_name, e);
                }
            }

            None
        }
    }
}

#[cfg(unix)]
pub fn sudo() -> Option<PathBuf> {
    which("sudo").or_else(|| which("pkexec"))
}

pub fn editor() -> String {
    env::var("EDITOR").unwrap_or_else(|_| String::from(if cfg!(windows) { "notepad" } else { "vi" }))
}

pub fn require<T: AsRef<OsStr> + Debug>(binary_name: T) -> Result<PathBuf> {
    match which_crate::which(&binary_name) {
        Ok(path) => {
            debug!("Detected {:?} as {:?}", &path, &binary_name);
            Ok(path)
        }
        Err(e) => match e.kind() {
            which_crate::ErrorKind::CannotFindBinaryPath => {
                debug!("Cannot find {:?}", &binary_name);
                Err(SkipStep.into())
            }
            _ => {
                panic!("Detecting {:?} failed: {}", &binary_name, e);
            }
        },
    }
}

#[allow(dead_code)]
pub fn require_option<T>(option: Option<T>) -> Result<T> {
    if let Some(value) = option {
        Ok(value)
    } else {
        Err(SkipStep.into())
    }
}
