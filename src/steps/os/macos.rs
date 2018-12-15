use crate::error::Error;
use crate::executor::Executor;
use crate::terminal::print_separator;
use crate::utils::Check;

#[must_use]
pub fn upgrade_macos(dry_run: bool) -> Option<(&'static str, bool)> {
    print_separator("App Store");

    let success = || -> Result<(), Error> {
        Executor::new("softwareupdate", dry_run)
            .args(&["--install", "--all"])
            .spawn()?
            .wait()?
            .check()?;
        Ok(())
    }()
    .is_ok();

    Some(("App Store", success))
}
