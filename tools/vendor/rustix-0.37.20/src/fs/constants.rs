//! Filesystem API constants, translated into `bitflags` constants.

use crate::backend;

pub use crate::io::FdFlags;
pub use backend::fs::types::{Access, Dev, Mode, OFlags};

#[cfg(not(target_os = "redox"))]
pub use backend::fs::types::AtFlags;

#[cfg(apple)]
pub use backend::fs::types::{CloneFlags, CopyfileFlags};

#[cfg(linux_kernel)]
pub use backend::fs::types::*;

pub use backend::time::types::{Nsecs, Secs, Timespec};
