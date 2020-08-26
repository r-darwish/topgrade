//! SIGINT handling in Unix systems.
use crate::ctrlc::interrupted::set_interrupted;
use nix::sys::signal;

/// Handle SIGINT. Set the interruption flag.
extern "C" fn handle_sigint(_: i32) {
    set_interrupted()
}

/// Set the necessary signal handlers.
/// The function panics on failure.
pub fn set_handler() {
    let sig_action = signal::SigAction::new(
        signal::SigHandler::Handler(handle_sigint),
        signal::SaFlags::empty(),
        signal::SigSet::empty(),
    );
    unsafe {
        signal::sigaction(signal::SIGINT, &sig_action).unwrap();
    }
}
