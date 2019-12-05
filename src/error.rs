use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TopgradeError {
    #[error("Error asking the user for retry")]
    Retry,

    #[error("Cannot find the user base directories")]
    NoBaseDirectories,

    #[error("A step failed")]
    StepFailed,

    #[error("Error reading the configuration")]
    Configuration,

    #[error("A custom pre-command failed")]
    PreCommand,

    #[error("{0}")]
    ProcessFailed(ExitStatus),

    #[error("Unknown Linux Distribution")]
    #[cfg(target_os = "linux")]
    UnknownLinuxDistribution,

    #[error("Process execution failure")]
    ProcessExecution,

    #[error("Self-update failure")]
    #[cfg(feature = "self-update")]
    SelfUpdate,

    #[error("A step should be skipped")]
    SkipStep,

    #[cfg(all(windows, feature = "self-update"))]
    #[error("Topgrade Upgraded")]
    Upgraded(ExitStatus),
}
