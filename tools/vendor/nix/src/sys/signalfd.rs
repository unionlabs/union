//! Interface for the `signalfd` syscall.
//!
//! # Signal discarding
//! When a signal can't be delivered to a process (or thread), it will become a pending signal.
//! Failure to deliver could happen if the signal is blocked by every thread in the process or if
//! the signal handler is still handling a previous signal.
//!
//! If a signal is sent to a process (or thread) that already has a pending signal of the same
//! type, it will be discarded. This means that if signals of the same type are received faster than
//! they are processed, some of those signals will be dropped. Because of this limitation,
//! `signalfd` in itself cannot be used for reliable communication between processes or threads.
//!
//! Once the signal is unblocked, or the signal handler is finished, and a signal is still pending
//! (ie. not consumed from a signalfd) it will be delivered to the signal handler.
//!
//! Please note that signal discarding is not specific to `signalfd`, but also happens with regular
//! signal handlers.
use crate::unistd;
use crate::Result;
use crate::errno::Errno;
pub use crate::sys::signal::{self, SigSet};
pub use libc::signalfd_siginfo as siginfo;

use std::os::unix::io::{RawFd, AsRawFd};
use std::mem;


libc_bitflags!{
    pub struct SfdFlags: libc::c_int {
        SFD_NONBLOCK;
        SFD_CLOEXEC;
    }
}

pub const SIGNALFD_NEW: RawFd = -1;
#[deprecated(since = "0.23.0", note = "use mem::size_of::<siginfo>() instead")]
pub const SIGNALFD_SIGINFO_SIZE: usize = mem::size_of::<siginfo>();

/// Creates a new file descriptor for reading signals.
///
/// **Important:** please read the module level documentation about signal discarding before using
/// this function!
///
/// The `mask` parameter specifies the set of signals that can be accepted via this file descriptor.
///
/// A signal must be blocked on every thread in a process, otherwise it won't be visible from
/// signalfd (the default handler will be invoked instead).
///
/// See [the signalfd man page for more information](https://man7.org/linux/man-pages/man2/signalfd.2.html)
pub fn signalfd(fd: RawFd, mask: &SigSet, flags: SfdFlags) -> Result<RawFd> {
    unsafe {
        Errno::result(libc::signalfd(fd as libc::c_int, mask.as_ref(), flags.bits()))
    }
}

/// A helper struct for creating, reading and closing a `signalfd` instance.
///
/// **Important:** please read the module level documentation about signal discarding before using
/// this struct!
///
/// # Examples
///
/// ```
/// # use nix::sys::signalfd::*;
/// // Set the thread to block the SIGUSR1 signal, otherwise the default handler will be used
/// let mut mask = SigSet::empty();
/// mask.add(signal::SIGUSR1);
/// mask.thread_block().unwrap();
///
/// // Signals are queued up on the file descriptor
/// let mut sfd = SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK).unwrap();
///
/// match sfd.read_signal() {
///     // we caught a signal
///     Ok(Some(sig)) => (),
///     // there were no signals waiting (only happens when the SFD_NONBLOCK flag is set,
///     // otherwise the read_signal call blocks)
///     Ok(None) => (),
///     Err(err) => (), // some error happend
/// }
/// ```
#[derive(Debug, Eq, Hash, PartialEq)]
pub struct SignalFd(RawFd);

impl SignalFd {
    pub fn new(mask: &SigSet) -> Result<SignalFd> {
        Self::with_flags(mask, SfdFlags::empty())
    }

    pub fn with_flags(mask: &SigSet, flags: SfdFlags) -> Result<SignalFd> {
        let fd = signalfd(SIGNALFD_NEW, mask, flags)?;

        Ok(SignalFd(fd))
    }

    pub fn set_mask(&mut self, mask: &SigSet) -> Result<()> {
        signalfd(self.0, mask, SfdFlags::empty()).map(drop)
    }

    pub fn read_signal(&mut self) -> Result<Option<siginfo>> {
        let mut buffer = mem::MaybeUninit::<siginfo>::uninit();

        let size = mem::size_of_val(&buffer);
        let res = Errno::result(unsafe {
            libc::read(self.0, buffer.as_mut_ptr() as *mut libc::c_void, size)
        }).map(|r| r as usize);
        match res {
            Ok(x) if x == size => Ok(Some(unsafe { buffer.assume_init() })),
            Ok(_) => unreachable!("partial read on signalfd"),
            Err(Errno::EAGAIN) => Ok(None),
            Err(error) => Err(error)
        }
    }
}

impl Drop for SignalFd {
    fn drop(&mut self) {
        let e = unistd::close(self.0);
        if !std::thread::panicking() && e == Err(Errno::EBADF) {
            panic!("Closing an invalid file descriptor!");
        };
    }
}

impl AsRawFd for SignalFd {
    fn as_raw_fd(&self) -> RawFd {
        self.0
    }
}

impl Iterator for SignalFd {
    type Item = siginfo;

    fn next(&mut self) -> Option<Self::Item> {
        match self.read_signal() {
            Ok(Some(sig)) => Some(sig),
            Ok(None) | Err(_) => None,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_signalfd() {
        let mask = SigSet::empty();
        let fd = SignalFd::new(&mask);
        assert!(fd.is_ok());
    }

    #[test]
    fn create_signalfd_with_opts() {
        let mask = SigSet::empty();
        let fd = SignalFd::with_flags(&mask, SfdFlags::SFD_CLOEXEC | SfdFlags::SFD_NONBLOCK);
        assert!(fd.is_ok());
    }

    #[test]
    fn read_empty_signalfd() {
        let mask = SigSet::empty();
        let mut fd = SignalFd::with_flags(&mask, SfdFlags::SFD_NONBLOCK).unwrap();

        let res = fd.read_signal();
        assert!(res.unwrap().is_none());
    }
}
