use super::super::c;
#[cfg(not(target_os = "wasi"))]
use bitflags::bitflags;

#[cfg(any(target_os = "android", target_os = "linux"))]
bitflags! {
    /// `RWF_*` constants for use with [`preadv2`] and [`pwritev2`].
    ///
    /// [`preadv2`]: crate::io::preadv2
    /// [`pwritev2`]: crate::io::pwritev
    pub struct ReadWriteFlags: c::c_int {
        /// `RWF_DSYNC` (since Linux 4.7)
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const DSYNC = c::RWF_DSYNC;
        /// `RWF_HIPRI` (since Linux 4.6)
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const HIPRI = c::RWF_HIPRI;
        /// `RWF_SYNC` (since Linux 4.7)
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const SYNC = c::RWF_SYNC;
        /// `RWF_NOWAIT` (since Linux 4.14)
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const NOWAIT = c::RWF_NOWAIT;
        /// `RWF_APPEND` (since Linux 4.16)
        #[cfg(all(target_os = "linux", target_env = "gnu"))]
        const APPEND = c::RWF_APPEND;
    }
}

#[cfg(not(target_os = "wasi"))]
bitflags! {
    /// `O_*` constants for use with [`dup2`].
    ///
    /// [`dup2`]: crate::io::dup2
    pub struct DupFlags: c::c_int {
        /// `O_CLOEXEC`
        #[cfg(not(any(
            target_os = "android",
            target_os = "ios",
            target_os = "macos",
            target_os = "redox",
        )))] // Android 5.0 has dup3, but libc doesn't have bindings
        const CLOEXEC = c::O_CLOEXEC;
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "wasi")))]
bitflags! {
    /// `O_*` constants for use with [`pipe_with`].
    ///
    /// [`pipe_with`]: crate::io::pipe_with
    pub struct PipeFlags: c::c_int {
        /// `O_CLOEXEC`
        const CLOEXEC = c::O_CLOEXEC;
        /// `O_DIRECT`
        #[cfg(not(any(target_os = "illumos", target_os = "openbsd", target_os = "redox")))]
        const DIRECT = c::O_DIRECT;
        /// `O_NONBLOCK`
        const NONBLOCK = c::O_NONBLOCK;
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
bitflags! {
    /// `EFD_*` flags for use with [`eventfd`].
    ///
    /// [`eventfd`]: crate::io::eventfd
    pub struct EventfdFlags: c::c_int {
        /// `EFD_CLOEXEC`
        const CLOEXEC = c::EFD_CLOEXEC;
        /// `EFD_NONBLOCK`
        const NONBLOCK = c::EFD_NONBLOCK;
        /// `EFD_SEMAPHORE`
        const SEMAPHORE = c::EFD_SEMAPHORE;
    }
}

/// `PIPE_BUF`—The maximum size of a write to a pipe guaranteed to be atomic.
#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi")))]
pub const PIPE_BUF: usize = c::PIPE_BUF;

#[cfg(not(any(windows, target_os = "redox")))]
pub(crate) const AT_FDCWD: c::c_int = c::AT_FDCWD;
#[cfg(not(windows))]
pub(crate) const STDIN_FILENO: c::c_int = c::STDIN_FILENO;
#[cfg(not(windows))]
pub(crate) const STDOUT_FILENO: c::c_int = c::STDOUT_FILENO;
#[cfg(not(windows))]
pub(crate) const STDERR_FILENO: c::c_int = c::STDERR_FILENO;
