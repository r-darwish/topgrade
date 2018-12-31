use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::{print_separator, print_warning};
use crate::utils::which;
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
    pub fn upgrade(self, sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Option<(&'static str, bool)> {
        print_separator("System update");

        let success = match self {
            Distribution::Arch => upgrade_arch_linux(&sudo, cleanup, run_type),
            Distribution::CentOS => upgrade_redhat(&sudo, run_type),
            Distribution::Fedora => upgrade_fedora(&sudo, run_type),
            Distribution::Ubuntu | Distribution::Debian => upgrade_debian(&sudo, cleanup, run_type),
            Distribution::Gentoo => upgrade_gentoo(&sudo, run_type),
            Distribution::OpenSuse => upgrade_opensuse(&sudo, run_type),
            Distribution::Void => upgrade_void(&sudo, run_type),
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

fn upgrade_arch_linux(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Result<(), Error> {
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
        run_type.execute(yay).check_run()?;
    } else if let Some(sudo) = &sudo {
        run_type.execute(&sudo).args(&["/usr/bin/pacman", "-Syu"]).check_run()?;
    } else {
        print_warning("No sudo or yay detected. Skipping system upgrade");
    }

    if cleanup {
        if let Some(sudo) = &sudo {
            run_type.execute(&sudo).args(&["/usr/bin/pacman", "-Scc"]).check_run()?;
        }
    }

    Ok(())
}

fn upgrade_redhat(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type.execute(&sudo).args(&["/usr/bin/yum", "upgrade"]).check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_opensuse(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/zypper", "refresh"])
            .check_run()?;

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/zypper", "dist-upgrade"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_void(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/xbps-install", "-Su"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_fedora(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type.execute(&sudo).args(&["/usr/bin/dnf", "upgrade"]).check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_gentoo(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        if let Some(layman) = which("layman") {
            run_type.execute(&sudo).arg(layman).args(&["-s", "ALL"]).check_run()?;
        }

        println!("Syncing portage");
        run_type
            .execute(&sudo)
            .arg("/usr/bin/emerge")
            .args(&["-q", "--sync"])
            .check_run()?;

        if let Some(eix_update) = which("eix-update") {
            run_type.execute(&sudo).arg(eix_update).check_run()?;
        }

        run_type
            .execute(&sudo)
            .arg("/usr/bin/emerge")
            .args(&["-uDNa", "world"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_debian(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type.execute(&sudo).args(&["/usr/bin/apt", "update"]).check_run()?;

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/apt", "dist-upgrade"])
            .check_run()?;

        if cleanup {
            run_type.execute(&sudo).args(&["/usr/bin/apt", "clean"]).check_run()?;

            run_type
                .execute(&sudo)
                .args(&["/usr/bin/apt", "autoremove"])
                .check_run()?;
        }
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

#[must_use]
pub fn run_needrestart(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(needrestart) = which("needrestart") {
            print_separator("Check for needed restarts");

            let success = || -> Result<(), Error> {
                run_type.execute(&sudo).arg(needrestart).check_run()?;

                Ok(())
            }()
            .is_ok();

            return Some(("Restarts", success));
        }
    }

    None
}

#[must_use]
pub fn run_fwupdmgr(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(fwupdmgr) = which("fwupdmgr") {
        print_separator("Firmware upgrades");

        let success = || -> Result<(), Error> {
            run_type.execute(&fwupdmgr).arg("refresh").check_run()?;
            run_type.execute(&fwupdmgr).arg("get-updates").check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Firmware upgrade", success));
    }

    None
}

#[must_use]
pub fn flatpak_user_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(flatpak) = which("flatpak") {
        print_separator("Flatpak User Packages");

        let success = || -> Result<(), Error> {
            run_type
                .execute(&flatpak)
                .args(&["update", "--user", "-y"])
                .check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Flatpak User Packages", success));
    }

    None
}

#[must_use]
pub fn flatpak_global_update(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(flatpak) = which("flatpak") {
            print_separator("Flatpak Global Packages");

            let success = || -> Result<(), Error> {
                run_type
                    .execute(&sudo)
                    .args(&[flatpak.to_str().unwrap(), "update", "-y"])
                    .check_run()?;
                Ok(())
            }()
            .is_ok();

            return Some(("Flatpak Global Packages", success));
        }
    }

    None
}

#[must_use]
pub fn run_snap(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(snap) = which("snap") {
            if PathBuf::from("/var/snapd.socket").exists() {
                print_separator("snap");

                let success = || -> Result<(), Error> {
                    run_type
                        .execute(&sudo)
                        .args(&[snap.to_str().unwrap(), "refresh"])
                        .check_run()?;

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
pub fn run_etc_update(sudo: &Option<PathBuf>, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(sudo) = sudo {
        if let Some(etc_update) = which("etc-update") {
            print_separator("etc-update");

            let success = || -> Result<(), Error> {
                run_type.execute(&sudo).arg(&etc_update.to_str().unwrap()).check_run()?;

                Ok(())
            }()
            .is_ok();

            return Some(("etc-update", success));
        }
    }

    None
}
