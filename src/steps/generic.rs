use crate::error::{SkipStep, TopgradeError};
use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, RunType};
use crate::terminal::{print_separator, shell};
use crate::utils::{self, PathExt};
use anyhow::Result;
use directories::BaseDirs;
use log::debug;
use std::env;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempfile_in;

pub fn run_cargo_update(run_type: RunType) -> Result<()> {
    let cargo_update = utils::require("cargo-install-update")?;

    print_separator("Cargo");

    run_type
        .execute(cargo_update)
        .args(&["install-update", "--git", "--all"])
        .check_run()
}

pub fn run_flutter_upgrade(run_type: RunType) -> Result<()> {
    let flutter = utils::require("flutter")?;

    print_separator("Flutter");
    run_type.execute(&flutter).arg("upgrade").check_run()
}

pub fn run_go(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let go = utils::require("go")?;
    env::var("GOPATH")
        .unwrap_or_else(|_| base_dirs.home_dir().join("go").to_str().unwrap().to_string())
        .require()?;

    print_separator("Go");
    run_type.execute(&go).arg("get").arg("-u").arg("all").check_run()
}

pub fn run_gem(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
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
pub fn run_apm(run_type: RunType) -> Result<()> {
    let apm = utils::require("apm")?;

    print_separator("Atom Package Manager");

    run_type.execute(&apm).args(&["upgrade", "--confirm=false"]).check_run()
}

pub fn run_rustup(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let rustup = utils::require("rustup")?;

    print_separator("rustup");

    if rustup.canonicalize()?.is_descendant_of(base_dirs.home_dir()) {
        run_type.execute(&rustup).args(&["self", "update"]).check_run()?;
    }

    run_type.execute(&rustup).arg("update").check_run()
}

pub fn run_jetpack(run_type: RunType) -> Result<()> {
    let jetpack = utils::require("jetpack")?;

    print_separator("Jetpack");

    run_type.execute(&jetpack).args(&["global", "update"]).check_run()
}

pub fn run_opam_update(run_type: RunType) -> Result<()> {
    let opam = utils::require("opam")?;

    print_separator("OCaml Package Manager");

    run_type.execute(&opam).arg("update").check_run()?;
    run_type.execute(&opam).arg("upgrade").check_run()
}

pub fn run_vcpkg_update(run_type: RunType) -> Result<()> {
    let vcpkg = utils::require("vcpkg")?;
    print_separator("vcpkg");

    run_type.execute(&vcpkg).args(&["upgrade", "--no-dry-run"]).check_run()
}

pub fn run_pipx_update(run_type: RunType) -> Result<()> {
    let pipx = utils::require("pipx")?;
    print_separator("pipx");

    run_type.execute(&pipx).arg("upgrade-all").check_run()
}

pub fn run_stack_update(run_type: RunType) -> Result<()> {
    let stack = utils::require("stack")?;
    print_separator("stack");

    run_type.execute(&stack).arg("upgrade").check_run()
}

pub fn run_tlmgr_update(sudo: &Option<PathBuf>, run_type: RunType) -> Result<()> {
    let tlmgr = utils::require("tlmgr")?;
    let kpsewhich = utils::require("kpsewhich")?;
    let tlmgr_directory = {
        let mut d = PathBuf::from(
            std::str::from_utf8(
                &Command::new(&kpsewhich)
                    .arg("-var-value=SELFAUTOPARENT")
                    .output()?
                    .stdout,
            )?
            .trim(),
        );
        d.push("tlpkg");
        d
    }
    .require()?;

    let directory_writable = tempfile_in(&tlmgr_directory).is_ok();
    debug!("{:?} writable: {}", tlmgr_directory, directory_writable);

    print_separator("TeX Live package manager");

    let mut command = if directory_writable {
        run_type.execute(&tlmgr)
    } else {
        let mut c = run_type.execute(sudo.as_ref().ok_or(TopgradeError::SudoRequired)?);
        c.arg(&tlmgr);
        c
    };
    command.args(&["update", "--self", "--all"]);

    command.check_run()
}

pub fn run_myrepos_update(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
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

pub fn run_custom_command(name: &str, command: &str, ctx: &ExecutionContext) -> Result<()> {
    print_separator(name);
    ctx.run_type().execute(shell()).arg("-c").arg(command).check_run()
}

pub fn run_composer_update(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let composer = utils::require("composer")?;
    let composer_home = Command::new(&composer)
        .args(&["global", "config", "--absolute", "--quiet", "home"])
        .check_output()
        .map_err(|_| (SkipStep))
        .map(|s| PathBuf::from(s.trim()))?
        .require()?;

    if !composer_home.is_descendant_of(base_dirs.home_dir()) {
        return Err(SkipStep.into());
    }

    print_separator("Composer");

    let output = Command::new(&composer).args(&["global", "update"]).check_output()?;
    if output.contains("valet") {
        if let Some(valet) = utils::which("valet") {
            run_type.execute(&valet).arg("install").check_run()?;
        }
    }

    Ok(())
}

pub fn run_remote_topgrade(
    ctx: &ExecutionContext,
    hostname: &str,
) -> Result<()> {
    let ssh = utils::require("ssh")?;

    if ctx.config().run_in_tmux() && !ctx.run_type().dry() {
        #[cfg(unix)]
        {
            crate::tmux::run_remote_topgrade(hostname, &ssh, ctx.config().tmux_arguments())?;
            Err(SkipStep.into())
        }

        #[cfg(not(unix))]
        unreachable!("Tmux execution is only implemented in Unix");
    } else {
        let mut args = vec!["-t", hostname];

        if let Some(ssh_arguments) = ctx.config().ssh_arguments() {
            args.extend(ssh_arguments.split_whitespace());
        }

        let env = format!("TOPGRADE_PREFIX={}", hostname);
        args.extend(&["env", &env, "topgrade"]);

        ctx.run_type().execute(&ssh).args(&args).check_run()
    }
}
