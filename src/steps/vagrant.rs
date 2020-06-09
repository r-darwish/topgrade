use crate::execution_context::ExecutionContext;
use crate::executor::CommandExt;
use crate::terminal::print_separator;
use crate::utils;
use anyhow::Result;
use log::debug;
use std::path::PathBuf;
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

impl Vagrant {
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
            if !status.powered_on() && !(ctx.config().vagrant_power_on().unwrap_or(false)) {
                debug!("Skipping powered off box {}", vagrant_box);
                continue;
            }

            println!("Running Topgrade in {} @ {}", vagrant_box, directory);
            let command = format!("env TOPGRADE_PREFIX={} topgrade", vagrant_box);
            ctx.run_type()
                .execute(&vagrant.path)
                .args(&["ssh", "-c", &command])
                .check_run()?;
        }
    }
    Ok(())
}
