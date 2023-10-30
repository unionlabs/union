//! POSIX-style filesystem functions which operate on bare paths.

#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
use crate::fs::StatFs;
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
use {
    crate::fs::StatVfs,
    crate::{backend, io, path},
};

/// `statfs`—Queries filesystem metadata.
///
/// Compared to [`statvfs`], this function often provides more information,
/// though it's less portable.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/statfs.2.html
#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[inline]
pub fn statfs<P: path::Arg>(path: P) -> io::Result<StatFs> {
    path.into_with_c_str(backend::fs::syscalls::statfs)
}

/// `statvfs`—Queries filesystem metadata, POSIX version.
///
/// Compared to [`statfs`], this function often provides less information,
/// but it is more portable. But even so, filesystems are very diverse and not
/// all the fields are meaningful for every filesystem. And `f_fsid` doesn't
/// seem to have a clear meaning anywhere.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/statvfs.html
/// [Linux]: https://man7.org/linux/man-pages/man2/statvfs.2.html
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
#[inline]
pub fn statvfs<P: path::Arg>(path: P) -> io::Result<StatVfs> {
    path.into_with_c_str(backend::fs::syscalls::statvfs)
}
