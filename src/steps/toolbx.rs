use anyhow::Result;

use crate::config::Step;
use crate::terminal::print_separator;
use crate::{execution_context::ExecutionContext, utils::require};
use log::debug;
use std::path::Path;
use std::{path::PathBuf, process::Command};

fn list_toolboxes(toolbx: &Path) -> Result<Vec<String>> {
    let output = Command::new(toolbx).args(&["list", "--containers"]).output()?;
    let output_str = String::from_utf8(output.stdout)?;

    let proc: Vec<String> = output_str
        .lines()
        // Skip the first line since that contains only status information
        .skip(1)
        .map(|line| match line.split_whitespace().nth(1) {
            Some(word) => word.to_string(),
            None => String::from(""),
        })
        .filter(|x| !x.is_empty())
        .collect();

    Ok(proc)
}

pub fn run_toolbx(ctx: &ExecutionContext) -> Result<()> {
    let toolbx = require("toolbox")?;

    print_separator("Toolbx");
    let toolboxes = list_toolboxes(&toolbx)?;
    debug!("Toolboxes to inspect: {:?}", toolboxes);

    let mut topgrade_path = PathBuf::from("/run/host");
    // Path of the running topgrade executable
    // Skip 1 to eliminate the path root, otherwise push overwrites the path
    topgrade_path.push(std::env::current_exe()?.components().skip(1).collect::<PathBuf>());
    let topgrade_path = topgrade_path.to_str().unwrap();

    for tb in toolboxes.iter() {
        let topgrade_prefix = format!("TOPGRADE_PREFIX='Toolbx {}'", tb);
        let mut args = vec![
            "run",
            "-c",
            tb,
            "env",
            &topgrade_prefix,
            topgrade_path,
            "--only",
            "system",
        ];
        if ctx.config().yes(Step::Toolbx) {
            args.push("--yes");
        }

        let _output = ctx.run_type().execute(&toolbx).args(&args).check_run();
    }

    Ok(())
}
