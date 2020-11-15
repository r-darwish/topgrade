#![allow(unused_imports)]
use crate::error::{SkipStep, TopgradeError};
use crate::execution_context::ExecutionContext;
use crate::executor::{CommandExt, ExecutorOutput, RunType};
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
    run_type
        .execute(&go)
        .args(&["get", "-u", "all"])
        .env_remove("GO111MODULE")
        .check_run()
}

pub fn run_gem(base_dirs: &BaseDirs, run_type: RunType) -> Result<()> {
    let gem = utils::require("gem")?;
    base_dirs.home_dir().join(".gem").require()?;

    print_separator("RubyGems");

    let mut command = run_type.execute(&gem);
    command.arg("update");

    if env::var_os("RBENV_SHELL").is_none() {
        debug!("Detected rbenv. Avoiding --user-install");
        command.arg("--user-install");
    }

    command.check_run()
}

pub fn run_sheldon(ctx: &ExecutionContext) -> Result<()> {
    let sheldon = utils::require("sheldon")?;

    print_separator("Sheldon");

    ctx.run_type().execute(&sheldon).args(&["lock", "--update"]).check_run()
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

pub fn run_choosenim(ctx: &ExecutionContext) -> Result<()> {
    let choosenim = utils::require("choosenim")?;

    print_separator("choosenim");
    let run_type = ctx.run_type();

    run_type.execute(&choosenim).args(&["update", "self"]).check_run()?;
    run_type.execute(&choosenim).args(&["update", "stable"]).check_run()
}

pub fn run_krew_upgrade(run_type: RunType) -> Result<()> {
    let krew = utils::require("kubectl-krew")?;

    print_separator("Krew");

    run_type.execute(&krew).args(&["upgrade"]).check_run()
}

pub fn run_gcloud_components_update(run_type: RunType) -> Result<()> {
    let gcloud = utils::require("gcloud")?;

    print_separator("gcloud");

    run_type
        .execute(&gcloud)
        .args(&["components", "update", "--quiet"])
        .check_run()
}

pub fn run_jetpack(run_type: RunType) -> Result<()> {
    let jetpack = utils::require("jetpack")?;

    print_separator("Jetpack");

    run_type.execute(&jetpack).args(&["global", "update"]).check_run()
}

pub fn run_rtcl(ctx: &ExecutionContext) -> Result<()> {
    let rupdate = utils::require("rupdate")?;

    print_separator("rtcl");

    ctx.run_type().execute(&rupdate).check_run()
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

pub fn run_tlmgr_update(ctx: &ExecutionContext) -> Result<()> {
    cfg_if::cfg_if! {
        if #[cfg(target_os = "linux")] {
            if !ctx.config().enable_tlmgr_linux() {
                return Err(SkipStep(String::from("tlmgr must be explicity enabled in the configuration to run in Linux")).into());
            }
        }
    }

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
        ctx.run_type().execute(&tlmgr)
    } else {
        let mut c = ctx
            .run_type()
            .execute(ctx.sudo().as_ref().ok_or(TopgradeError::SudoRequired)?);
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

pub fn run_composer_update(ctx: &ExecutionContext) -> Result<()> {
    let composer = utils::require("composer")?;
    let composer_home = Command::new(&composer)
        .args(&["global", "config", "--absolute", "--quiet", "home"])
        .check_output()
        .map_err(|e| (SkipStep(format!("Error getting the composer directory: {}", e))))
        .map(|s| PathBuf::from(s.trim()))?
        .require()?;

    if !composer_home.is_descendant_of(ctx.base_dirs().home_dir()) {
        return Err(SkipStep(format!(
            "Composer directory {} isn't a decandent of the user's home directory",
            composer_home.display()
        ))
        .into());
    }

    print_separator("Composer");

    if ctx.config().composer_self_update() {
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                // If self-update fails without sudo then there's probably an update
                let has_update = match ctx.run_type().execute(&composer).arg("self-update").output()? {
                    ExecutorOutput::Wet(output) => !output.status.success(),
                    _ => false
                };

                if has_update {
                    ctx.run_type()
                        .execute(ctx.sudo().as_ref().unwrap())
                        .arg(&composer)
                        .arg("self-update")
                        .check_run()?;
                }
            } else {
                ctx.run_type().execute(&composer).arg("self-update").check_run()?;
            }
        }
    }

    let output = Command::new(&composer).args(&["global", "update"]).output()?;
    let status = output.status;
    if !status.success() {
        return Err(TopgradeError::ProcessFailed(status).into());
    }
    let stdout = String::from_utf8(output.stdout)?;
    let stderr = String::from_utf8(output.stderr)?;
    print!("{}\n{}", stdout, stderr);

    if stdout.contains("valet") || stderr.contains("valet") {
        if let Some(valet) = utils::which("valet") {
            ctx.run_type().execute(&valet).arg("install").check_run()?;
        }
    }

    Ok(())
}
