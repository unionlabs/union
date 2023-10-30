use super::super::c;
use super::super::conv::borrowed_fd;
#[cfg(windows)]
use super::super::fd::RawFd;
use super::super::fd::{AsFd, AsRawFd, BorrowedFd, LibcFd};
use bitflags::bitflags;
use core::marker::PhantomData;
#[cfg(windows)]
use std::fmt;

bitflags! {
    /// `POLL*` flags for use with [`poll`].
    ///
    /// [`poll`]: crate::io::poll
    pub struct PollFlags: c::c_short {
        /// `POLLIN`
        const IN = c::POLLIN;
        /// `POLLPRI`
        #[cfg(not(target_os = "wasi"))]
        const PRI = c::POLLPRI;
        /// `POLLOUT`
        const OUT = c::POLLOUT;
        /// `POLLRDNORM`
        const RDNORM = c::POLLRDNORM;
        /// `POLLWRNORM`
        const WRNORM = c::POLLWRNORM;
        /// `POLLRDBAND`
        #[cfg(not(target_os = "wasi"))]
        const RDBAND = c::POLLRDBAND;
        /// `POLLWRBAND`
        #[cfg(not(target_os = "wasi"))]
        const WRBAND = c::POLLWRBAND;
        /// `POLLERR`
        const ERR = c::POLLERR;
        /// `POLLHUP`
        const HUP = c::POLLHUP;
        /// `POLLNVAL`
        const NVAL = c::POLLNVAL;
        /// `POLLRDHUP`
        #[cfg(all(
            linux_kernel,
            not(any(target_arch = "sparc", target_arch = "sparc64"))),
        )]
        const RDHUP = c::POLLRDHUP;
    }
}

/// `struct pollfd`—File descriptor and flags for use with [`poll`].
///
/// [`poll`]: crate::io::poll
#[doc(alias = "pollfd")]
#[derive(Clone)]
#[cfg_attr(not(windows), derive(Debug))]
#[repr(transparent)]
pub struct PollFd<'fd> {
    pollfd: c::pollfd,
    _phantom: PhantomData<BorrowedFd<'fd>>,
}

#[cfg(windows)]
impl<'fd> fmt::Debug for PollFd<'fd> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("pollfd")
            .field("fd", &self.pollfd.fd)
            .field("events", &self.pollfd.events)
            .field("revents", &self.pollfd.revents)
            .finish()
    }
}

impl<'fd> PollFd<'fd> {
    /// Constructs a new `PollFd` holding `fd` and `events`.
    #[inline]
    pub fn new<Fd: AsFd>(fd: &'fd Fd, events: PollFlags) -> Self {
        Self::from_borrowed_fd(fd.as_fd(), events)
    }

    /// Sets the contained file descriptor to `fd`.
    #[inline]
    pub fn set_fd<Fd: AsFd>(&mut self, fd: &'fd Fd) {
        self.pollfd.fd = fd.as_fd().as_raw_fd() as LibcFd;
    }

    /// Clears the ready events.
    #[inline]
    pub fn clear_revents(&mut self) {
        self.pollfd.revents = 0;
    }

    /// Constructs a new `PollFd` holding `fd` and `events`.
    ///
    /// This is the same as `new`, but can be used to avoid borrowing the
    /// `BorrowedFd`, which can be tricky in situations where the `BorrowedFd`
    /// is a temporary.
    #[inline]
    pub fn from_borrowed_fd(fd: BorrowedFd<'fd>, events: PollFlags) -> Self {
        Self {
            pollfd: c::pollfd {
                fd: borrowed_fd(fd),
                events: events.bits(),
                revents: 0,
            },
            _phantom: PhantomData,
        }
    }

    /// Returns the ready events.
    #[inline]
    pub fn revents(&self) -> PollFlags {
        // Use `unwrap()` here because in theory we know we know all the bits
        // the OS might set here, but OS's have added extensions in the past.
        PollFlags::from_bits(self.pollfd.revents).unwrap()
    }
}

#[cfg(not(windows))]
impl<'fd> AsFd for PollFd<'fd> {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        // SAFETY: Our constructors and `set_fd` require `pollfd.fd` to be
        // valid for the `fd lifetime.
        unsafe { BorrowedFd::borrow_raw(self.pollfd.fd) }
    }
}

#[cfg(windows)]
impl<'fd> io_lifetimes::AsSocket for PollFd<'fd> {
    #[inline]
    fn as_socket(&self) -> BorrowedFd<'_> {
        // SAFETY: Our constructors and `set_fd` require `pollfd.fd` to be
        // valid for the `fd lifetime.
        unsafe { BorrowedFd::borrow_raw(self.pollfd.fd as RawFd) }
    }
}
