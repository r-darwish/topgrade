//! Utilities for command execution
use super::error::{Error, ErrorKind};
use super::utils::Check;
use failure::ResultExt;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

/// An enum telling whether Topgrade should perform dry runs or actually perform the steps.
#[derive(Clone, Copy, Debug)]
pub enum RunType {
    /// Executing commands will just print the command with its argument.
    Dry,

    /// Executing commands will perform actual execution.
    Wet,
}

impl RunType {
    /// Create a new instance from a boolean telling whether to dry run.
    pub fn new(dry_run: bool) -> Self {
        if dry_run {
            RunType::Dry
        } else {
            RunType::Wet
        }
    }

    /// Create an instance of `Executor` that should run `program`.
    pub fn execute<S: AsRef<OsStr>>(self, program: S) -> Executor {
        match self {
            RunType::Dry => Executor::Dry(DryCommand {
                program: program.as_ref().into(),
                ..Default::default()
            }),
            RunType::Wet => Executor::Wet(Command::new(program)),
        }
    }

    #[cfg(feature = "self-update")]
    /// Tells whether we're performing a dry run.
    pub fn dry(self) -> bool {
        match self {
            RunType::Dry => true,
            RunType::Wet => false,
        }
    }
}

/// An enum providing a similar interface to `std::process::Command`.
/// If the enum is set to `Wet`, execution will be performed with `std::process::Command`.
/// If the enum is set to `Dry`, execution will just print the command with its arguments.
pub enum Executor {
    Wet(Command),
    Dry(DryCommand),
}

impl Executor {
    /// See `std::process::Command::arg`
    pub fn arg<S: AsRef<OsStr>>(&mut self, arg: S) -> &mut Executor {
        match self {
            Executor::Wet(c) => {
                c.arg(arg);
            }
            Executor::Dry(c) => {
                c.args.push(arg.as_ref().into());
            }
        }

        self
    }

    /// See `std::process::Command::args`
    pub fn args<I, S>(&mut self, args: I) -> &mut Executor
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        match self {
            Executor::Wet(c) => {
                c.args(args);
            }
            Executor::Dry(c) => {
                c.args.extend(args.into_iter().map(|arg| arg.as_ref().into()));
            }
        }

        self
    }

    /// See `std::process::Command::current_dir`
    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Executor {
        match self {
            Executor::Wet(c) => {
                c.current_dir(dir);
            }
            Executor::Dry(c) => c.directory = Some(dir.as_ref().into()),
        }

        self
    }

    /// See `std::process::Command::spawn`
    pub fn spawn(&mut self) -> Result<ExecutorChild, Error> {
        let result = match self {
            Executor::Wet(c) => c.spawn().context(ErrorKind::ProcessExecution).map(ExecutorChild::Wet)?,
            Executor::Dry(c) => {
                c.dry_run();
                ExecutorChild::Dry
            }
        };

        Ok(result)
    }

    /// See `std::process::Command::output`
    pub fn output(&mut self) -> Result<ExecutorOutput, Error> {
        match self {
            Executor::Wet(c) => Ok(ExecutorOutput::Wet(c.output().context(ErrorKind::ProcessExecution)?)),
            Executor::Dry(c) => {
                c.dry_run();
                Ok(ExecutorOutput::Dry)
            }
        }
    }

    /// A convinence method for `spawn().wait().check()`.
    /// Returns an error if something went wrong during the execution or if the
    /// process exited with failure.
    pub fn check_run(&mut self) -> Result<(), Error> {
        self.spawn()?.wait()?.check()
    }
}

pub enum ExecutorOutput {
    Wet(std::process::Output),
    Dry,
}

/// A struct represending a command. Trying to execute it will just print its arguments.
#[derive(Default)]
pub struct DryCommand {
    program: OsString,
    args: Vec<OsString>,
    directory: Option<OsString>,
}

impl DryCommand {
    fn dry_run(&self) {
        print!(
            "Dry running: {} {}",
            self.program.to_string_lossy(),
            self.args
                .iter()
                .map(|a| String::from(a.to_string_lossy()))
                .collect::<Vec<String>>()
                .join(" ")
        );
        match &self.directory {
            Some(dir) => println!(" in {}", dir.to_string_lossy()),
            None => println!(),
        };
    }
}

/// The Result of spawn. Contains an actual `std::process::Child` if executed by a wet command.
pub enum ExecutorChild {
    Wet(Child),
    Dry,
}

impl ExecutorChild {
    /// See `std::process::Child::wait`
    pub fn wait(&mut self) -> Result<ExecutorExitStatus, Error> {
        let result = match self {
            ExecutorChild::Wet(c) => c
                .wait()
                .context(ErrorKind::ProcessExecution)
                .map(ExecutorExitStatus::Wet)?,
            ExecutorChild::Dry => ExecutorExitStatus::Dry,
        };

        Ok(result)
    }
}

/// The Result of wait. Contains an actual `std::process::ExitStatus` if executed by a wet command.
pub enum ExecutorExitStatus {
    Wet(ExitStatus),
    Dry,
}

impl Check for ExecutorExitStatus {
    fn check(self) -> Result<(), Error> {
        match self {
            ExecutorExitStatus::Wet(e) => e.check(),
            ExecutorExitStatus::Dry => Ok(()),
        }
    }
}

/// Extension methods for `std::process::Command`
pub trait CommandExt {
    /// Run the command, wait for it to complete, check the return code and decode the output as UTF-8.
    fn check_output(&mut self) -> Result<String, Error>;
}

impl CommandExt for Command {
    fn check_output(&mut self) -> Result<String, Error> {
        let output = self.output().context(ErrorKind::ProcessExecution)?;
        let status = output.status;
        if !status.success() {
            Err(ErrorKind::ProcessFailed(status))?
        }
        Ok(String::from_utf8(output.stdout).context(ErrorKind::ProcessExecution)?)
    }
}
