use failure::Error;
use std::env::home_dir;
use std::ffi::OsStr;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;
use which as which_mod;

#[derive(Fail, Debug)]
#[fail(display = "Process failed")]
pub struct ProcessFailed;

pub trait Check {
    fn check(self) -> Result<(), Error>;
}

impl Check for ExitStatus {
    fn check(self) -> Result<(), Error> {
        if self.success() {
            Ok(())
        } else {
            Err(Error::from(ProcessFailed {}))
        }
    }
}

pub fn home_path(p: &str) -> PathBuf {
    let mut path = home_dir().unwrap();
    path.push(p);
    path
}

pub fn is_ancestor(ancestor: &Path, path: &Path) -> bool {
    let mut p = path;
    while let Some(parent) = p.parent() {
        if parent == ancestor {
            return true;
        }

        p = parent;
    }

    false
}

pub fn which<T: AsRef<OsStr> + Debug>(binary_name: T) -> Option<PathBuf> {
    match which_mod::which(&binary_name) {
        Ok(path) => {
            debug!("Detected {:?} as {:?}", &path, &binary_name);
            Some(path)
        }
        Err(e) => {
            match e.kind() {
                which_mod::ErrorKind::CannotFindBinaryPath => {
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
