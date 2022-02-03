use anyhow::Result;

use crate::error;
use crate::terminal::print_separator;
use crate::{execution_context::ExecutionContext, utils::require};
use log::debug;
use std::path::Path;
use std::process::Command;

/// Returns a Vector of all containers, with Strings in the format
/// "REGISTRY/[PATH/]CONTAINER_NAME:TAG"
fn list_containers(crt: &Path) -> Result<Vec<String>> {
    let output = Command::new(crt)
        .args(&["images", "--format", "{{.Repository}}:{{.Tag}}"])
        .output()?;
    let output_str = String::from_utf8(output.stdout)?;

    let mut retval = vec![];
    for line in output_str.lines() {
        if line.starts_with("localhost") {
            // Don't know how to update self-built containers
            debug!("Skipping self-built container '{}'", line);
            continue;
        }

        retval.push(String::from(line));
    }

    Ok(retval)
}

pub fn run_containers(ctx: &ExecutionContext) -> Result<()> {
    // Prefer podman, fall back to docker if not present
    let crt = match require("podman") {
        Ok(path) => path,
        Err(_) => require("docker")?,
    };
    debug!("Using container runtime '{}'", crt.display());

    print_separator("Containers");
    let mut success = true;
    let containers = list_containers(&crt)?;
    debug!("Containers to inspect: {:?}", containers);

    for container in containers.iter() {
        let args = vec!["pull", &container[..]];

        debug!("Pulling container '{}'", container);
        if let Err(e) = ctx.run_type().execute(&crt).args(&args).check_run() {
            // FIXME: Should this print a warning to stderr?
            debug!("Pulling container '{}' failed: {}", container, e);
            success = false;
        }
    }

    if ctx.config().cleanup() {
        // Remove dangling images
        debug!("Removing dangling images");
        if let Err(e) = ctx.run_type().execute(&crt).args(&["image", "prune", "-f"]).check_run() {
            debug!("Removing dangling images failed: {}", e);
            success = false;
        }
    }

    if success {
        Ok(())
    } else {
        Err(anyhow::anyhow!(error::StepFailed))
    }
}
