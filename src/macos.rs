use super::executor::Executor;
use super::terminal::print_separator;
use super::utils::Check;
use failure;

#[must_use]
pub fn upgrade_macos(dry_run: bool) -> Option<(&'static str, bool)> {
    print_separator("App Store");

    let success = || -> Result<(), failure::Error> {
        Executor::new("softwareupdate", dry_run)
            .args(&["--install", "--all"])
            .spawn()?
            .wait()?
            .check()?;
        Ok(())
    }().is_ok();

    Some(("App Store", success))
}
