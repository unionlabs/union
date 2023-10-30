use crate::process::Pid;
use crate::{backend, io};
use bitflags::bitflags;

bitflags! {
    /// Options for modifying the behavior of wait/waitpid
    pub struct WaitOptions: u32 {
        /// Return immediately if no child has exited.
        const NOHANG = backend::process::wait::WNOHANG as _;
        /// Return if a child has stopped (but not traced via `ptrace(2)`)
        const UNTRACED = backend::process::wait::WUNTRACED as _;
        /// Return if a stopped child has been resumed by delivery of `SIGCONT`
        const CONTINUED = backend::process::wait::WCONTINUED as _;
    }
}

/// the status of the child processes the caller waited on
#[derive(Debug, Clone, Copy)]
pub struct WaitStatus(u32);

impl WaitStatus {
    /// create a `WaitStatus` out of an integer.
    #[inline]
    pub(crate) fn new(status: u32) -> Self {
        Self(status)
    }

    /// Converts a `WaitStatus` into its raw representation as an integer.
    #[inline]
    pub const fn as_raw(self) -> u32 {
        self.0
    }

    /// Returns whether the process is currently stopped.
    #[inline]
    pub fn stopped(self) -> bool {
        backend::process::wait::WIFSTOPPED(self.0 as _)
    }

    /// Returns whether the process has continued from a job control stop.
    #[inline]
    pub fn continued(self) -> bool {
        backend::process::wait::WIFCONTINUED(self.0 as _)
    }

    /// Returns the number of the signal that stopped the process,
    /// if the process was stopped by a signal.
    #[inline]
    pub fn stopping_signal(self) -> Option<u32> {
        if self.stopped() {
            Some(backend::process::wait::WSTOPSIG(self.0 as _) as _)
        } else {
            None
        }
    }

    /// Returns the exit status number returned by the process,
    /// if it exited normally.
    #[inline]
    pub fn exit_status(self) -> Option<u32> {
        if backend::process::wait::WIFEXITED(self.0 as _) {
            Some(backend::process::wait::WEXITSTATUS(self.0 as _) as _)
        } else {
            None
        }
    }

    /// Returns the number of the signal that terminated the process,
    /// if the process was terminated by a signal.
    #[inline]
    pub fn terminating_signal(self) -> Option<u32> {
        if backend::process::wait::WIFSIGNALED(self.0 as _) {
            Some(backend::process::wait::WTERMSIG(self.0 as _) as _)
        } else {
            None
        }
    }
}

/// `waitpid(pid, waitopts)`—Wait for a specific process to change state.
///
/// If the pid is `None`, the call will wait for any child process whose
/// process group id matches that of the calling process.
///
/// If the pid is equal to `RawPid::MAX`, the call will wait for any child
/// process.
///
/// Otherwise if the `wrapping_neg` of pid is less than pid, the call will wait
/// for any child process with a group ID equal to the `wrapping_neg` of `pid`.
///
/// Otherwise, the call will wait for the child process with the given pid.
///
/// On Success, returns the status of the selected process.
///
/// If `NOHANG` was specified in the options, and the selected child process
/// didn't change state, returns `None`.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/wait.html
/// [Linux]: https://man7.org/linux/man-pages/man2/waitpid.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn waitpid(pid: Option<Pid>, waitopts: WaitOptions) -> io::Result<Option<WaitStatus>> {
    Ok(backend::process::syscalls::waitpid(pid, waitopts)?.map(|(_, status)| status))
}

/// `wait(waitopts)`—Wait for any of the children of calling process to
/// change state.
///
/// On success, returns the pid of the child process whose state changed, and
/// the status of said process.
///
/// If `NOHANG` was specified in the options, and the selected child process
/// didn't change state, returns `None`.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/wait.html
/// [Linux]: https://man7.org/linux/man-pages/man2/waitpid.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn wait(waitopts: WaitOptions) -> io::Result<Option<(Pid, WaitStatus)>> {
    backend::process::syscalls::wait(waitopts)
}
