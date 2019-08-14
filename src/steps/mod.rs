pub mod emacs;
pub mod generic;
pub mod git;
pub mod node;
pub mod os;
pub mod powershell;
#[cfg(unix)]
pub mod tmux;
pub mod vim;

pub use self::os::*;
