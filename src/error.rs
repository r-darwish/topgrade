use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TopgradeError {
    #[error("A step failed")]
    StepFailed,

    #[error("{0}")]
    ProcessFailed(ExitStatus),

    #[error("Unknown Linux Distribution")]
    #[cfg(target_os = "linux")]
    UnknownLinuxDistribution,
}

#[derive(Error, Debug)]
#[error("A step should be skipped")]
pub struct SkipStep;

#[cfg(all(windows, feature = "self-update"))]
#[derive(Error, Debug)]
#[error("Topgrade Upgraded")]
pub struct Upgraded(pub ExitStatus);
