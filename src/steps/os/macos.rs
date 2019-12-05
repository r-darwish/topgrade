use crate::error::Error;
use crate::executor::RunType;
use crate::terminal::print_separator;

#[must_use]
pub fn upgrade_macos(run_type: RunType) -> Result<()> {
    print_separator("App Store");

    run_type
        .execute("softwareupdate")
        .args(&["--install", "--all"])
        .check_run()
}
