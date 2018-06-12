use super::terminal::Terminal;
use super::Check;
use failure;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use which::which;

#[derive(Copy, Clone, Debug)]
pub enum Distribution {
    Arch,
    CentOS,
    Fedora,
    Debian,
    Ubuntu,
}

#[derive(Debug, Fail)]
#[fail(display = "Unknown Linux Distribution")]
struct UnknownLinuxDistribution;

impl Distribution {
    pub fn detect() -> Result<Self, failure::Error> {
        let content = fs::read_to_string("/etc/os-release")?;

        if content.contains("Arch") {
            return Ok(Distribution::Arch);
        }

        if content.contains("CentOS") {
            return Ok(Distribution::CentOS);
        }

        if content.contains("Fedora") {
            return Ok(Distribution::Fedora);
        }

        if content.contains("Ubuntu") {
            return Ok(Distribution::Ubuntu);
        }

        if content.contains("Debian") {
            return Ok(Distribution::Debian);
        }

        Err(UnknownLinuxDistribution.into())
    }
}

pub fn upgrade_arch_linux(
    sudo: &Option<PathBuf>,
    terminal: &Terminal,
) -> Result<(), failure::Error> {
    if let Ok(yay) = which("yay") {
        Command::new(yay).spawn()?.wait()?.check()?;
    } else {
        if let Some(sudo) = &sudo {
            Command::new(&sudo)
                .args(&["pacman", "-Syu"])
                .spawn()?
                .wait()?
                .check()?;
        } else {
            terminal.print_warning("No sudo or yay detected. Skipping system upgrade");
        }
    }

    Ok(())
}

pub fn upgrade_redhat(sudo: &Option<PathBuf>, terminal: &Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["yum", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

pub fn upgrade_fedora(sudo: &Option<PathBuf>, terminal: &Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["dnf", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

pub fn upgrade_debian(sudo: &Option<PathBuf>, terminal: &Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["apt", "update"])
            .spawn()?
            .wait()?
            .check()?;

        Command::new(&sudo)
            .args(&["apt", "dist-upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}
