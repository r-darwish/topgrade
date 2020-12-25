use crate::error::{SkipStep, TopgradeError};
use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, ExecutorExitStatus, RunType};
use crate::terminal::{print_separator, print_warning};
use crate::utils::{require, require_option, which, PathExt};
use anyhow::Result;
use ini::Ini;
use log::debug;
use serde::Deserialize;
use std::env::var_os;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
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
    ClearLinux,
    Fedora,
    Debian,
    Gentoo,
    Suse,
    Void,
    Solus,
    Exherbo,
    NixOS,
}

impl Distribution {
    fn parse_os_release(os_release: &ini::Ini) -> Result<Self> {
        let section = os_release.general_section();
        let id = section.get("ID");
        let id_like: Option<Vec<&str>> = section.get("ID_LIKE").map(|s| s.split_whitespace().collect());

        Ok(match id {
            Some("centos") | Some("rhel") | Some("ol") => Distribution::CentOS,
            Some("clear-linux-os") => Distribution::ClearLinux,
            Some("fedora") => Distribution::Fedora,
            Some("void") => Distribution::Void,
            Some("debian") => Distribution::Debian,
            Some("arch") | Some("anarchy") | Some("manjaro-arm") | Some("garuda") | Some("artix") => Distribution::Arch,
            Some("solus") => Distribution::Solus,
            Some("gentoo") => Distribution::Gentoo,
            Some("exherbo") => Distribution::Exherbo,
            Some("nixos") => Distribution::NixOS,
            _ => {
                if let Some(id_like) = id_like {
                    if id_like.contains(&"debian") || id_like.contains(&"ubuntu") {
                        return Ok(Distribution::Debian);
                    } else if id_like.contains(&"centos") {
                        return Ok(Distribution::CentOS);
                    } else if id_like.contains(&"suse") {
                        return Ok(Distribution::Suse);
                    } else if id_like.contains(&"arch") || id_like.contains(&"archlinux") {
                        return Ok(Distribution::Arch);
                    } else if id_like.contains(&"fedora") {
                        return Ok(Distribution::Fedora);
                    }
                }
                return Err(TopgradeError::UnknownLinuxDistribution.into());
            }
        })
    }

    pub fn detect() -> Result<Self> {
        if PathBuf::from(OS_RELEASE_PATH).exists() {
            let os_release = Ini::load_from_file(OS_RELEASE_PATH)?;

            return Self::parse_os_release(&os_release);
        }

        Err(TopgradeError::UnknownLinuxDistribution.into())
    }

    pub fn upgrade(self, ctx: &ExecutionContext) -> Result<()> {
        print_separator("System update");
        let sudo = ctx.sudo();
        let run_type = ctx.run_type();
        let yes = ctx.config().yes();
        let cleanup = ctx.config().cleanup();

        match self {
            Distribution::Arch => upgrade_arch_linux(ctx),
            Distribution::CentOS | Distribution::Fedora => upgrade_redhat(ctx),
            Distribution::ClearLinux => upgrade_clearlinux(&sudo, run_type),
            Distribution::Debian => upgrade_debian(&sudo, cleanup, run_type, yes),
            Distribution::Gentoo => upgrade_gentoo(ctx),
            Distribution::Suse => upgrade_suse(&sudo, run_type),
            Distribution::Void => upgrade_void(&sudo, run_type),
            Distribution::Solus => upgrade_solus(&sudo, run_type),
            Distribution::Exherbo => upgrade_exherbo(&sudo, cleanup, run_type),
            Distribution::NixOS => upgrade_nixos(&sudo, cleanup, run_type),
        }
    }

    pub fn show_summary(self) {
        if let Distribution::Arch = self {
            show_pacnew();
        }
    }

    pub fn redhat_based(self) -> bool {
        matches!(self, Distribution::CentOS | Distribution::Fedora)
    }
}

fn is_wsl() -> Result<bool> {
    let output = Command::new("uname").arg("-r").check_output()?;
    debug!("Uname output: {}", output);
    Ok(output.contains("microsoft"))
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

fn upgrade_arch_linux(ctx: &ExecutionContext) -> Result<()> {
    let pacman = which("powerpill").unwrap_or_else(|| PathBuf::from("/usr/bin/pacman"));
    let yes = ctx.config().yes();
    let sudo = ctx.sudo();
    let run_type = ctx.run_type();
    let cleanup = ctx.config().cleanup();

    let path = {
        let mut path = OsString::from("/usr/bin:");
        path.push(var_os("PATH").unwrap());
        path
    };
    debug!("Running Arch update with path: {:?}", path);

    if let Some(yay) = which("yay").or_else(|| which("paru")) {
        run_type
            .execute(&yay)
            .arg("-Pw")
            .spawn()
            .and_then(|mut p| p.wait())
            .ok();

        let mut command = run_type.execute(&yay);

        command
            .arg("--pacman")
            .arg(&pacman)
            .arg("-Syu")
            .args(ctx.config().yay_arguments().split_whitespace())
            .env("PATH", path);

        if yes {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if cleanup {
            let mut command = run_type.execute(&yay);
            command.arg("--pacman").arg(&pacman).arg("-Scc");
            if yes {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }
    } else if let Some(trizen) = which("trizen") {
        let mut command = run_type.execute(&trizen);

        command
            .arg("-Syu")
            .args(ctx.config().trizen_arguments().split_whitespace())
            .env("PATH", path);

        if yes {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if cleanup {
            let mut command = run_type.execute(&trizen);
            command.arg("-Sc");
            if yes {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }
    } else if let Some(sudo) = &sudo {
        let mut command = run_type.execute(&sudo);
        command.arg(&pacman).arg("-Syu").env("PATH", path);
        if yes {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if cleanup {
            let mut command = run_type.execute(&sudo);
            command.arg(&pacman).arg("-Scc");
            if yes {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }
    } else {
        print_warning("Neither sudo nor yay detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_redhat(ctx: &ExecutionContext) -> Result<()> {
    if let Some(ostree) = Path::new("/usr/bin/rpm-ostree").if_exists() {
        let mut command = ctx.run_type().execute(ostree);
        command.arg("upgrade");
        if ctx.config().yes() {
            command.arg("-y");
        }

        return command.check_run();
    }

    if let Some(sudo) = &ctx.sudo() {
        let mut command = ctx.run_type().execute(&sudo);
        command
            .arg(
                Path::new("/usr/bin/dnf-3")
                    .if_exists()
                    .unwrap_or_else(|| Path::new("/usr/bin/yum")),
            )
            .arg("upgrade");

        if let Some(args) = ctx.config().dnf_arguments() {
            command.args(args.split_whitespace());
        }

        if ctx.config().yes() {
            command.arg("-y");
        }

        command.check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_suse(sudo: &Option<PathBuf>, run_type: RunType) -> Result<()> {
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

fn upgrade_void(sudo: &Option<PathBuf>, run_type: RunType) -> Result<()> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/xbps-install", "-Su", "xbps"])
            .check_run()?;

        run_type
            .execute(&sudo)
            .args(&["/usr/bin/xbps-install", "-u"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_gentoo(ctx: &ExecutionContext) -> Result<()> {
    let run_type = ctx.run_type();

    if let Some(sudo) = &ctx.sudo() {
        if let Some(layman) = which("layman") {
            run_type.execute(&sudo).arg(layman).args(&["-s", "ALL"]).check_run()?;
        }

        println!("Syncing portage");
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/emerge", "--sync"])
            .args(
                ctx.config()
                    .emerge_sync_flags()
                    .map(|s| s.split_whitespace().collect())
                    .unwrap_or_else(|| vec!["-q"]),
            )
            .check_run()?;

        if let Some(eix_update) = which("eix-update") {
            run_type.execute(&sudo).arg(eix_update).check_run()?;
        }

        run_type
            .execute(&sudo)
            .arg("/usr/bin/emerge")
            .args(
                ctx.config()
                    .emerge_update_flags()
                    .map(|s| s.split_whitespace().collect())
                    .unwrap_or_else(|| vec!["-uDNa", "--with-bdeps=y", "world"]),
            )
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_debian(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType, yes: bool) -> Result<()> {
    if let Some(sudo) = &sudo {
        let apt = which("apt-fast").unwrap_or_else(|| PathBuf::from("/usr/bin/apt"));
        run_type.execute(&sudo).arg(&apt).arg("update").check_run()?;

        let mut command = run_type.execute(&sudo);
        command.arg(&apt).arg("dist-upgrade");
        if yes {
            command.arg("-y");
        }
        command.check_run()?;

        if cleanup {
            run_type.execute(&sudo).arg(&apt).arg("clean").check_run()?;

            let mut command = run_type.execute(&sudo);
            command.arg(&apt).arg("autoremove");
            if yes {
                command.arg("-y");
            }
            command.check_run()?;
        }
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_solus(sudo: &Option<PathBuf>, run_type: RunType) -> Result<()> {
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

fn upgrade_clearlinux(sudo: &Option<PathBuf>, run_type: RunType) -> Result<()> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/usr/bin/swupd", "update"])
            .check_run()?;
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

fn upgrade_exherbo(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Result<()> {
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

fn upgrade_nixos(sudo: &Option<PathBuf>, cleanup: bool, run_type: RunType) -> Result<()> {
    if let Some(sudo) = &sudo {
        run_type
            .execute(&sudo)
            .args(&["/run/current-system/sw/bin/nixos-rebuild", "switch", "--upgrade"])
            .check_run()?;

        if cleanup {
            run_type
                .execute(&sudo)
                .args(&["/run/current-system/sw/bin/nix-collect-garbage", "-d"])
                .check_run()?;
        }
    } else {
        print_warning("No sudo detected. Skipping system upgrade");
    }

    Ok(())
}

pub fn run_needrestart(sudo: Option<&PathBuf>, run_type: RunType) -> Result<()> {
    let sudo = require_option(sudo, String::from("sudo is not installed"))?;
    let needrestart = require("needrestart")?;
    let distribution = Distribution::detect()?;

    if distribution.redhat_based() {
        return Err(SkipStep(String::from("needrestart will be ran by the package manager")).into());
    }

    print_separator("Check for needed restarts");

    run_type.execute(&sudo).arg(needrestart).check_run()?;

    Ok(())
}

pub fn run_fwupdmgr(run_type: RunType) -> Result<()> {
    let fwupdmgr = require("fwupdmgr")?;

    if is_wsl()? {
        return Err(SkipStep(String::from("Should not run in WSL")).into());
    }

    print_separator("Firmware upgrades");

    for argument in vec!["refresh", "get-updates"].into_iter() {
        let exit_status = run_type.execute(&fwupdmgr).arg(argument).spawn()?.wait()?;

        if let ExecutorExitStatus::Wet(e) = exit_status {
            if !(e.success() || e.code().map(|c| c == 2).unwrap_or(false)) {
                return Err(TopgradeError::ProcessFailed(e).into());
            }
        }
    }

    Ok(())
}

pub fn flatpak_update(run_type: RunType) -> Result<()> {
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

pub fn run_snap(sudo: Option<&PathBuf>, run_type: RunType) -> Result<()> {
    let sudo = require_option(sudo, String::from("sudo is not installed"))?;
    let snap = require("snap")?;

    if !PathBuf::from("/var/snapd.socket").exists() && !PathBuf::from("/run/snapd.socket").exists() {
        return Err(SkipStep(String::from("Snapd socket does not exist")).into());
    }
    print_separator("snap");

    run_type.execute(sudo).arg(snap).arg("refresh").check_run()
}

pub fn run_pihole_update(sudo: Option<&PathBuf>, run_type: RunType) -> Result<()> {
    let sudo = require_option(sudo, String::from("sudo is not installed"))?;
    let pihole = require("pihole")?;
    Path::new("/opt/pihole/update.sh").require()?;

    print_separator("pihole");

    run_type.execute(sudo).arg(pihole).arg("-up").check_run()
}

pub fn run_etc_update(sudo: Option<&PathBuf>, run_type: RunType) -> Result<()> {
    let sudo = require_option(sudo, String::from("sudo is not installed"))?;
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
    fn test_rhel() {
        test_template(&include_str!("os_release/rhel"), Distribution::CentOS);
    }

    #[test]
    fn test_clearlinux() {
        test_template(&include_str!("os_release/clearlinux"), Distribution::ClearLinux);
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
    fn test_manjaro_arm() {
        test_template(&include_str!("os_release/manjaro-arm"), Distribution::Arch);
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

    #[test]
    fn test_amazon_linux() {
        test_template(&include_str!("os_release/amazon_linux"), Distribution::CentOS);
    }

    #[test]
    fn test_nixos() {
        test_template(&include_str!("os_release/nixos"), Distribution::NixOS);
    }

    #[test]
    fn test_fedoraremixonwsl() {
        test_template(&include_str!("os_release/fedoraremixforwsl"), Distribution::Fedora);
    }

    #[test]
    fn test_pengwinonwsl() {
        test_template(&include_str!("os_release/pengwinonwsl"), Distribution::Debian);
    }

    #[test]
    fn test_artix() {
        test_template(&include_str!("os_release/artix"), Distribution::Arch);
    }

    #[test]
    fn test_garuda() {
        test_template(&include_str!("os_release/garuda"), Distribution::Arch);
    }
}
