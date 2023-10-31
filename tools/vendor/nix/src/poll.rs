//! Wait for events to trigger on specific file descriptors
#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
use crate::sys::time::TimeSpec;
#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
use crate::sys::signal::SigSet;
use std::os::unix::io::{AsRawFd, RawFd};

use crate::Result;
use crate::errno::Errno;

/// This is a wrapper around `libc::pollfd`.
///
/// It's meant to be used as an argument to the [`poll`](fn.poll.html) and
/// [`ppoll`](fn.ppoll.html) functions to specify the events of interest
/// for a specific file descriptor.
///
/// After a call to `poll` or `ppoll`, the events that occured can be
/// retrieved by calling [`revents()`](#method.revents) on the `PollFd`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct PollFd {
    pollfd: libc::pollfd,
}

impl PollFd {
    /// Creates a new `PollFd` specifying the events of interest
    /// for a given file descriptor.
    pub const fn new(fd: RawFd, events: PollFlags) -> PollFd {
        PollFd {
            pollfd: libc::pollfd {
                fd,
                events: events.bits(),
                revents: PollFlags::empty().bits(),
            },
        }
    }

    /// Returns the events that occured in the last call to `poll` or `ppoll`.  Will only return
    /// `None` if the kernel provides status flags that Nix does not know about.
    pub fn revents(self) -> Option<PollFlags> {
        PollFlags::from_bits(self.pollfd.revents)
    }

    /// The events of interest for this `PollFd`.
    pub fn events(self) -> PollFlags {
        PollFlags::from_bits(self.pollfd.events).unwrap()
    }

    /// Modify the events of interest for this `PollFd`.
    pub fn set_events(&mut self, events: PollFlags) {
        self.pollfd.events = events.bits();
    }
}

impl AsRawFd for PollFd {
    fn as_raw_fd(&self) -> RawFd {
        self.pollfd.fd
    }
}

libc_bitflags! {
    /// These flags define the different events that can be monitored by `poll` and `ppoll`
    pub struct PollFlags: libc::c_short {
        /// There is data to read.
        POLLIN;
        /// There is some exceptional condition on the file descriptor.
        ///
        /// Possibilities include:
        ///
        /// *  There is out-of-band data on a TCP socket (see
        ///    [tcp(7)](https://man7.org/linux/man-pages/man7/tcp.7.html)).
        /// *  A pseudoterminal master in packet mode has seen a state
        ///    change on the slave (see
        ///    [ioctl_tty(2)](https://man7.org/linux/man-pages/man2/ioctl_tty.2.html)).
        /// *  A cgroup.events file has been modified (see
        ///    [cgroups(7)](https://man7.org/linux/man-pages/man7/cgroups.7.html)).
        POLLPRI;
        /// Writing is now possible, though a write larger that the
        /// available space in a socket or pipe will still block (unless
        /// `O_NONBLOCK` is set).
        POLLOUT;
        /// Equivalent to [`POLLIN`](constant.POLLIN.html)
        #[cfg(not(target_os = "redox"))]
        POLLRDNORM;
        #[cfg(not(target_os = "redox"))]
        /// Equivalent to [`POLLOUT`](constant.POLLOUT.html)
        POLLWRNORM;
        /// Priority band data can be read (generally unused on Linux).
        #[cfg(not(target_os = "redox"))]
        POLLRDBAND;
        /// Priority data may be written.
        #[cfg(not(target_os = "redox"))]
        POLLWRBAND;
        /// Error condition (only returned in
        /// [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        /// This bit is also set for a file descriptor referring to the
        /// write end of a pipe when the read end has been closed.
        POLLERR;
        /// Hang up (only returned in [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        /// Note that when reading from a channel such as a pipe or a stream
        /// socket, this event merely indicates that the peer closed its
        /// end of the channel.  Subsequent reads from the channel will
        /// return 0 (end of file) only after all outstanding data in the
        /// channel has been consumed.
        POLLHUP;
        /// Invalid request: `fd` not open (only returned in
        /// [`PollFd::revents`](struct.PollFd.html#method.revents);
        /// ignored in [`PollFd::new`](struct.PollFd.html#method.new)).
        POLLNVAL;
    }
}

/// `poll` waits for one of a set of file descriptors to become ready to perform I/O.
/// ([`poll(2)`](https://pubs.opengroup.org/onlinepubs/9699919799/functions/poll.html))
///
/// `fds` contains all [`PollFd`](struct.PollFd.html) to poll.
/// The function will return as soon as any event occur for any of these `PollFd`s.
///
/// The `timeout` argument specifies the number of milliseconds that `poll()`
/// should block waiting for a file descriptor to become ready.  The call
/// will block until either:
///
/// *  a file descriptor becomes ready;
/// *  the call is interrupted by a signal handler; or
/// *  the timeout expires.
///
/// Note that the timeout interval will be rounded up to the system clock
/// granularity, and kernel scheduling delays mean that the blocking
/// interval may overrun by a small amount.  Specifying a negative value
/// in timeout means an infinite timeout.  Specifying a timeout of zero
/// causes `poll()` to return immediately, even if no file descriptors are
/// ready.
pub fn poll(fds: &mut [PollFd], timeout: libc::c_int) -> Result<libc::c_int> {
    let res = unsafe {
        libc::poll(fds.as_mut_ptr() as *mut libc::pollfd,
                   fds.len() as libc::nfds_t,
                   timeout)
    };

    Errno::result(res)
}

/// `ppoll()` allows an application to safely wait until either a file
/// descriptor becomes ready or until a signal is caught.
/// ([`poll(2)`](https://man7.org/linux/man-pages/man2/poll.2.html))
///
/// `ppoll` behaves like `poll`, but let you specify what signals may interrupt it
/// with the `sigmask` argument. If you want `ppoll` to block indefinitely,
/// specify `None` as `timeout` (it is like `timeout = -1` for `poll`).
///
#[cfg(any(target_os = "android", target_os = "dragonfly", target_os = "freebsd", target_os = "linux"))]
pub fn ppoll(fds: &mut [PollFd], timeout: Option<TimeSpec>, sigmask: SigSet) -> Result<libc::c_int> {
    let timeout = timeout.as_ref().map_or(core::ptr::null(), |r| r.as_ref());
    let res = unsafe {
        libc::ppoll(fds.as_mut_ptr() as *mut libc::pollfd,
                    fds.len() as libc::nfds_t,
                    timeout,
                    sigmask.as_ref())
    };
    Errno::result(res)
}
