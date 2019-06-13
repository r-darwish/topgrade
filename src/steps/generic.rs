use crate::error::{Error, ErrorKind};
use crate::executor::{CommandExt, RunType};
use crate::terminal::print_separator;
use crate::utils::{self, PathExt};
use directories::BaseDirs;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

pub fn run_cargo_update(run_type: RunType) -> Result<(), Error> {
    let cargo_update = utils::require("cargo-install-update")?;

    print_separator("Cargo");

    run_type
        .execute(cargo_update)
        .args(&["install-update", "--git", "--all"])
        .check_run()
}

pub fn run_gem(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let gem = utils::require("gem")?;
    base_dirs.home_dir().join(".gem").require()?;

    print_separator("RubyGems");

    run_type.execute(&gem).args(&["update", "--user-install"]).check_run()
}

#[cfg(not(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
pub fn run_apm(run_type: RunType) -> Result<(), Error> {
    let apm = utils::require("apm")?;

    print_separator("Atom Package Manager");

    run_type.execute(&apm).args(&["upgrade", "--confirm=false"]).check_run()
}

pub fn run_rustup(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let rustup = utils::require("rustup")?;

    print_separator("rustup");

    if rustup
        .canonicalize()
        .context(ErrorKind::StepFailed)?
        .is_descendant_of(base_dirs.home_dir())
    {
        run_type.execute(&rustup).args(&["self", "update"]).check_run()?;
    }

    run_type.execute(&rustup).arg("update").check_run()
}

pub fn run_jetpack(run_type: RunType) -> Result<(), Error> {
    let jetpack = utils::require("jetpack")?;

    print_separator("Jetpack");

    run_type.execute(&jetpack).args(&["global", "update"]).check_run()
}

pub fn run_opam_update(run_type: RunType) -> Result<(), Error> {
    let opam = utils::require("opam")?;

    print_separator("OCaml Package Manager");

    run_type.execute(&opam).arg("update").check_run()?;
    run_type.execute(&opam).arg("upgrade").check_run()
}

pub fn run_vcpkg_update(run_type: RunType) -> Result<(), Error> {
    let vcpkg = utils::require("vcpkg")?;
    print_separator("vcpkg");

    run_type.execute(&vcpkg).args(&["upgrade", "--no-dry-run"]).check_run()
}

pub fn run_pipx_update(run_type: RunType) -> Result<(), Error> {
    let pipx = utils::require("pipx")?;
    print_separator("pipx");

    run_type.execute(&pipx).arg("upgrade-all").check_run()
}

pub fn run_myrepos_update(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let myrepos = utils::require("mr")?;
    base_dirs.home_dir().join(".mrconfig").require()?;

    print_separator("myrepos");

    run_type
        .execute(&myrepos)
        .arg("--directory")
        .arg(base_dirs.home_dir())
        .arg("checkout")
        .check_run()?;
    run_type
        .execute(&myrepos)
        .arg("--directory")
        .arg(base_dirs.home_dir())
        .arg("update")
        .check_run()
}

pub fn run_custom_command(name: &str, command: &str, run_type: RunType) -> Result<(), Error> {
    print_separator(name);
    run_type.execute("sh").arg("-c").arg(command).check_run()
}

pub fn run_composer_update(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let composer = utils::require("composer")?;
    let composer_home = Command::new(&composer)
        .args(&["global", "config", "--absolute", "home"])
        .check_output()
        .map_err(|_| Error::from(ErrorKind::SkipStep))
        .map(PathBuf::from)
        .and_then(PathExt::require)?;

    if !composer_home.is_descendant_of(base_dirs.home_dir()) {
        Err(ErrorKind::SkipStep)?;
    }

    print_separator("Composer");

    run_type.execute(&composer).args(&["global", "update"]).check_run()?;

    if let Some(valet) = utils::which("valet") {
        run_type.execute(&valet).arg("install").check_run()?;
    }

    Ok(())
}

pub fn run_remote_topgrade(run_type: RunType, hostname: &str, run_in_tmux: bool) -> Result<(), Error> {
    let ssh = utils::require("ssh")?;

    if run_in_tmux && !run_type.dry() {
        #[cfg(unix)]
        {
            crate::tmux::run_remote_topgrade(hostname, &ssh)?;
            Err(ErrorKind::SkipStep)?
        }

        unreachable!("Tmux execution is only implemented in Unix");
    } else {
        run_type
            .execute(&ssh)
            .args(&[
                "-t",
                hostname,
                "env",
                &format!("TOPGRADE_PREFIX={}", hostname),
                "topgrade",
            ])
            .check_run()
    }
}
