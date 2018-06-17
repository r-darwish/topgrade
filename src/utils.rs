use failure::Error;
use std::env::home_dir;
use std::path::{Path, PathBuf};
use std::process::ExitStatus;

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
