use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;

#[must_use]
pub fn upgrade_macos(run_type: RunType) -> Option<(&'static str, bool)> {
    print_separator("App Store");

    let success = || -> Result<(), Error> {
        run_type
            .execute("softwareupdate")
            .args(&["--install", "--all"])
            .check_run()?;
        Ok(())
    }()
    .is_ok();

    Some(("App Store", success))
}
