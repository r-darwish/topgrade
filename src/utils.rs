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

    /// Returns the path if it exists or ErrorKind::SkipStep otherwise
    fn require(self) -> Result<Self, Error>;
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

    fn require(self) -> Result<Self, Error> {
        if self.exists() {
            Ok(self)
        } else {
            Err(ErrorKind::SkipStep)?
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

#[cfg(any(target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
pub fn sudo() -> Option<PathBuf> {
    which("sudo").or_else(|| which("pkexec"))
}

/// `std::fmt::Display` implementation for `std::path::Path`.
///
/// This struct differs from `std::path::Display` in that in Windows it takes care of printing backslashes
/// instead of slashes and don't print the `\\?` prefix in long paths.
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
                let mut print_seperator = iterator.peek().is_some();

                match &component {
                    Component::Normal(c) if *c == "?" => {
                        print_seperator = false;
                    }
                    Component::RootDir | Component::CurDir => {
                        print_seperator = false;
                    }
                    Component::ParentDir => {
                        write!(f, "..")?;
                    }
                    Component::Prefix(p) => {
                        write!(f, "{}", p.as_os_str().to_string_lossy())?;
                        print_seperator = true;
                    }
                    Component::Normal(c) => {
                        write!(f, "{}", c.to_string_lossy())?;
                    }
                };

                if print_seperator {
                    write!(f, "{}", std::path::MAIN_SEPARATOR)?;
                }
            }
        } else {
            write!(f, "{}", self.path.display())?;
        }

        Ok(())
    }
}

#[cfg(test)]
#[cfg(windows)]
mod tests {
    use super::*;

    fn humanize<P: AsRef<Path>>(path: P) -> String {
        format!("{}", HumanizedPath::from(path.as_ref()))
    }

    #[test]
    fn test_just_drive() {
        assert_eq!("C:\\", humanize("C:\\"));
    }

    #[test]
    fn test_path() {
        assert_eq!("C:\\hi", humanize("C:\\hi"));
    }

    #[test]
    fn test_unc() {
        assert_eq!("\\\\server\\share\\", humanize("\\\\server\\share"));
    }

    #[test]
    fn test_long_path() {
        assert_eq!("C:\\hi", humanize("//?/C:/hi"));
    }
}

pub fn require<T: AsRef<OsStr> + Debug>(binary_name: T) -> Result<PathBuf, Error> {
    match which_crate::which(&binary_name) {
        Ok(path) => {
            debug!("Detected {:?} as {:?}", &path, &binary_name);
            Ok(path)
        }
        Err(e) => match e.kind() {
            which_crate::ErrorKind::CannotFindBinaryPath => {
                debug!("Cannot find {:?}", &binary_name);
                Err(ErrorKind::SkipStep)?
            }
            _ => {
                panic!("Detecting {:?} failed: {}", &binary_name, e);
            }
        },
    }
}

#[allow(dead_code)]
pub fn require_option<T>(option: Option<T>) -> Result<T, Error> {
    if let Some(value) = option {
        Ok(value)
    } else {
        Err(ErrorKind::SkipStep)?
    }
}
