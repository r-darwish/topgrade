//! SIGINT handling in Unix systems.
use lazy_static::lazy_static;
use nix::sys::signal;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    /// A global variable telling whether the application has been interrupted.
    static ref INTERRUPTED: AtomicBool = AtomicBool::new(false);
}

/// Tells whether the program has been interrupted
pub fn interrupted() -> bool {
    INTERRUPTED.load(Ordering::SeqCst)
}

/// Clears the interrupted flag
pub fn unset_interrupted() {
    debug_assert!(INTERRUPTED.load(Ordering::SeqCst));
    INTERRUPTED.store(false, Ordering::SeqCst)
}

/// Handle SIGINT. Set the interruption flag.
extern "C" fn handle_sigint(_: i32) {
    INTERRUPTED.store(true, Ordering::SeqCst)
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
