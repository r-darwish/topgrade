use super::utils::Check;
use failure;
use std::ffi::{OsStr, OsString};
use std::io;
use std::path::Path;
use std::process::{Child, Command, ExitStatus};

pub enum Executor {
    Wet(Command),
    Dry(DryCommand),
}

impl Executor {
    pub fn new<S: AsRef<OsStr>>(program: S, dry: bool) -> Self {
        match dry {
            false => Executor::Wet(Command::new(program)),
            true => Executor::Dry(DryCommand {
                program: program.as_ref().into(),
                ..Default::default()
            }),
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

    pub fn spawn(&mut self) -> Result<ExecutorChild, io::Error> {
        match self {
            Executor::Wet(c) => c.spawn().map(|c| ExecutorChild::Wet(c)),
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
                Ok(ExecutorChild::Dry)
            }
        }
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
    pub fn wait(&mut self) -> Result<ExecutorExitStatus, io::Error> {
        match self {
            ExecutorChild::Wet(c) => c.wait().map(|s| ExecutorExitStatus::Wet(s)),
            ExecutorChild::Dry => Ok(ExecutorExitStatus::Dry),
        }
    }
}

pub enum ExecutorExitStatus {
    Wet(ExitStatus),
    Dry,
}

impl Check for ExecutorExitStatus {
    fn check(self) -> Result<(), failure::Error> {
        match self {
            ExecutorExitStatus::Wet(e) => e.check(),
            ExecutorExitStatus::Dry => Ok(()),
        }
    }
}
