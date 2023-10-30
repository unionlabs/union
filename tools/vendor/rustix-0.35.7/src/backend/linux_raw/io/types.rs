use super::super::c;
use bitflags::bitflags;

bitflags! {
    /// `RWF_*` constants for use with [`preadv2`] and [`pwritev2`].
    ///
    /// [`preadv2`]: crate::io::preadv2
    /// [`pwritev2`]: crate::io::pwritev
    pub struct ReadWriteFlags: c::c_uint {
        /// `RWF_DSYNC` (since Linux 4.7)
        const DSYNC = linux_raw_sys::general::RWF_DSYNC;
        /// `RWF_HIPRI` (since Linux 4.6)
        const HIPRI = linux_raw_sys::general::RWF_HIPRI;
        /// `RWF_SYNC` (since Linux 4.7)
        const SYNC = linux_raw_sys::general::RWF_SYNC;
        /// `RWF_NOWAIT` (since Linux 4.14)
        const NOWAIT = linux_raw_sys::general::RWF_NOWAIT;
        /// `RWF_APPEND` (since Linux 4.16)
        const APPEND = linux_raw_sys::general::RWF_APPEND;
    }
}

bitflags! {
    /// `O_*` constants for use with [`dup2`].
    ///
    /// [`dup2`]: crate::io::dup2
    pub struct DupFlags: c::c_uint {
        /// `O_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::O_CLOEXEC;
    }
}

bitflags! {
    /// `O_*` constants for use with [`pipe_with`].
    ///
    /// [`pipe_with`]: crate::io::pipe_with
    pub struct PipeFlags: c::c_uint {
        /// `O_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::O_CLOEXEC;
        /// `O_DIRECT`
        const DIRECT = linux_raw_sys::general::O_DIRECT;
        /// `O_NONBLOCK`
        const NONBLOCK = linux_raw_sys::general::O_NONBLOCK;
    }
}

bitflags! {
    /// `EFD_*` flags for use with [`eventfd`].
    ///
    /// [`eventfd`]: crate::io::eventfd
    pub struct EventfdFlags: c::c_uint {
        /// `EFD_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::EFD_CLOEXEC;
        /// `EFD_NONBLOCK`
        const NONBLOCK = linux_raw_sys::general::EFD_NONBLOCK;
        /// `EFD_SEMAPHORE`
        const SEMAPHORE = linux_raw_sys::general::EFD_SEMAPHORE;
    }
}

/// `PIPE_BUF`—The maximum size of a write to a pipe guaranteed to be atomic.
pub const PIPE_BUF: usize = linux_raw_sys::general::PIPE_BUF as usize;

pub(crate) const AT_FDCWD: c::c_int = linux_raw_sys::general::AT_FDCWD;
pub(crate) const STDIN_FILENO: c::c_uint = linux_raw_sys::general::STDIN_FILENO;
pub(crate) const STDOUT_FILENO: c::c_uint = linux_raw_sys::general::STDOUT_FILENO;
pub(crate) const STDERR_FILENO: c::c_uint = linux_raw_sys::general::STDERR_FILENO;
