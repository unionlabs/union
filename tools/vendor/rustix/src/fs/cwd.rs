//! The `cwd` function, representing the current working directory.
//!
//! # Safety
//!
//! This file uses `AT_FDCWD`, which is a raw file descriptor, but which is
//! always valid.

#![allow(unsafe_code)]

use crate::backend;
use backend::c;
use backend::fd::{BorrowedFd, RawFd};

/// `AT_FDCWD`—A handle representing the current working directory.
///
/// This is a file descriptor which refers to the process current directory
/// which can be used as the directory argument in `*at` functions such as
/// [`openat`].
///
/// # References
///  - [POSIX]
///
/// [`openat`]: crate::fs::openat
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/fcntl.h.html
// SAFETY: `AT_FDCWD` is a reserved value that is never dynamically
// allocated, so it'll remain valid for the duration of `'static`.
#[doc(alias = "AT_FDCWD")]
#[cfg(not(target_os = "haiku"))] // Haiku needs <https://github.com/rust-lang/rust/pull/112371>
pub const CWD: BorrowedFd<'static> =
    unsafe { BorrowedFd::<'static>::borrow_raw(c::AT_FDCWD as RawFd) };

/// Return the value of [`CWD`].
#[deprecated(note = "Use `CWD` in place of `cwd()`.")]
#[cfg(not(target_os = "haiku"))] // Haiku needs <https://github.com/rust-lang/rust/pull/112371>
pub const fn cwd() -> BorrowedFd<'static> {
    let at_fdcwd = c::AT_FDCWD as RawFd;

    // SAFETY: `AT_FDCWD` is a reserved value that is never dynamically
    // allocated, so it'll remain valid for the duration of `'static`.
    unsafe { BorrowedFd::<'static>::borrow_raw(at_fdcwd) }
}
