use super::error::{Error, ErrorKind};
use super::executor::Executor;
use super::terminal::print_separator;
use super::utils::{self, Check, PathExt};
use directories::BaseDirs;
use failure::ResultExt;
use std::path::PathBuf;
use std::process::Command;

const EMACS_UPGRADE: &str = include_str!("emacs.el");

#[must_use]
pub fn run_cargo_update(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(cargo_update) = utils::which("cargo-install-update") {
        print_separator("Cargo");

        let success = || -> Result<(), Error> {
            Executor::new(cargo_update, dry_run)
                .args(&["install-update", "--git", "--all"])
                .spawn()?
                .wait()?
                .check()?;

            Ok(())
        }()
        .is_ok();

        return Some(("Cargo", success));
    }

    None
}

#[must_use]
pub fn run_gem(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(gem) = utils::which("gem") {
        if base_dirs.home_dir().join(".gem").exists() {
            print_separator("RubyGems");

            let success = || -> Result<(), Error> {
                Executor::new(&gem, dry_run)
                    .args(&["update", "--user-install"])
                    .spawn()?
                    .wait()?
                    .check()?;

                Ok(())
            }()
            .is_ok();

            return Some(("RubyGems", success));
        }
    }
    None
}

#[must_use]
pub fn run_emacs(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(emacs) = utils::which("emacs") {
        if let Some(init_file) = base_dirs.home_dir().join(".emacs.d/init.el").if_exists() {
            print_separator("Emacs");

            let success = || -> Result<(), Error> {
                Executor::new(&emacs, dry_run)
                    .args(&["--batch", "-l", init_file.to_str().unwrap(), "--eval", EMACS_UPGRADE])
                    .spawn()?
                    .wait()?
                    .check()?;

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
pub fn run_apm(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(apm) = utils::which("apm") {
        print_separator("Atom Package Manager");

        let success = || -> Result<(), Error> {
            Executor::new(&apm, dry_run)
                .args(&["upgrade", "--confirm=false"])
                .spawn()?
                .wait()?
                .check()?;

            Ok(())
        }()
        .is_ok();

        return Some(("apm", success));
    }

    None
}

#[must_use]
pub fn run_rustup(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(rustup) = utils::which("rustup") {
        print_separator("rustup");

        let success = || -> Result<(), Error> {
            if rustup.is_descendant_of(base_dirs.home_dir()) {
                Executor::new(&rustup, dry_run)
                    .args(&["self", "update"])
                    .spawn()?
                    .wait()?
                    .check()?;
            }

            Executor::new(&rustup, dry_run).arg("update").spawn()?.wait()?.check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("rustup", success));
    }

    None
}

#[must_use]
pub fn run_jetpack(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(jetpack) = utils::which("jetpack") {
        print_separator("Jetpack");

        let success = || -> Result<(), Error> {
            Executor::new(&jetpack, dry_run)
                .args(&["global", "update"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("Jetpack", success));
    }

    None
}

#[must_use]
pub fn run_opam_update(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(opam) = utils::which("opam") {
        print_separator("OCaml Package Manager");

        let success = || -> Result<(), Error> {
            Executor::new(&opam, dry_run).arg("update").spawn()?.wait()?.check()?;
            Executor::new(&opam, dry_run).arg("upgrade").spawn()?.wait()?.check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("OPAM", success));
    }

    None
}

#[must_use]
pub fn run_vcpkg_update(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(vcpkg) = utils::which("vcpkg") {
        print_separator("vcpkg");

        let success = || -> Result<(), Error> {
            Executor::new(&vcpkg, dry_run)
                .args(&["upgrade", "--no-dry-run"])
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("vcpkg", success));
    }

    None
}

#[must_use]
pub fn run_pipx_update(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(pipx) = utils::which("pipx") {
        print_separator("pipx");

        let success = || -> Result<(), Error> {
            Executor::new(&pipx, dry_run)
                .arg("upgrade-all")
                .spawn()?
                .wait()?
                .check()?;
            Ok(())
        }()
        .is_ok();

        return Some(("pipx", success));
    }

    None
}

#[must_use]
pub fn run_custom_command(name: &str, command: &str, dry_run: bool) -> Result<(), Error> {
    print_separator(name);
    Executor::new("sh", dry_run)
        .arg("-c")
        .arg(command)
        .spawn()?
        .wait()?
        .check()?;

    Ok(())
}

#[must_use]
pub fn run_composer_update(base_dirs: &BaseDirs, dry_run: bool) -> Option<(&'static str, bool)> {
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
                    Executor::new(&composer, dry_run)
                        .args(&["global", "update"])
                        .spawn()?
                        .wait()?
                        .check()?;
                    Ok(())
                }()
                .is_ok();

                return Some(("Composer", success));
            }
        }
    }

    None
}

#[must_use]
pub fn run_gpg(dry_run: bool) -> Option<(&'static str, bool)> {
    if let Some(gpg) = utils::which("gpg") {
        print_separator("gpg keys");

        let success = || -> Result<(), Error> {
            Executor::new(&gpg, dry_run)
                .arg("--refresh-keys")
                .spawn()?
                .wait()?
                .check()?;

            Ok(())
        }()
        .is_ok();

        return Some(("gpg", success));
    }

    None
}
