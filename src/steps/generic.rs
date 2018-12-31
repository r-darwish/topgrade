use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{self, Check, PathExt};
use directories::BaseDirs;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

#[must_use]
pub fn run_cargo_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(cargo_update) = utils::which("cargo-install-update") {
        print_separator("Cargo");

        let success = || -> Result<(), Error> {
            run_type
                .execute(cargo_update)
                .args(&["install-update", "--git", "--all"])
                .check_run()?;

            Ok(())
        }()
        .is_ok();

        return Some(("Cargo", success));
    }

    None
}

#[must_use]
pub fn run_gem(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(gem) = utils::which("gem") {
        if base_dirs.home_dir().join(".gem").exists() {
            print_separator("RubyGems");

            let success = || -> Result<(), Error> {
                run_type.execute(&gem).args(&["update", "--user-install"]).check_run()?;

                Ok(())
            }()
            .is_ok();

            return Some(("RubyGems", success));
        }
    }
    None
}

#[must_use]
pub fn run_emacs(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(emacs) = utils::which("emacs") {
        if let Some(init_file) = base_dirs.home_dir().join(".emacs.d/init.el").if_exists() {
            print_separator("Emacs");

            let success = || -> Result<(), Error> {
                run_type
                    .execute(&emacs)
                    .args(&["--batch", "-l", init_file.to_str().unwrap(), "--eval", EMACS_UPGRADE])
                    .check_run()?;

                Ok(())
            }()
            .is_ok();

            return Some(("Emacs", success));
        }
    }
    None
}

#[must_use]
#[cfg(not(any(
    target_os = "freebsd",
    target_os = "openbsd",
    target_os = "netbsd",
    target_os = "dragonfly"
)))]
pub fn run_apm(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(apm) = utils::which("apm") {
        print_separator("Atom Package Manager");

        let success = || -> Result<(), Error> {
            run_type
                .execute(&apm)
                .args(&["upgrade", "--confirm=false"])
                .check_run()?;

            Ok(())
        }()
        .is_ok();

        return Some(("apm", success));
    }

    None
}

#[must_use]
pub fn run_rustup(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(rustup) = utils::which("rustup") {
        print_separator("rustup");

        let success = || -> Result<(), Error> {
            if rustup.is_descendant_of(base_dirs.home_dir()) {
                run_type.execute(&rustup).args(&["self", "update"]).check_run()?;
            }

            run_type.execute(&rustup).arg("update").check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("rustup", success));
    }

    None
}

#[must_use]
pub fn run_jetpack(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(jetpack) = utils::which("jetpack") {
        print_separator("Jetpack");

        let success = || -> Result<(), Error> {
            run_type.execute(&jetpack).args(&["global", "update"]).check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Jetpack", success));
    }

    None
}

#[must_use]
pub fn run_opam_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(opam) = utils::which("opam") {
        print_separator("OCaml Package Manager");

        let success = || -> Result<(), Error> {
            run_type.execute(&opam).arg("update").check_run()?;
            run_type.execute(&opam).arg("upgrade").check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("OPAM", success));
    }

    None
}

#[must_use]
pub fn run_vcpkg_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(vcpkg) = utils::which("vcpkg") {
        print_separator("vcpkg");

        let success = || -> Result<(), Error> {
            run_type
                .execute(&vcpkg)
                .args(&["upgrade", "--no-dry-run"])
                .check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("vcpkg", success));
    }

    None
}

#[must_use]
pub fn run_pipx_update(run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(pipx) = utils::which("pipx") {
        print_separator("pipx");

        let success = || -> Result<(), Error> {
            run_type.execute(&pipx).arg("upgrade-all").check_run()?;
            Ok(())
        }()
        .is_ok();

        return Some(("pipx", success));
    }

    None
}

#[must_use]
pub fn run_custom_command(name: &str, command: &str, run_type: RunType) -> Result<(), Error> {
    print_separator(name);
    run_type.execute("sh").arg("-c").arg(command).check_run()?;

    Ok(())
}

#[must_use]
pub fn run_composer_update(base_dirs: &BaseDirs, run_type: RunType) -> Option<(&'static str, bool)> {
    if let Some(composer) = utils::which("composer") {
        let composer_home = || -> Result<PathBuf, Error> {
            let output = Command::new(&composer)
                .args(&["global", "config", "--absolute", "home"])
                .output()
                .context(ErrorKind::ProcessExecution)?;
            output.status.check()?;
            Ok(PathBuf::from(
                &String::from_utf8(output.stdout).context(ErrorKind::ProcessExecution)?,
            ))
        }();

        if let Ok(composer_home) = composer_home {
            if composer_home.is_descendant_of(base_dirs.home_dir()) {
                print_separator("Composer");

                let success = || -> Result<(), Error> {
                    run_type.execute(&composer).args(&["global", "update"]).check_run()?;

                    if let Some(valet) = utils::which("valet") {
                        run_type.execute(&valet).arg("install").check_run()?;
                    }

                    Ok(())
                }()
                .is_ok();

                return Some(("Composer", success));
            }
        }
    }

    None
}
