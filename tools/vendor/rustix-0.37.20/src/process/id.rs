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
pub use backend::process::types::RawUid;

/// The raw integer value of a Unix group ID.
pub use backend::process::types::RawGid;

/// The raw integer value of a Unix process ID.
pub use backend::process::types::RawPid;

/// The raw integer value of a Unix process ID.
pub use backend::process::types::RawNonZeroPid;

/// `uid_t`—A Unix user ID.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Uid(RawUid);

/// `gid_t`—A Unix group ID.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Gid(RawGid);

/// `pid_t`—A non-zero Unix process ID.
///
/// This is a pid, and not a pidfd. It is not a file descriptor, and the
/// process it refers to could disappear at any time and be replaced by
/// another, unrelated, process.
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Pid(RawNonZeroPid);

/// A Linux CPU ID.
#[cfg(linux_kernel)]
#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Hash)]
pub struct Cpuid(RawCpuid);

impl Uid {
    /// A `Uid` corresponding to the root user (uid 0).
    pub const ROOT: Self = Self(0);

    /// Converts a `RawUid` into a `Uid`.
    ///
    /// # Safety
    ///
    /// `raw` must be the value of a valid Unix user ID.
    #[inline]
    pub const unsafe fn from_raw(raw: RawUid) -> Self {
        Self(raw)
    }

    /// Converts a `Uid` into a `RawUid`.
    #[inline]
    pub const fn as_raw(self) -> RawUid {
        self.0
    }

    /// Test whether this uid represents the root user (uid 0).
    #[inline]
    pub const fn is_root(self) -> bool {
        self.0 == Self::ROOT.0
    }
}

impl Gid {
    /// A `Gid` corresponding to the root group (gid 0).
    pub const ROOT: Self = Self(0);

    /// Converts a `RawGid` into a `Gid`.
    ///
    /// # Safety
    ///
    /// `raw` must be the value of a valid Unix group ID.
    #[inline]
    pub const unsafe fn from_raw(raw: RawGid) -> Self {
        Self(raw)
    }

    /// Converts a `Gid` into a `RawGid`.
    #[inline]
    pub const fn as_raw(self) -> RawGid {
        self.0
    }

    /// Test whether this gid represents the root group (gid 0).
    #[inline]
    pub const fn is_root(self) -> bool {
        self.0 == Self::ROOT.0
    }
}

impl Pid {
    /// A `Pid` corresponding to the init process (pid 1).
    pub const INIT: Self = Self(
        // SAFETY: The init process' pid is always valid.
        unsafe { RawNonZeroPid::new_unchecked(1) },
    );

    /// Converts a `RawPid` into a `Pid`.
    ///
    /// # Safety
    ///
    /// `raw` must be the value of a valid Unix process ID, or zero.
    #[inline]
    pub const unsafe fn from_raw(raw: RawPid) -> Option<Self> {
        match RawNonZeroPid::new(raw) {
            Some(pid) => Some(Self(pid)),
            None => None,
        }
    }

    /// Converts a known non-zero `RawPid` into a `Pid`.
    ///
    /// # Safety
    ///
    /// `raw` must be the value of a valid Unix process ID. It must not be
    /// zero.
    #[inline]
    pub const unsafe fn from_raw_nonzero(raw: RawNonZeroPid) -> Self {
        Self(raw)
    }

    /// Creates a `Pid` holding the ID of the given child process.
    #[cfg(feature = "std")]
    #[inline]
    pub fn from_child(child: &std::process::Child) -> Self {
        let id = child.id();
        debug_assert_ne!(id, 0);

        // SAFETY: We know the returned ID is valid because it came directly
        // from an OS API.
        unsafe { Self::from_raw_nonzero(RawNonZeroPid::new_unchecked(id as _)) }
    }

    /// Converts a `Pid` into a `RawNonZeroPid`.
    #[inline]
    pub const fn as_raw_nonzero(self) -> RawNonZeroPid {
        self.0
    }

    /// Converts an `Option<Pid>` into a `RawPid`.
    #[inline]
    pub fn as_raw(pid: Option<Self>) -> RawPid {
        pid.map_or(0, |pid| pid.0.get())
    }

    /// Test whether this pid represents the init process (pid 0).
    #[inline]
    pub const fn is_init(self) -> bool {
        self.0.get() == Self::INIT.0.get()
    }
}

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
    backend::process::syscalls::getuid()
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
    backend::process::syscalls::geteuid()
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
    backend::process::syscalls::getgid()
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
    backend::process::syscalls::getegid()
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
    backend::process::syscalls::getpid()
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

// Return the raw value of the IDs. In case of `None` it returns `u32::MAX`
// since it has the same bit pattern as `-1` indicating no change to the
// owner/group ID.
pub(crate) fn translate_fchown_args(owner: Option<Uid>, group: Option<Gid>) -> (u32, u32) {
    let ow = match owner {
        Some(o) => o.as_raw(),
        None => u32::MAX,
    };

    let gr = match group {
        Some(g) => g.as_raw(),
        None => u32::MAX,
    };

    (ow, gr)
}
