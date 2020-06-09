use crate::execution_context::ExecutionContext;
use crate::executor::CommandExt;
use crate::terminal::print_separator;
use crate::utils;
use anyhow::Result;
use log::debug;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::str::FromStr;
use strum::EnumString;

#[derive(Debug, Copy, Clone, EnumString)]
#[strum(serialize_all = "lowercase")]
enum BoxStatus {
    PowerOff,
    Running,
}

impl BoxStatus {
    fn powered_on(self) -> bool {
        match self {
            BoxStatus::PowerOff => false,
            BoxStatus::Running => true,
        }
    }
}

struct Vagrant {
    path: PathBuf,
}

impl<'a> Vagrant {
    fn get_box_status(&self, directory: &str) -> Result<Vec<(String, BoxStatus)>> {
        let output = Command::new(&self.path)
            .arg("status")
            .current_dir(directory)
            .check_output()?;
        debug!("Vagrant output in {}: {}", directory, output);

        let boxes = output
            .split('\n')
            .skip(2)
            .take_while(|line| !(line.is_empty() || line.starts_with('\r')))
            .map(|line| {
                debug!("Vagrant line: {:?}", line);
                let mut elements = line.split_whitespace();
                let box_name = elements.next().unwrap();
                let box_status = BoxStatus::from_str(elements.next().unwrap()).unwrap();
                debug!("{:?}: {:?}", box_name, box_status);
                (box_name.to_string(), box_status)
            })
            .collect();

        Ok(boxes)
    }

    fn temporary_power_on(
        &'a self,
        directory: &'a str,
        vagrant_box: &'a str,
        ctx: &'a ExecutionContext,
    ) -> Result<TemporaryPowerOn<'a>> {
        TemporaryPowerOn::create(&self.path, directory, vagrant_box, ctx)
    }
}

struct TemporaryPowerOn<'a> {
    vagrant: &'a Path,
    directory: &'a str,
    vagrant_box: &'a str,
    ctx: &'a ExecutionContext<'a>,
}

impl<'a> TemporaryPowerOn<'a> {
    fn create(
        vagrant: &'a Path,
        directory: &'a str,
        vagrant_box: &'a str,
        ctx: &'a ExecutionContext<'a>,
    ) -> Result<TemporaryPowerOn<'a>> {
        println!("Powering on {} @ {}", vagrant_box, directory);
        ctx.run_type().execute(vagrant).args(&["up", vagrant_box]).check_run()?;
        Ok(TemporaryPowerOn {
            vagrant,
            directory,
            vagrant_box,
            ctx,
        })
    }
}

impl<'a> Drop for TemporaryPowerOn<'a> {
    fn drop(&mut self) {
        println!("Powering off {} @ {}", self.vagrant_box, self.directory);
        self.ctx
            .run_type()
            .execute(self.vagrant)
            .args(&["halt", self.vagrant_box])
            .current_dir(self.directory)
            .check_run()
            .ok();
    }
}

pub fn topgrade_vagrant_boxes(ctx: &ExecutionContext) -> Result<()> {
    let directories = utils::require_option(ctx.config().vagrant_directories())?;
    let vagrant = Vagrant {
        path: utils::require("vagrant")?,
    };

    print_separator("Vagrant");

    for directory in directories {
        let boxes = vagrant.get_box_status(directory)?;
        debug!("{:?}", boxes);
        for (vagrant_box, status) in boxes {
            let mut _poweron = None;
            if !status.powered_on() {
                if !(ctx.config().vagrant_power_on().unwrap_or(true)) {
                    debug!("Skipping powered off box {}", vagrant_box);
                    continue;
                } else {
                    _poweron = Some(vagrant.temporary_power_on(directory, &vagrant_box, ctx)?);
                }
            }

            println!("Running Topgrade in {} @ {}", vagrant_box, directory);
            let mut command = format!("env TOPGRADE_PREFIX={} topgrade", vagrant_box);
            if ctx.config().yes() {
                command.push_str(" -y");
            }

            ctx.run_type()
                .execute(&vagrant.path)
                .args(&["ssh", "-c", &command])
                .check_run()?;
        }
    }
    Ok(())
}
