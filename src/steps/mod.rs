pub mod generic;
pub mod git;
pub mod node;
pub mod os;
#[cfg(unix)]
pub mod tmux;
pub mod vim;

pub use self::os::*;
