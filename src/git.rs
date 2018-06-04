use failure::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use which::which;

pub struct Git {
    git: Option<PathBuf>,
}

impl Git {
    pub fn new() -> Self {
        Self {
            git: which("git").ok(),
        }
    }

    pub fn get_repo_root<P: AsRef<Path>>(&self, path: P) -> Option<String> {
        if !path.as_ref().exists() {
            return None;
        }

        if let Some(git) = &self.git {
            let output = Command::new(&git)
                .arg("rev-parse")
                .arg("--show-toplevel")
                .current_dir(path)
                .output();

            if let Ok(output) = output {
                if !output.status.success() {
                    return None;
                }

                return Some(String::from_utf8_lossy(&output.stdout).trim().to_string());
            }
        }

        None
    }

    pub fn pull<P: AsRef<Path>>(&self, path: P) -> Result<(), Error> {
        if let Some(git) = &self.git {
            if let Ok(mut command) = Command::new(&git)
                .arg("pull")
                .arg("--rebase")
                .arg("--autostash")
                .current_dir(path)
                .spawn()
            {
                command.wait()?;
            }
        }

        Ok(())
    }
}
