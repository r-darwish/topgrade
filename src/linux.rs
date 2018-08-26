use super::terminal::Terminal;
use super::utils::{which, Check};
use failure;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

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

#[derive(Debug, Fail)]
#[fail(display = "Detected Python is not the system Python")]
struct NotSystemPython;

impl Distribution {
    pub fn detect() -> Result<Self, failure::Error> {
        let content = fs::read_to_string("/etc/os-release")?;

        if content.contains("Arch") | content.contains("Manjaro") | content.contains("Antergos") {
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

fn upgrade_arch_linux(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Result<(), failure::Error> {
    if let Some(yay) = which("yay") {
        if let Some(python) = which("python") {
            if python != PathBuf::from("/usr/bin/python") {
                terminal.print_warning(format!(
                    "Python detected at {:?}, which is probably not the system Python.
It's dangerous to run yay since Python based AUR packages will be installed in the wrong location",
                    python
                ));
                return Err(NotSystemPython.into());
            }
        }

        Command::new(yay).spawn()?.wait()?.check()?;
    } else if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["/usr/bin/pacman", "-Syu"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo or yay detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_redhat(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["/usr/bin/yum", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_fedora(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["/usr/bin/dnf", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_debian(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Result<(), failure::Error> {
    if let Some(sudo) = &sudo {
        Command::new(&sudo)
            .args(&["/usr/bin/apt", "update"])
            .spawn()?
            .wait()?
            .check()?;

        Command::new(&sudo)
            .args(&["/usr/bin/apt", "dist-upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        terminal.print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

#[must_use]
pub fn upgrade(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    terminal.print_separator("System update");

    let success = match Distribution::detect() {
        Ok(distribution) => match distribution {
            Distribution::Arch => upgrade_arch_linux(&sudo, terminal),
            Distribution::CentOS => upgrade_redhat(&sudo, terminal),
            Distribution::Fedora => upgrade_fedora(&sudo, terminal),
            Distribution::Ubuntu | Distribution::Debian => upgrade_debian(&sudo, terminal),
        }.is_ok(),
        Err(e) => {
            println!("Error detecting current distribution: {}", e);
            false
        }
    };

    Some(("System update", success))
}

#[must_use]
pub fn run_needrestart(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(needrestart) = which("needrestart") {
            terminal.print_separator("Check for needed restarts");

            let success = || -> Result<(), failure::Error> {
                Command::new(&sudo).arg(needrestart).spawn()?.wait()?.check()?;

                Ok(())
            }().is_ok();

            return Some(("Restarts", success));
        }
    }

    None
}

#[must_use]
pub fn run_fwupdmgr(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(fwupdmgr) = which("fwupdmgr") {
        terminal.print_separator("Firmware upgrades");

        let success = || -> Result<(), failure::Error> {
            Command::new(&fwupdmgr).arg("refresh").spawn()?.wait()?.check()?;
            Command::new(&fwupdmgr).arg("get-updates").spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("Firmware upgrade", success));
    }

    None
}

#[must_use]
pub fn run_flatpak(terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(flatpak) = which("flatpak") {
        terminal.print_separator("Flatpak");

        let success = || -> Result<(), failure::Error> {
            Command::new(&flatpak).args(&["update", "-y"]).spawn()?.wait()?.check()?;
            Ok(())
        }().is_ok();

        return Some(("Flatpak", success));
    }

    None
}

#[must_use]
pub fn run_snap(sudo: &Option<PathBuf>, terminal: &mut Terminal) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(snap) = which("snap") {
            if PathBuf::from("/var/snapd.socket").exists() {
                terminal.print_separator("snap");

                let success = || -> Result<(), failure::Error> {
                    Command::new(&sudo)
                        .args(&[snap.to_str().unwrap(), "refresh"])
                        .spawn()?
                        .wait()?
                        .check()?;

                    Ok(())
                }().is_ok();

                return Some(("snap", success));
            }
        }
    }

    None
}
