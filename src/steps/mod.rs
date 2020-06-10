pub mod emacs;
pub mod generic;
pub mod git;
pub mod node;
pub mod os;
pub mod powershell;
#[cfg(unix)]
pub mod tmux;
pub mod vagrant;
pub mod vim;
#[cfg(unix)]
pub mod zsh;

pub use self::os::*;
