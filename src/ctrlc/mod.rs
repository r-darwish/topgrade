mod interrupted;

#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::set_handler;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::set_handler;

pub use self::interrupted::*;
