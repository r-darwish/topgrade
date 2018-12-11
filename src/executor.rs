use super::error::{Error, ErrorKind};
use super::utils::Check;
use failure::ResultExt;
use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

pub enum Executor {
    Wet(Command),
    Dry(DryCommand),
}

impl Executor {
    pub fn new<S: AsRef<OsStr>>(program: S, dry: bool) -> Self {
        if dry {
            Executor::Dry(DryCommand {
                program: program.as_ref().into(),
                ..Default::default()
            })
        } else {
            Executor::Wet(Command::new(program))
        }
    }

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

    pub fn current_dir<P: AsRef<Path>>(&mut self, dir: P) -> &mut Executor {
        match self {
            Executor::Wet(c) => {
                c.current_dir(dir);
            }
            Executor::Dry(c) => c.directory = Some(dir.as_ref().into()),
        }

        self
    }

    pub fn spawn(&mut self) -> Result<ExecutorChild, Error> {
        let result = match self {
            Executor::Wet(c) => c.spawn().context(ErrorKind::ProcessExecution).map(ExecutorChild::Wet)?,
            Executor::Dry(c) => {
                print!(
                    "Dry running: {} {}",
                    c.program.to_string_lossy(),
                    c.args
                        .iter()
                        .map(|a| String::from(a.to_string_lossy()))
                        .collect::<Vec<String>>()
                        .join(" ")
                );
                match &c.directory {
                    Some(dir) => println!(" in {}", dir.to_string_lossy()),
                    None => println!(),
                };
                ExecutorChild::Dry
            }
        };

        Ok(result)
    }
}

#[derive(Default)]
pub struct DryCommand {
    program: OsString,
    args: Vec<OsString>,
    directory: Option<OsString>,
}

pub enum ExecutorChild {
    Wet(Child),
    Dry,
}

impl ExecutorChild {
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
