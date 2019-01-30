use super::error::{Error, ErrorKind};
use log::{debug, error};
use std::ffi::OsStr;
use std::fmt::{self, Debug};
use std::path::{Component, Path, PathBuf};
use std::process::{ExitStatus, Output};
use which_crate;

pub trait Check {
    fn check(self) -> Result<(), Error>;
}

impl Check for ExitStatus {
    fn check(self) -> Result<(), Error> {
        if self.success() {
            Ok(())
        } else {
            Err(ErrorKind::ProcessFailed(self))?
        }
    }
}

impl Check for Output {
    fn check(self) -> Result<(), Error> {
        self.status.check()
    }
}

pub trait PathExt
where
    Self: Sized,
{
    fn if_exists(self) -> Option<Self>;
    fn is_descendant_of(&self, ancestor: &Path) -> bool;
}

impl PathExt for PathBuf {
    fn if_exists(self) -> Option<Self> {
        if self.exists() {
            Some(self)
        } else {
            None
        }
    }

    fn is_descendant_of(&self, ancestor: &Path) -> bool {
        self.iter().zip(ancestor.iter()).all(|(a, b)| a == b)
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

/// `std::fmt::Display` implementation for `std::path::Path`.
///
/// This struct differs from `std::path::Display` in that in Windows it takes care of printing slashes
/// instead of backslashes and don't print the `\\?` prefix in long paths.
pub struct HumanizedPath<'a> {
    path: &'a Path,
}

impl<'a> From<&'a Path> for HumanizedPath<'a> {
    fn from(path: &'a Path) -> Self {
        Self { path }
    }
}

impl<'a> fmt::Display for HumanizedPath<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if cfg!(windows) {
            let mut iterator = self.path.components().peekable();

            while let Some(component) = iterator.next() {
                let is_prefix = if let Component::RootDir = &component {
                    true
                } else {
                    false
                };

                let printed = match &component {
                    Component::Normal(c) if *c == "?" => false,
                    Component::RootDir | Component::CurDir => false,
                    Component::ParentDir => {
                        write!(f, "..")?;
                        true
                    }
                    Component::Prefix(p) => {
                        write!(f, "{}", p.as_os_str().to_string_lossy())?;
                        true
                    }
                    Component::Normal(c) => {
                        write!(f, "{}", c.to_string_lossy())?;
                        true
                    }
                };

                if printed && (iterator.peek().is_some() || is_prefix) {
                    write!(f, "{}", std::path::MAIN_SEPARATOR)?;
                }
            }
        } else {
            write!(f, "{}", self.path.display())?;
        }

        Ok(())
    }
}
