use crate::fd::OwnedFd;
use crate::process::Pid;
use crate::{backend, io};

bitflags::bitflags! {
    /// `PIDFD_*` flags for use with [`pidfd_open`].
    ///
    /// [`pidfd_open`]: crate::process::pidfd_open
    pub struct PidfdFlags: backend::c::c_uint {
        /// `PIDFD_NONBLOCK`.
        const NONBLOCK = backend::c::PIDFD_NONBLOCK;
    }
}

/// `syscall(SYS_pidfd_open, pid, flags)`—Creates a file descriptor for
/// a process.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pidfd_open.2.html
#[inline]
pub fn pidfd_open(pid: Pid, flags: PidfdFlags) -> io::Result<OwnedFd> {
    backend::process::syscalls::pidfd_open(pid, flags)
}
