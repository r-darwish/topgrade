use super::error::{Error, ErrorKind};
use super::executor::Executor;
use super::terminal::{print_separator, print_warning};
use super::utils::{which, Check};
use failure::ResultExt;
use std::fs;
use std::path::PathBuf;
use walkdir::WalkDir;

#[derive(Copy, Clone, Debug)]
pub enum Distribution {
    Arch,
    CentOS,
    Fedora,
    Debian,
    Ubuntu,
    Gentoo,
    OpenSuse,
    Void,
}

impl Distribution {
    pub fn detect() -> Result<Self, Error> {
        let content = fs::read_to_string("/etc/os-release").context(ErrorKind::UnknownLinuxDistribution)?;

        if content.contains("Arch") | content.contains("Manjaro") | content.contains("Antergos") {
            return Ok(Distribution::Arch);
        }

        if content.contains("CentOS") || content.contains("Oracle Linux") {
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

        if content.contains("openSUSE") {
            return Ok(Distribution::OpenSuse);
        }

        if content.contains("void") {
            return Ok(Distribution::Void);
        }

        if PathBuf::from("/etc/gentoo-release").exists() {
            return Ok(Distribution::Gentoo);
        }

        Err(ErrorKind::UnknownLinuxDistribution)?
    }

    #[must_use]
    pub fn upgrade(self, sudo: &Option<PathBuf>, cleanup: bool, dry_run: bool) -> Option<(&'static str, bool)> {
        print_separator("System update");

        let success = match self {
            Distribution::Arch => upgrade_arch_linux(&sudo, dry_run),
            Distribution::CentOS => upgrade_redhat(&sudo, dry_run),
            Distribution::Fedora => upgrade_fedora(&sudo, dry_run),
            Distribution::Ubuntu | Distribution::Debian => upgrade_debian(&sudo, cleanup, dry_run),
            Distribution::Gentoo => upgrade_gentoo(&sudo, dry_run),
            Distribution::OpenSuse => upgrade_opensuse(&sudo, dry_run),
            Distribution::Void => upgrade_void(&sudo, dry_run),
        };

        Some(("System update", success.is_ok()))
    }

    pub fn show_summary(self) {
        if let Distribution::Arch = self {
            show_pacnew();
        }
    }
}

pub fn show_pacnew() {
    let mut iter = WalkDir::new("/etc")
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|f| {
            f.path()
                .extension()
                .filter(|ext| ext == &"pacnew" || ext == &"pacsave")
                .is_some()
        })
        .peekable();

    if iter.peek().is_some() {
        println!("\nPacman backup configuration files found:");

        for entry in iter {
            println!("{}", entry.path().display());
        }
    }
}

fn upgrade_arch_linux(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(yay) = which("yay") {
        if let Some(python) = which("python") {
            if python != PathBuf::from("/usr/bin/python") {
                print_warning(format!(
                    "Python detected at {:?}, which is probably not the system Python.
It's dangerous to run yay since Python based AUR packages will be installed in the wrong location",
                    python
                ));
                return Err(ErrorKind::NotSystemPython)?;
            }
        }

        Executor::new(yay, dry_run).spawn()?.wait()?.check()?;
    } else if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/pacman", "-Syu"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo or yay detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_redhat(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/yum", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_opensuse(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/zypper", "refresh"])
            .spawn()?
            .wait()?
            .check()?;

        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/zypper", "dist-upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_void(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/xbps-install", "-Su"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_fedora(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/dnf", "upgrade"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_gentoo(sudo: &Option<PathBuf>, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        if let Some(layman) = which("layman") {
            Executor::new(&sudo, dry_run)
                .arg(layman)
                .args(&["-s", "ALL"])
                .spawn()?
                .wait()?
                .check()?;
        }

        println!("Syncing portage");
        Executor::new(&sudo, dry_run)
            .arg("/usr/bin/emerge")
            .args(&["-q", "--sync"])
            .spawn()?
            .wait()?
            .check()?;

        if let Some(eix_update) = which("eix-update") {
            Executor::new(&sudo, dry_run).arg(eix_update).spawn()?.wait()?.check()?;
        }

        Executor::new(&sudo, dry_run)
            .arg("/usr/bin/emerge")
            .args(&["-uDNa", "world"])
            .spawn()?
            .wait()?
            .check()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_debian(sudo: &Option<PathBuf>, cleanup: bool, dry_run: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/apt", "update"])
            .spawn()?
            .wait()?
            .check()?;

        Executor::new(&sudo, dry_run)
            .args(&["/usr/bin/apt", "dist-upgrade"])
            .spawn()?
            .wait()?
            .check()?;

        if cleanup {   
            Executor::new(&sudo, dry_run)
                .args(&["/usr/bin/apt", "clean"])
                .spawn()?
                .wait()?
                .check()?;

            Executor::new(&sudo, dry_run)
                .args(&["/usr/bin/apt", "autoremove"])
                .spawn()?
                .wait()?
                .check()?;
        }
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

#[must_use]
pub fn run_needrestart(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(needrestart) = which("needrestart") {
            print_separator("Check for needed restarts");

            let success = || -> Result<(), Error> {
                Executor::new(&sudo, dry_run)
                    .arg(needrestart)
                    .spawn()?
                    .wait()?
                    .check()?;

                Ok(())
            }()
            .is_ok();

            return Some(("Restarts", success));
        }
    }

    None
}

#[must_use]
pub fn run_fwupdmgr(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(fwupdmgr) = which("fwupdmgr") {
        print_separator("Firmware upgrades");

        let success = || -> Result<(), Error> {
            Executor::new(&fwupdmgr, dry_run)
                .arg("refresh")
                .spawn()?
                .wait()?
                .check()?;
            Executor::new(&fwupdmgr, dry_run)
                .arg("get-updates")
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Firmware upgrade", success));
    }

    None
}

#[must_use]
pub fn flatpak_user_update(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(flatpak) = which("flatpak") {
        print_separator("Flatpak User Packages");

        let success = || -> Result<(), Error> {
            Executor::new(&flatpak, dry_run)
                .args(&["update", "--user", "-y"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Flatpak User Packages", success));
    }

    None
}

#[must_use]
pub fn flatpak_global_update(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(flatpak) = which("flatpak") {
            print_separator("Flatpak Global Packages");

            let success = || -> Result<(), Error> {
                Executor::new(&sudo, dry_run)
                    .args(&[flatpak.to_str().unwrap(), "update", "-y"])
                    .spawn()?
                    .wait()?
                    .check()?;
                Ok(())
            }()
            .is_ok();

            return Some(("Flatpak Global Packages", success));
        }
    }

    None
}

#[must_use]
pub fn run_snap(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(snap) = which("snap") {
            if PathBuf::from("/var/snapd.socket").exists() {
                print_separator("snap");

                let success = || -> Result<(), Error> {
                    Executor::new(&sudo, dry_run)
                        .args(&[snap.to_str().unwrap(), "refresh"])
                        .spawn()?
                        .wait()?
                        .check()?;

                    Ok(())
                }()
                .is_ok();

                return Some(("snap", success));
            }
        }
    }

    None
}

#[must_use]
pub fn run_etc_update(sudo: &Option<PathBuf>, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(etc_update) = which("etc-update") {
            print_separator("etc-update");

            let success = || -> Result<(), Error> {
                Executor::new(&sudo, dry_run)
                    .arg(&etc_update.to_str().unwrap())
                    .spawn()?
                    .wait()?
                    .check()?;

                Ok(())
            }()
            .is_ok();

            return Some(("etc-update", success));
        }
    }

    None
}
