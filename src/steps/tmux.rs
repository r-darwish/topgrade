use crate::error::{Error, ErrorKind};
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::{which, Check, PathExt};
use directories::BaseDirs;
use failure::ResultExt;
use std::env;
use std::io;
use std::os::unix::process::CommandExt;
use std::path::Path;
use std::process::{exit, Command};

pub fn run_tpm(base_dirs: &BaseDirs, run_type: RunType) -> Result<(), Error> {
    let tpm = base_dirs
        .home_dir()
        .join(".tmux/plugins/tpm/bin/update_plugins")
        .require()?;

    print_separator("tmux plugins");

    run_type.execute(&tpm).arg("all").check_run()
}

fn has_session(tmux: &Path, session_name: &str) -> Result<bool, io::Error> {
    Ok(Command::new(tmux)
        .args(&["has-session", "-t", session_name])
        .env_remove("TMUX")
        .output()?
        .status
        .success())
}

fn new_session(tmux: &Path, session_name: &str) -> Result<bool, io::Error> {
    Ok(Command::new(tmux)
        .args(&["new-session", "-d", "-s", session_name, "-n", "dummy"])
        .env_remove("TMUX")
        .spawn()?
        .wait()?
        .success())
}

fn run_in_session(tmux: &Path, command: &str) -> Result<(), Error> {
    Command::new(tmux)
        .args(&["new-window", "-a", "-t", "topgrade:1", "-n", "local", command])
        .env_remove("TMUX")
        .spawn()
        .context(ErrorKind::ProcessExecution)?
        .wait()
        .context(ErrorKind::ProcessExecution)?
        .check()?;

    Ok(())
}

pub fn run_in_tmux() -> ! {
    let tmux = which("tmux").expect("Could not find tmux");
    let command = {
        let mut command = vec![
            String::from("env"),
            String::from("TOPGRADE_KEEP_END=1"),
            String::from("TOPGRADE_INSIDE_TMUX=1"),
        ];
        command.extend(env::args());
        command.join(" ")
    };

    if !has_session(&tmux, "topgrade").expect("Error launching tmux") {
        new_session(&tmux, "topgrade").expect("Error launching tmux");
    }

    run_in_session(&tmux, &command).expect("Error launching tmux");
    Command::new(&tmux)
        .args(&["kill-window", "-t", "topgrade:dummy"])
        .output()
        .unwrap();

    if env::var("TMUX").is_err() {
        let err = Command::new(tmux).args(&["attach", "-t", "topgrade"]).exec();
        panic!("{:?}", err);
    } else {
        println!("Topgrade launched in a new tmux session");
        exit(0);
    }
}

pub fn run_remote_topgrade(hostname: &str, ssh: &Path) -> Result<(), Error> {
    let command = format!(
        "{ssh} -t {hostname} env TOPGRADE_PREFIX={hostname} TOPGRADE_KEEP_END=1 topgrade",
        ssh = ssh.display(),
        hostname = hostname
    );
    Command::new(which("tmux").unwrap())
        .args(&["new-window", "-a", "-t", "topgrade:1", "-n", hostname, &command])
        .env_remove("TMUX")
        .spawn()
        .context(ErrorKind::ProcessExecution)?
        .wait()
        .context(ErrorKind::ProcessExecution)?
        .check()
}
