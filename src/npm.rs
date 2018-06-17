use super::utils::Check;
use failure;
use std::path::PathBuf;
use std::process::Command;

pub struct NPM {
    command: PathBuf,
}

impl NPM {
    pub fn new(command: PathBuf) -> Self {
        Self { command }
    }

    pub fn root(&self) -> Result<PathBuf, failure::Error> {
        let output = Command::new(&self.command).args(&["root", "-g"]).output()?;

        output.status.check()?;

        Ok(PathBuf::from(&String::from_utf8(output.stdout)?))
    }

    pub fn upgrade(&self) -> Result<(), failure::Error> {
        Command::new(&self.command)
            .args(&["update", "-g"])
            .spawn()?
            .wait()?
            .check()?;

        Ok(())
    }
}
