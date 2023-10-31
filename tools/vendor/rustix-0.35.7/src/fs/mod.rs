//! Filesystem operations.

#[cfg(feature = "fs")]
mod abs;
#[cfg(not(target_os = "redox"))]
#[cfg(any(feature = "fs", feature = "procfs"))]
mod at;
mod constants;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod copy_file_range;
#[cfg(not(target_os = "redox"))]
mod cwd;
#[cfg(not(target_os = "redox"))]
#[cfg(any(feature = "fs", feature = "procfs"))]
mod dir;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
mod fadvise;
pub(crate) mod fcntl;
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod fcntl_darwin;
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod fcopyfile;
pub(crate) mod fd;
mod file_type;
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod getpath;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
mod makedev;
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
mod memfd_create;
#[cfg(any(target_os = "android", target_os = "linux"))]
#[cfg(feature = "fs")]
mod openat2;
#[cfg(target_os = "linux")]
mod sendfile;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod statx;

#[cfg(not(any(
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[cfg(feature = "fs")]
pub use abs::statfs;
#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi")))]
#[cfg(feature = "fs")]
pub use abs::statvfs;
#[cfg(not(any(target_os = "illumos", target_os = "redox")))]
#[cfg(feature = "fs")]
pub use at::accessat;
#[cfg(any(target_os = "ios", target_os = "macos"))]
#[cfg(feature = "fs")]
pub use at::fclonefileat;
#[cfg(not(any(
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
    target_os = "wasi",
)))]
#[cfg(feature = "fs")]
pub use at::mknodat;
#[cfg(any(target_os = "android", target_os = "linux"))]
#[cfg(feature = "fs")]
pub use at::renameat_with;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[cfg(feature = "fs")]
pub use at::{chmodat, chownat};
#[cfg(not(target_os = "redox"))]
#[cfg(any(feature = "fs", feature = "procfs"))]
pub use at::{
    linkat, mkdirat, openat, readlinkat, renameat, statat, symlinkat, unlinkat, utimensat, RawMode,
    UTIME_NOW, UTIME_OMIT,
};
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use constants::CloneFlags;
/// `copyfile_flags_t`
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use constants::CopyfileFlags;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use constants::RenameFlags;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use constants::ResolveFlags;
pub use constants::{Access, FdFlags, Mode, Nsecs, OFlags, Secs, Timespec};
#[cfg(not(target_os = "redox"))]
pub use constants::{AtFlags, Dev};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use copy_file_range::copy_file_range;
#[cfg(not(target_os = "redox"))]
pub use cwd::cwd;
#[cfg(not(target_os = "redox"))]
#[cfg(any(feature = "fs", feature = "procfs"))]
pub use dir::{Dir, DirEntry};
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use fadvise::{fadvise, Advice};
#[cfg(not(target_os = "wasi"))]
pub use fcntl::fcntl_dupfd_cloexec;
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "linux",
))]
pub use fcntl::{fcntl_add_seals, fcntl_get_seals, SealFlags};
pub use fcntl::{fcntl_getfd, fcntl_getfl, fcntl_setfd, fcntl_setfl};
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use fcntl_darwin::{fcntl_fullfsync, fcntl_rdadvise};
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use fcopyfile::{
    copyfile_state_alloc, copyfile_state_free, copyfile_state_get, copyfile_state_get_copied,
    copyfile_state_t, fcopyfile,
};
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
)))]
pub use fd::fdatasync;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub use fd::{fallocate, FallocateFlags};
#[cfg(not(target_os = "wasi"))]
pub use fd::{fchmod, fchown, flock, FlockOperation};
pub use fd::{fstat, fsync, ftruncate, futimens, is_file_read_write, seek, tell, Stat, Timestamps};
#[cfg(not(any(
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
pub use fd::{fstatfs, StatFs};
#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi")))]
pub use fd::{fstatvfs, StatVfs, StatVfsMountFlags};
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use fd::{FsWord, NFS_SUPER_MAGIC, PROC_SUPER_MAGIC};
pub use file_type::FileType;
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub use getpath::getpath;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
pub use makedev::{major, makedev, minor};
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
pub use memfd_create::{memfd_create, MemfdFlags};
#[cfg(any(target_os = "android", target_os = "linux"))]
#[cfg(feature = "fs")]
pub use openat2::openat2;
#[cfg(target_os = "linux")]
pub use sendfile::sendfile;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use statx::{statx, Statx, StatxFlags, StatxTimestamp};

/// Re-export types common to POSIX-ish platforms.
#[cfg(feature = "std")]
#[cfg(unix)]
pub use std::os::unix::fs::{DirEntryExt, FileExt, FileTypeExt, MetadataExt, OpenOptionsExt};
#[cfg(feature = "std")]
#[cfg(target_os = "wasi")]
pub use std::os::wasi::fs::{DirEntryExt, FileExt, FileTypeExt, MetadataExt, OpenOptionsExt};
