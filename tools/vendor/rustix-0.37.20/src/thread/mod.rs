//! Thread-associated operations.

#[cfg(not(target_os = "redox"))]
mod clock;
#[cfg(linux_raw)]
mod futex;
#[cfg(linux_kernel)]
mod id;
#[cfg(linux_kernel)]
mod libcap;
#[cfg(linux_kernel)]
mod prctl;
#[cfg(linux_kernel)]
mod setns;

#[cfg(not(target_os = "redox"))]
pub use clock::*;
#[cfg(linux_raw)]
pub use futex::{futex, FutexFlags, FutexOperation};
#[cfg(linux_kernel)]
pub use id::{gettid, set_thread_gid, set_thread_res_gid, set_thread_res_uid, set_thread_uid};
#[cfg(linux_kernel)]
pub use libcap::{capabilities, set_capabilities, CapabilityFlags, CapabilitySets};
#[cfg(linux_kernel)]
pub use prctl::*;
#[cfg(linux_kernel)]
pub use setns::*;
