use std::process::ExitStatus;
use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum TopgradeError {
    #[error("{0}")]
    ProcessFailed(ExitStatus),

    #[error("Sudo is required for this step")]
    #[allow(dead_code)]
    SudoRequired,

    #[error("Unknown Linux Distribution")]
    #[cfg(target_os = "linux")]
    UnknownLinuxDistribution,
}

#[derive(Error, Debug)]
#[error("A step failed")]
pub struct StepFailed;

#[derive(Error, Debug)]
#[error("A step should be skipped")]
pub struct SkipStep;

#[cfg(all(windows, feature = "self-update"))]
#[derive(Error, Debug)]
#[error("Topgrade Upgraded")]
pub struct Upgraded(pub ExitStatus);
