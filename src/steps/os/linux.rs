use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::{print_separator, print_warning};
use crate::utils::{require, require_option, which};
use failure::ResultExt;
use ini::Ini;
use log::debug;
use serde::Deserialize;
use std::env::var_os;
use std::ffi::OsString;
use std::path::PathBuf;
use walkdir::WalkDir;

static OS_RELEASE_PATH: &str = "/etc/os-release";

#[derive(Debug, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
struct OsRelease {
    id_like: Option<String>,
    id: String,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Distribution {
    Arch,
    CentOS,
    Fedora,
    Debian,
    Gentoo,
    Suse,
    Void,
    Solus,
    Exherbo,
}

impl Distribution {
    fn parse_os_release(os_release: &ini::Ini) -> Result<Self, Error> {
        let section = os_release.general_section();
        let id = section.get("ID").map(String::as_str);
        let id_like: Option<Vec<&str>> = section
            .get("ID_LIKE")
            .map(|s| String::as_str(s).split_whitespace().collect());

        if let Some(id_like) = id_like {
            if id_like.contains(&"debian") || id_like.contains(&"ubuntu") {
                return Ok(Distribution::Debian);
            } else if id_like.contains(&"suse") {
                return Ok(Distribution::Suse);
            } else if id_like.contains(&"arch") || id_like.contains(&"archlinux") {
                return Ok(Distribution::Arch);
            }
        }

        Ok(match id {
            Some("centos") | Some("ol") => Distribution::CentOS,
            Some("fedora") => Distribution::Fedora,
            Some("void") => Distribution::Void,
            Some("debian") => Distribution::Debian,
            Some("arch") | Some("ana") => Distribution::Arch,
            Some("solus") => Distribution::Solus,
            Some("gentoo") => Distribution::Gentoo,
            Some("exherbo") => Distribution::Exherbo,
            _ => return Err(ErrorKind::UnknownLinuxDistribution.into()),
        })
    }

    pub fn detect() -> Result<Self, Error> {
        if PathBuf::from(OS_RELEASE_PATH).exists() {
            let os_release = Ini::load_from_file(OS_RELEASE_PATH).context(ErrorKind::UnknownLinuxDistribution)?;

            return Self::parse_os_release(&os_release);
        }

        Err(ErrorKind::UnknownLinuxDistribution.into())
    }

    #[must_use]
    pub fn upgrade(self, sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType, yes: bool) -> Result<(), Error> {
        print_separator("System update");

        match self {
            Distribution::Arch => upgrade_arch_linux(&sudo, cleanup, run_type, yes),
            Distribution::CentOS => upgrade_redhat(&sudo, run_type, yes, false),
            Distribution::Fedora => upgrade_redhat(&sudo, run_type, yes, true),
            Distribution::Debian => upgrade_debian(&sudo, cleanup, run_type, yes),
            Distribution::Gentoo => upgrade_gentoo(&sudo, run_type),
            Distribution::Suse => upgrade_suse(&sudo, run_type),
            Distribution::Void => upgrade_void(&sudo, run_type),
            Distribution::Solus => upgrade_solus(&sudo, run_type),
            Distribution::Exherbo => upgrade_exherbo(&sudo, cleanup, run_type),
        }
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
        .filter_map(Result::ok)
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

fn upgrade_arch_linux(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType, yes: bool) -> Result<(), Error> {
    let pacman = which("powerpill").unwrap_or_else(|| PathBuf::from("/usr/bin/pacman"));

    let path = {
        let mut path = OsString::from("/usr/bin:");
        path.push(var_os("PATH").unwrap());
        path
    };
    debug!("Running Arch update with path: {:?}", path);

    if let Some(yay) = which("yay") {
        let mut command = run_type.execute(yay);

        command
            .arg("--pacman")
            .arg(pacman)
            .arg("-Syu")
            .arg("--devel")
            .env("PATH", path);

        if yes {
            command.arg("--noconfirm");
        }
        command.check_run()?;
    } else if let Some(sudo) = &sudo {
        let mut command = run_type.execute(&sudo);
        command.arg(pacman).arg("-Syu").env("PATH", path);
        if yes {
            command.arg("--noconfirm");
        }
        command.check_run()?;
    } else {
        print_warning("Neither sudo nor yay detected. Skipping system upgrade");
    }

    if cleanup {
        if let Some(sudo) = &sudo {
            run_type.execute(&sudo).args(&["/usr/bin/pacman", "-Scc"]).check_run()?;
        }
    }

    Ok(())
}

fn upgrade_redhat(sudo: &Option<PathBuf>, run_type: RunType, yes: bool, dnf: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        let mut command = run_type.execute(&sudo);
        command.args(&[if dnf { "/usr/bin/dnf" } else { "/usr/bin/yum" }, "upgrade"]);
        if yes {
            command.arg("-y");
        }

        command.check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_suse(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
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
        for _ in 0..2 {
            run_type
                .execute(&sudo)
                .args(&["/usr/bin/xbps-install", "-Su"])
                .check_run()?;
        }
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

fn upgrade_debian(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType, yes: bool) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        let apt = which("apt-fast").unwrap_or_else(|| PathBuf::from("/usr/bin/apt"));
        run_type.execute(&sudo).arg(&apt).arg("update").check_run()?;
        let mut command = run_type.execute(&sudo);
        command.arg(&apt).arg("dist-upgrade").check_run()?;

        if yes {
            command.arg("-y");
        }

        if cleanup {
            run_type.execute(&sudo).arg(&apt).arg("clean").check_run()?;

            run_type.execute(&sudo).arg(&apt).arg("autoremove").check_run()?;
        }
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_solus(sudo: &Option<PathBuf>, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/eopkg", "upgrade"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_exherbo(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Result<(), Error> {
    if let Some(sudo) = &sudo {
        run_type.execute(&sudo).args(&["/usr/bin/cave", "sync"]).check_run()?;

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/cave", "resolve", "world", "-c1", "-Cs", "-km", "-Km", "-x"])
            .check_run()?;

        if cleanup {
            run_type
                .execute(&sudo)
                .args(&["/usr/bin/cave", "purge", "-x"])
                .check_run()?;
        }

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/cave", "fix-linkage", "-x", "--", "-Cs"])
            .check_run()?;

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/eclectic", "config", "interactive"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

pub fn run_needrestart(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    let needrestart = require("needrestart")?;

    print_separator("Check for needed restarts");

    run_type.execute(&sudo).arg(needrestart).check_run()?;

    Ok(())
}

#[must_use]
pub fn run_fwupdmgr(run_type: RunType) -> Result<(), Error> {
    let fwupdmgr = require("fwupdmgr")?;

    print_separator("Firmware upgrades");

    run_type.execute(&fwupdmgr).arg("refresh").check_run()?;
    run_type.execute(&fwupdmgr).arg("get-updates").check_run()
}

#[must_use]
pub fn flatpak_update(run_type: RunType) -> Result<(), Error> {
    let flatpak = require("flatpak")?;
    print_separator("Flatpak User Packages");

    run_type
        .execute(&flatpak)
        .args(&["update", "--user", "-y"])
        .check_run()?;
    run_type
        .execute(&flatpak)
        .args(&["update", "--system", "-y"])
        .check_run()
}

#[must_use]
pub fn run_snap(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    let snap = require("snap")?;

    if !PathBuf::from("/var/snapd.socket").exists() && !PathBuf::from("/run/snapd.socket").exists() {
        return Err(ErrorKind::SkipStep.into());
    }
    print_separator("snap");

    run_type.execute(sudo).arg(snap).arg("refresh").check_run()
}

#[must_use]
pub fn run_rpi_update(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    let rpi_update = require("rpi-update")?;

    print_separator("rpi-update");

    run_type.execute(sudo).arg(rpi_update).check_run()
}

#[must_use]
pub fn run_pihole_update(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    let pihole = require("pihole")?;

    print_separator("pihole");

    run_type.execute(sudo).arg(pihole).arg("-up").check_run()
}

#[must_use]
pub fn run_etc_update(sudo: Option<&PathBuf>, run_type: RunType) -> Result<(), Error> {
    let sudo = require_option(sudo)?;
    let etc_update = require("etc-update")?;
    print_separator("etc-update");

    run_type.execute(sudo).arg(etc_update).check_run()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_template(os_release_file: &str, expected_distribution: Distribution) {
        let os_release = Ini::load_from_str(os_release_file).unwrap();
        assert_eq!(
            Distribution::parse_os_release(&os_release).unwrap(),
            expected_distribution
        );
    }
    #[test]
    fn test_arch_linux() {
        test_template(&include_str!("os_release/arch"), Distribution::Arch);
        test_template(&include_str!("os_release/arch32"), Distribution::Arch);
    }

    #[test]
    fn test_centos() {
        test_template(&include_str!("os_release/centos"), Distribution::CentOS);
    }

    #[test]
    fn test_debian() {
        test_template(&include_str!("os_release/debian"), Distribution::Debian);
    }

    #[test]
    fn test_ubuntu() {
        test_template(&include_str!("os_release/ubuntu"), Distribution::Debian);
    }

    #[test]
    fn test_mint() {
        test_template(&include_str!("os_release/mint"), Distribution::Debian);
    }

    #[test]
    fn test_opensuse() {
        test_template(&include_str!("os_release/opensuse"), Distribution::Suse);
    }

    #[test]
    fn test_oraclelinux() {
        test_template(&include_str!("os_release/oracle"), Distribution::CentOS);
    }

    #[test]
    fn test_fedora() {
        test_template(&include_str!("os_release/fedora"), Distribution::Fedora);
    }

    #[test]
    fn test_antergos() {
        test_template(&include_str!("os_release/antergos"), Distribution::Arch);
    }

    #[test]
    fn test_manjaro() {
        test_template(&include_str!("os_release/manjaro"), Distribution::Arch);
    }

    #[test]
    fn test_anarchy() {
        test_template(&include_str!("os_release/anarchy"), Distribution::Arch);
    }

    #[test]
    fn test_gentoo() {
        test_template(&include_str!("os_release/gentoo"), Distribution::Gentoo);
    }

    #[test]
    fn test_exherbo() {
        test_template(&include_str!("os_release/exherbo"), Distribution::Exherbo);
    }
}
