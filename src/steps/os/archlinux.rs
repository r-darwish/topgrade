use crate::config;
use crate::execution_context::ExecutionContext;
use crate::utils::which;
use anyhow::Result;
use std::env::var_os;
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use walkdir::WalkDir;

fn get_execution_path() -> OsString {
    let mut path = OsString::from("/usr/bin:");
    path.push(var_os("PATH").unwrap());
    path
}

pub trait ArchPackageManager {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()>;
}

pub struct YayParu {
    executable: PathBuf,
    pacman: PathBuf,
}

impl ArchPackageManager for YayParu {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        if ctx.config().show_arch_news() {
            Command::new(&self.executable)
                .arg("-Pw")
                .spawn()
                .and_then(|mut p| p.wait())
                .ok();
        }

        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("--pacman")
            .arg(&self.pacman)
            .arg("-Syu")
            .args(ctx.config().yay_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes() {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("--pacman").arg(&self.pacman).arg("-Scc");
            if ctx.config().yes() {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }

        Ok(())
    }
}

impl YayParu {
    fn get(exec_name: &str, pacman: &Path) -> Option<Self> {
        Some(Self {
            executable: which(exec_name)?,
            pacman: pacman.to_owned(),
        })
    }
}

pub struct Trizen {
    executable: PathBuf,
}

impl ArchPackageManager for Trizen {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.executable);

        command
            .arg("-Syu")
            .args(ctx.config().trizen_arguments().split_whitespace())
            .env("PATH", get_execution_path());

        if ctx.config().yes() {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.executable);
            command.arg("-Sc");
            if ctx.config().yes() {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }

        Ok(())
    }
}

impl Trizen {
    fn get() -> Option<Self> {
        Some(Self {
            executable: which("trizen")?,
        })
    }
}

pub struct Pacman {
    sudo: PathBuf,
    executable: PathBuf,
}

impl ArchPackageManager for Pacman {
    fn upgrade(&self, ctx: &ExecutionContext) -> Result<()> {
        let mut command = ctx.run_type().execute(&self.sudo);
        command
            .arg(&self.executable)
            .arg("-Syu")
            .env("PATH", get_execution_path());
        if ctx.config().yes() {
            command.arg("--noconfirm");
        }
        command.check_run()?;

        if ctx.config().cleanup() {
            let mut command = ctx.run_type().execute(&self.sudo);
            command.arg(&self.executable).arg("-Scc");
            if ctx.config().yes() {
                command.arg("--noconfirm");
            }
            command.check_run()?;
        }

        Ok(())
    }
}

impl Pacman {
    pub fn get(ctx: &ExecutionContext) -> Option<Self> {
        Some(Self {
            executable: which("powerpill").unwrap_or_else(|| PathBuf::from("/usr/bin/pacman")),
            sudo: ctx.sudo().to_owned()?,
        })
    }
}

fn box_pacakge_manager<P: 'static + ArchPackageManager>(package_manager: P) -> Box<dyn ArchPackageManager> {
    Box::new(package_manager) as Box<dyn ArchPackageManager>
}

pub fn get_arch_package_manager(ctx: &ExecutionContext) -> Option<Box<dyn ArchPackageManager>> {
    let pacman = which("powerpill").unwrap_or_else(|| PathBuf::from("/usr/bin/pacman"));

    match ctx.config().arch_package_manager() {
        config::ArchPackageManager::Autodetect => YayParu::get("paru", &pacman)
            .map(box_pacakge_manager)
            .or_else(|| YayParu::get("yay", &pacman).map(box_pacakge_manager))
            .or_else(|| {
                Trizen::get()
                    .map(box_pacakge_manager)
                    .or_else(|| Pacman::get(ctx).map(box_pacakge_manager))
            }),
        config::ArchPackageManager::Trizen => Trizen::get().map(box_pacakge_manager),
        config::ArchPackageManager::Paru => YayParu::get("paru", &pacman).map(box_pacakge_manager),
        config::ArchPackageManager::Yay => YayParu::get("yay", &pacman).map(box_pacakge_manager),
        config::ArchPackageManager::Pacman => Pacman::get(ctx).map(box_pacakge_manager),
    }
}

pub fn upgrade_arch_linux(ctx: &ExecutionContext) -> Result<()> {
    let package_manager = get_arch_package_manager(ctx).unwrap();
    package_manager.upgrade(ctx)
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
