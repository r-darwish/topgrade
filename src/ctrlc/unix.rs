use lazy_static::lazy_static;
use nix::sys::signal;
use std::sync::atomic::{AtomicBool, Ordering};

lazy_static! {
    static ref RUNNING: AtomicBool = AtomicBool::new(true);
}

pub fn running() -> bool {
    RUNNING.load(Ordering::SeqCst)
}

pub fn set_running(value: bool) {
    RUNNING.store(value, Ordering::SeqCst)
}

extern "C" fn handle_sigint(_: i32) {
    set_running(false);
}

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
