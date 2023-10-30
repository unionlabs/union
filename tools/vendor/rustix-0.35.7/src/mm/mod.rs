//! Memory map operations.

#[cfg(not(target_os = "redox"))]
mod madvise;
mod mmap;
mod msync;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod userfaultfd;

#[cfg(not(target_os = "redox"))]
pub use madvise::{madvise, Advice};
pub use mmap::{
    mlock, mmap, mmap_anonymous, mprotect, munlock, munmap, MapFlags, MprotectFlags, ProtFlags,
};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use mmap::{mlock_with, MlockFlags};
#[cfg(any(linux_raw, all(libc, target_os = "linux")))]
pub use mmap::{mremap, mremap_fixed, MremapFlags};
pub use msync::{msync, MsyncFlags};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use userfaultfd::{userfaultfd, UserfaultfdFlags};
