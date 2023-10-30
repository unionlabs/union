//! Unix user, group, and process identifiers.
//!
//! # Safety
//!
//! The `Uid`, `Gid`, and `Pid` types can be constructed from raw integers,
//! which is marked unsafe because actual OS's assign special meaning to some
//! integer values.
#![allow(unsafe_code)]

use crate::{backend, io};
use alloc::vec::Vec;
#[cfg(linux_kernel)]
use backend::process::types::RawCpuid;

/// The raw integer value of a Unix user ID.
pub use crate::ugid::RawUid;

/// The raw integer value of a Unix group ID.
pub use crate::ugid::RawGid;

/// The raw integer value of a Unix process ID.
pub use crate::pid::RawPid;

pub use crate::pid::Pid;
pub use crate::ugid::{Gid, Uid};

/// A Linux CPU ID.
#[cfg(linux_kernel)]
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Cpuid(RawCpuid);

#[cfg(linux_kernel)]
impl Cpuid {
    /// Converts a `RawCpuid` into a `Cpuid`.
    ///
    /// # Safety
    ///
    /// `raw` must be the value of a valid Linux CPU ID.
    #[inline]
    pub const unsafe fn from_raw(raw: RawCpuid) -> Self {
        Self(raw)
    }

    /// Converts a `Cpuid` into a `RawCpuid`.
    #[inline]
    pub const fn as_raw(self) -> RawCpuid {
        self.0
    }
}

/// `getuid()`—Returns the process' real user ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getuid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getuid.2.html
#[inline]
#[must_use]
pub fn getuid() -> Uid {
    backend::ugid::syscalls::getuid()
}

/// `geteuid()`—Returns the process' effective user ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/geteuid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/geteuid.2.html
#[inline]
#[must_use]
pub fn geteuid() -> Uid {
    backend::ugid::syscalls::geteuid()
}

/// `getgid()`—Returns the process' real group ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getgid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getgid.2.html
#[inline]
#[must_use]
pub fn getgid() -> Gid {
    backend::ugid::syscalls::getgid()
}

/// `getegid()`—Returns the process' effective group ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getegid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getegid.2.html
#[inline]
#[must_use]
pub fn getegid() -> Gid {
    backend::ugid::syscalls::getegid()
}

/// `getpid()`—Returns the process' ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getpid.2.html
#[inline]
#[must_use]
pub fn getpid() -> Pid {
    backend::pid::syscalls::getpid()
}

/// `getppid()`—Returns the parent process' ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getppid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getppid.2.html
#[inline]
#[must_use]
pub fn getppid() -> Option<Pid> {
    backend::process::syscalls::getppid()
}

/// `getpgid(pid)`—Returns the process group ID of the given process.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpgid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getpgid.2.html
#[inline]
pub fn getpgid(pid: Option<Pid>) -> io::Result<Pid> {
    backend::process::syscalls::getpgid(pid)
}

/// `setpgid(pid, pgid)`—Sets the process group ID of the given process.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setpgid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/setpgid.2.html
#[inline]
pub fn setpgid(pid: Option<Pid>, pgid: Option<Pid>) -> io::Result<()> {
    backend::process::syscalls::setpgid(pid, pgid)
}

/// `getpgrp()`—Returns the process' group ID.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpgrp.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getpgrp.2.html
#[inline]
#[must_use]
pub fn getpgrp() -> Pid {
    backend::process::syscalls::getpgrp()
}

/// `getsid(pid)`—Get the session ID of the given process.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getsid.2.html
#[cfg(not(target_os = "redox"))]
#[inline]
pub fn getsid(pid: Option<Pid>) -> io::Result<Pid> {
    backend::process::syscalls::getsid(pid)
}

/// `setsid()`—Create a new session.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsid.html
/// [Linux]: https://man7.org/linux/man-pages/man2/setsid.2.html
#[inline]
pub fn setsid() -> io::Result<Pid> {
    backend::process::syscalls::setsid()
}

/// `getgroups()`—Return a list of the current user's groups.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getgroups.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getgroups.2.html
pub fn getgroups() -> io::Result<Vec<Gid>> {
    let mut buffer = Vec::new();

    // This code would benefit from having a better way to read into
    // uninitialized memory, but that requires `unsafe`.
    buffer.reserve(8);
    buffer.resize(buffer.capacity(), Gid::ROOT);

    loop {
        let ngroups = backend::process::syscalls::getgroups(&mut buffer)?;

        let ngroups = ngroups as usize;
        assert!(ngroups <= buffer.len());
        if ngroups < buffer.len() {
            buffer.resize(ngroups, Gid::ROOT);
            return Ok(buffer);
        }
        buffer.reserve(1); // use `Vec` reallocation strategy to grow capacity exponentially
        buffer.resize(buffer.capacity(), Gid::ROOT);
    }
}
