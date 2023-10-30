//! Filesystem operations.

mod abs;
#[cfg(not(target_os = "redox"))]
mod at;
mod constants;
#[cfg(linux_kernel)]
mod copy_file_range;
#[cfg(not(target_os = "redox"))]
mod cwd;
#[cfg(not(target_os = "redox"))]
mod dir;
#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
mod fadvise;
pub(crate) mod fcntl;
#[cfg(apple)]
mod fcntl_apple;
#[cfg(apple)]
mod fcopyfile;
pub(crate) mod fd;
mod file_type;
#[cfg(apple)]
mod getpath;
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
mod makedev;
#[cfg(any(linux_kernel, target_os = "freebsd"))]
mod memfd_create;
#[cfg(linux_kernel)]
mod mount;
#[cfg(linux_kernel)]
mod openat2;
#[cfg(linux_kernel)]
mod raw_dir;
#[cfg(target_os = "linux")]
mod sendfile;
#[cfg(linux_kernel)]
mod statx;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
mod sync;
#[cfg(any(apple, linux_kernel))]
mod xattr;

#[cfg(linux_kernel)]
pub use crate::backend::fs::inotify;
pub use abs::*;
#[cfg(not(target_os = "redox"))]
pub use at::*;
pub use constants::*;
#[cfg(linux_kernel)]
pub use copy_file_range::copy_file_range;
#[cfg(not(target_os = "redox"))]
pub use cwd::cwd;
#[cfg(not(target_os = "redox"))]
pub use dir::{Dir, DirEntry};
#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
pub use fadvise::{fadvise, Advice};
pub use fcntl::*;
#[cfg(apple)]
pub use fcntl_apple::{fcntl_fullfsync, fcntl_rdadvise};
#[cfg(apple)]
pub use fcopyfile::*;
pub use fd::*;
pub use file_type::FileType;
#[cfg(apple)]
pub use getpath::getpath;
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub use makedev::*;
#[cfg(any(linux_kernel, target_os = "freebsd"))]
pub use memfd_create::{memfd_create, MemfdFlags};
#[cfg(linux_kernel)]
pub use mount::*;
#[cfg(linux_kernel)]
pub use openat2::openat2;
#[cfg(linux_kernel)]
pub use raw_dir::{RawDir, RawDirEntry};
#[cfg(target_os = "linux")]
pub use sendfile::sendfile;
#[cfg(linux_kernel)]
pub use statx::{statx, Statx, StatxFlags, StatxTimestamp};
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub use sync::sync;
#[cfg(any(apple, linux_kernel))]
pub use xattr::*;

/// Re-export types common to POSIX-ish platforms.
#[cfg(feature = "std")]
#[cfg(unix)]
pub use std::os::unix::fs::{DirEntryExt, FileExt, FileTypeExt, MetadataExt, OpenOptionsExt};
#[cfg(feature = "std")]
#[cfg(all(wasi_ext, target_os = "wasi"))]
pub use std::os::wasi::fs::{DirEntryExt, FileExt, FileTypeExt, MetadataExt, OpenOptionsExt};
