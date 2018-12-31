//! Provides handling for process interruption.
//! There's no actual handling for Windows at the moment.
#[cfg(unix)]
mod unix;
#[cfg(unix)]
pub use self::unix::*;

#[cfg(windows)]
mod windows;
#[cfg(windows)]
pub use self::windows::*;
