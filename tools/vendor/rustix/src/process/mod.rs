//! Process-associated operations.

#[cfg(not(target_os = "wasi"))]
mod chdir;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
mod chroot;
mod exit;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have get[gpu]id.
mod id;
mod ioctl;
#[cfg(not(target_os = "wasi"))]
mod kill;
#[cfg(linux_kernel)]
mod membarrier;
#[cfg(target_os = "linux")]
mod pidfd;
#[cfg(target_os = "linux")]
mod pidfd_getfd;
#[cfg(linux_kernel)]
mod prctl;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))] // WASI doesn't have [gs]etpriority.
mod priority;
#[cfg(freebsdlike)]
mod procctl;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
mod rlimit;
#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
mod sched;
mod sched_yield;
#[cfg(not(target_os = "wasi"))] // WASI doesn't have umask.
mod umask;
#[cfg(not(target_os = "wasi"))]
mod wait;

#[cfg(not(target_os = "wasi"))]
pub use chdir::*;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub use chroot::*;
pub use exit::*;
#[cfg(not(target_os = "wasi"))]
pub use id::*;
pub use ioctl::*;
#[cfg(not(target_os = "wasi"))]
pub use kill::*;
#[cfg(linux_kernel)]
pub use membarrier::*;
#[cfg(target_os = "linux")]
pub use pidfd::*;
#[cfg(target_os = "linux")]
pub use pidfd_getfd::*;
#[cfg(linux_kernel)]
pub use prctl::*;
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub use priority::*;
#[cfg(freebsdlike)]
pub use procctl::*;
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
pub use rlimit::*;
#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
pub use sched::*;
pub use sched_yield::sched_yield;
#[cfg(not(target_os = "wasi"))]
pub use umask::*;
#[cfg(not(target_os = "wasi"))]
pub use wait::*;
