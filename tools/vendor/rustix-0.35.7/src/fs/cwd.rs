//! The `cwd` function, representing the current working directory.
//!
//! # Safety
//!
//! This file uses `AT_FDCWD`, which is a raw file descriptor, but which is
//! always valid.

#![allow(unsafe_code)]

use crate::backend;
use backend::fd::{BorrowedFd, RawFd};

/// `AT_FDCWD`—Returns a handle representing the current working directory.
///
/// This returns a file descriptor which refers to the process current
/// directory which can be used as the directory argument in `*at`
/// functions such as [`openat`].
///
/// # References
///  - [POSIX]
///
/// [`openat`]: crate::fs::openat
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fcntl.h.html
#[inline]
#[doc(alias = "AT_FDCWD")]
pub const fn cwd() -> BorrowedFd<'static> {
    let at_fdcwd = backend::io::types::AT_FDCWD as RawFd;

    // Safety: `AT_FDCWD` is a reserved value that is never dynamically
    // allocated, so it'll remain valid for the duration of `'static`.
    unsafe { BorrowedFd::<'static>::borrow_raw(at_fdcwd) }
}
