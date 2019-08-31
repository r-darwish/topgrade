// use failure::{Backtrace, Context, Fail};
use snafu::Snafu;
// use std::fmt::{self, Display};
use std::path::PathBuf;
use std::process::ExitStatus;

#[derive(Debug, Snafu)]
#[snafu(visibility = "pub(crate)")]
pub enum Error {
    #[snafu(display("Error asking the user for retry"))]
    Retry { source: std::io::Error },

    #[snafu(display("Cannot find the user base directories"))]
    NoBaseDirectories,

    #[snafu(display("A step failed"))]
    StepFailed,

    #[snafu(display("Error reading the configuration"))]
    Configuration {
        source: Box<dyn std::error::Error>,
        config_path: PathBuf,
    },

    #[snafu(display("A custom pre-command failed"))]
    PreCommand,

    #[snafu(display("{}", status))]
    ProcessFailed { status: ExitStatus },

    #[cfg(target_os = "linux")]
    #[snafu(display("Unknown Linux Distribution"))]
    UnknownLinuxDistribution,

    #[snafu(display("Process execution failure"))]
    ProcessExecution,

    #[snafu(display("Self-update failure"))]
    #[cfg(feature = "self-update")]
    SelfUpdate,

    #[snafu(display("A step should be skipped"))]
    SkipStep,

    #[cfg(all(windows, feature = "self-update"))]
    #[snafu(display("Topgrade Upgraded"))]
    Upgraded { status: ExitStatus },
}

// impl Fail for Error {
//     fn cause(&self) -> Option<&dyn Fail> {
//         self.inner.cause()
//     }

//     fn backtrace(&self) -> Option<&Backtrace> {
//         self.inner.backtrace()
//     }
// }

// impl Display for Error {
//     fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
//         Display::fmt(&self.inner, f)
//     }
// }

// impl Error {
//     pub fn kind(&self) -> ErrorKind {
//         *self.inner.get_context()
//     }

//     #[cfg(all(windows, feature = "self-update"))]
//     pub fn upgraded(&self) -> bool {
//         if let ErrorKind::Upgraded(_) = self.kind() {
//             true
//         } else {
//             false
//         }
//     }
// }

// impl From<ErrorKind> for Error {
//     fn from(kind: ErrorKind) -> Error {
//         Error {
//             inner: Context::new(kind),
//         }
//     }
// }

// impl From<Context<ErrorKind>> for Error {
//     fn from(inner: Context<ErrorKind>) -> Error {
//         Error { inner }
//     }
// }
