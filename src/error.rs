use failure::{Backtrace, Context, Fail};
use std::fmt::{self, Display};
use std::process::ExitStatus;

#[derive(Debug)]
pub struct Error {
    inner: Context<ErrorKind>,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Fail)]
pub enum ErrorKind {
    #[fail(display = "Error asking the user for retry")]
    Retry,

    #[fail(display = "Cannot find the user base directories")]
    NoBaseDirectories,

    #[fail(display = "A step failed")]
    StepFailed,

    #[fail(display = "Error reading the configuration")]
    Configuration,

    #[fail(display = "A custom pre-command failed")]
    PreCommand,

    #[fail(display = "{}", _0)]
    ProcessFailed(ExitStatus),

    #[fail(display = "Unknown Linux Distribution")]
    #[cfg(target_os = "linux")]
    UnknownLinuxDistribution,

    #[fail(display = "Detected Python is not the system Python")]
    #[cfg(target_os = "linux")]
    NotSystemPython,

    #[fail(display = "Process execution failure")]
    ProcessExecution,

    #[fail(display = "Self-update failure")]
    #[cfg(feature = "self-update")]
    SelfUpdate,

    #[fail(display = "A step should be skipped")]
    SkipStep,
}

impl Fail for Error {
    fn cause(&self) -> Option<&Fail> {
        self.inner.cause()
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        self.inner.backtrace()
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&self.inner, f)
    }
}

impl Error {
    pub fn kind(&self) -> ErrorKind {
        *self.inner.get_context()
    }
}

impl From<ErrorKind> for Error {
    fn from(kind: ErrorKind) -> Error {
        Error {
            inner: Context::new(kind),
        }
    }
}

impl From<Context<ErrorKind>> for Error {
    fn from(inner: Context<ErrorKind>) -> Error {
        Error { inner }
    }
}
