use crate::process::Pid;
use crate::{backend, io};
use bitflags::bitflags;

#[cfg(target_os = "linux")]
use crate::fd::BorrowedFd;

#[cfg(linux_raw)]
use crate::backend::process::wait::SiginfoExt;

bitflags! {
    /// Options for modifying the behavior of wait/waitpid
    pub struct WaitOptions: u32 {
        /// Return immediately if no child has exited.
        const NOHANG = backend::process::wait::WNOHANG as _;
        /// Return if a child has stopped (but not traced via [`ptrace`])
        ///
        /// [`ptrace`]: https://man7.org/linux/man-pages/man2/ptrace.2.html
        const UNTRACED = backend::process::wait::WUNTRACED as _;
        /// Return if a stopped child has been resumed by delivery of
        /// [`Signal::Cont`].
        const CONTINUED = backend::process::wait::WCONTINUED as _;
    }
}

#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
bitflags! {
    /// Options for modifying the behavior of waitid
    pub struct WaitidOptions: u32 {
        /// Return immediately if no child has exited.
        const NOHANG = backend::process::wait::WNOHANG as _;
        /// Return if a stopped child has been resumed by delivery of
        /// [`Signal::Cont`]
        const CONTINUED = backend::process::wait::WCONTINUED as _;
        /// Wait for processed that have exited.
        const EXITED = backend::process::wait::WEXITED as _;
        /// Keep processed in a waitable state.
        const NOWAIT = backend::process::wait::WNOWAIT as _;
        /// Wait for processes that have been stopped.
        const STOPPED = backend::process::wait::WSTOPPED as _;
    }
}

/// The status of a child process after calling [`wait`]/[`waitpid`].
#[derive(Debug, Clone, Copy)]
#[repr(transparent)]
pub struct WaitStatus(u32);

impl WaitStatus {
    /// Creates a `WaitStatus` out of an integer.
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

    /// Returns whether the process has exited normally.
    #[inline]
    pub fn exited(self) -> bool {
        backend::process::wait::WIFEXITED(self.0 as _)
    }

    /// Returns whether the process was terminated by a signal.
    #[inline]
    pub fn signaled(self) -> bool {
        backend::process::wait::WIFSIGNALED(self.0 as _)
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
        if self.exited() {
            Some(backend::process::wait::WEXITSTATUS(self.0 as _) as _)
        } else {
            None
        }
    }

    /// Returns the number of the signal that terminated the process,
    /// if the process was terminated by a signal.
    #[inline]
    pub fn terminating_signal(self) -> Option<u32> {
        if self.signaled() {
            Some(backend::process::wait::WTERMSIG(self.0 as _) as _)
        } else {
            None
        }
    }
}

/// The status of a process after calling [`waitid`].
#[derive(Clone, Copy)]
#[repr(transparent)]
#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
pub struct WaitidStatus(pub(crate) backend::c::siginfo_t);

#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
impl WaitidStatus {
    /// Returns whether the process is currently stopped.
    #[inline]
    pub fn stopped(&self) -> bool {
        self.si_code() == backend::c::CLD_STOPPED
    }

    /// Returns whether the process is currently trapped.
    #[inline]
    pub fn trapped(&self) -> bool {
        self.si_code() == backend::c::CLD_TRAPPED
    }

    /// Returns whether the process has exited normally.
    #[inline]
    pub fn exited(&self) -> bool {
        self.si_code() == backend::c::CLD_EXITED
    }

    /// Returns whether the process was terminated by a signal
    /// and did not create a core file.
    #[inline]
    pub fn killed(&self) -> bool {
        self.si_code() == backend::c::CLD_KILLED
    }

    /// Returns whether the process was terminated by a signal
    /// and did create a core file.
    #[inline]
    pub fn dumped(&self) -> bool {
        self.si_code() == backend::c::CLD_DUMPED
    }

    /// Returns whether the process has continued from a job control stop.
    #[inline]
    pub fn continued(&self) -> bool {
        self.si_code() == backend::c::CLD_CONTINUED
    }

    /// Returns the number of the signal that stopped the process,
    /// if the process was stopped by a signal.
    #[inline]
    #[cfg(not(any(target_os = "netbsd", target_os = "fuchsia", target_os = "emscripten")))]
    pub fn stopping_signal(&self) -> Option<u32> {
        if self.stopped() {
            Some(self.si_status() as _)
        } else {
            None
        }
    }

    /// Returns the number of the signal that trapped the process,
    /// if the process was trapped by a signal.
    #[inline]
    #[cfg(not(any(target_os = "netbsd", target_os = "fuchsia", target_os = "emscripten")))]
    pub fn trapping_signal(&self) -> Option<u32> {
        if self.trapped() {
            Some(self.si_status() as _)
        } else {
            None
        }
    }

    /// Returns the exit status number returned by the process,
    /// if it exited normally.
    #[inline]
    #[cfg(not(any(target_os = "netbsd", target_os = "fuchsia", target_os = "emscripten")))]
    pub fn exit_status(&self) -> Option<u32> {
        if self.exited() {
            Some(self.si_status() as _)
        } else {
            None
        }
    }

    /// Returns the number of the signal that terminated the process,
    /// if the process was terminated by a signal.
    #[inline]
    #[cfg(not(any(target_os = "netbsd", target_os = "fuchsia", target_os = "emscripten")))]
    pub fn terminating_signal(&self) -> Option<u32> {
        if self.killed() || self.dumped() {
            Some(self.si_status() as _)
        } else {
            None
        }
    }

    /// Returns a reference to the raw platform-specific `siginfo_t` struct.
    #[inline]
    pub const fn as_raw(&self) -> &backend::c::siginfo_t {
        &self.0
    }

    #[cfg(linux_raw)]
    fn si_code(&self) -> u32 {
        self.0.si_code() as u32 // CLD_ consts are unsigned
    }

    #[cfg(not(linux_raw))]
    fn si_code(&self) -> backend::c::c_int {
        self.0.si_code
    }

    #[cfg(not(any(target_os = "netbsd", target_os = "fuchsia", target_os = "emscripten")))]
    #[allow(unsafe_code)]
    fn si_status(&self) -> backend::c::c_int {
        // SAFETY: POSIX [specifies] that the `siginfo_t` returned by a
        // `waitid` call always has a valid `si_status` value.
        //
        // [specifies]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/signal.h.html
        unsafe { self.0.si_status() }
    }
}

/// The identifier to wait on in a call to [`waitid`].
#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum WaitId<'a> {
    /// Wait on all processes.
    All,

    /// Wait for a specific process ID.
    Pid(Pid),

    /// Wait for a specific process file descriptor.
    #[cfg(target_os = "linux")]
    PidFd(BorrowedFd<'a>),

    /// Eat the lifetime for non-Linux platforms.
    #[doc(hidden)]
    #[cfg(not(target_os = "linux"))]
    __EatLifetime(core::marker::PhantomData<&'a ()>),
    // TODO(notgull): Once this crate has the concept of PGIDs, add a WaitId::Pgid
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

/// `waitid(_, _, _, opts)`—Wait for the specified child process to change
/// state.
#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[inline]
pub fn waitid<'a>(
    id: impl Into<WaitId<'a>>,
    options: WaitidOptions,
) -> io::Result<Option<WaitidStatus>> {
    backend::process::syscalls::waitid(id.into(), options)
}
