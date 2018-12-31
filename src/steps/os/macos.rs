use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;
use crate::utils::Check;

#[must_use]
pub fn upgrade_macos(run_type: RunType) -> Option<(&'static str, bool)> {
    print_separator("App Store");

    let success = || -> Result<(), Error> {
        run_type
            .execute("softwareupdate")
            .args(&["--install", "--all"])
            .spawn()?
            .wait()?
            .check()?;
        Ok(())
    }()
    .is_ok();

    Some(("App Store", success))
}
