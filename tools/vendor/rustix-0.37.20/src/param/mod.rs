//! Process parameters.
//!
//! These values correspond to `sysconf` in POSIX, and the auxv array in Linux.
//! Despite the POSIX name “sysconf”, these aren't *system* configuration
//! parameters; they're *process* configuration parameters, as they may differ
//! between different processes on the same system.

#[cfg(feature = "param")]
mod auxv;
#[cfg(target_vendor = "mustang")]
mod init;

#[cfg(feature = "param")]
pub use auxv::*;
#[cfg(target_vendor = "mustang")]
pub use init::init;
