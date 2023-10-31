//! libc syscalls supporting `rustix::termios`.
//!
//! # Safety
//!
//! See the `rustix::backend::syscalls` module documentation for details.

#![allow(unsafe_code)]

use super::super::c;
use super::super::conv::{borrowed_fd, ret, ret_pid_t};
use crate::fd::BorrowedFd;
#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
use crate::ffi::CStr;
use crate::io;
use crate::process::{Pid, RawNonZeroPid};
use crate::termios::{Action, OptionalActions, QueueSelector, Speed, Termios, Winsize};
use core::mem::MaybeUninit;
use libc_errno::errno;

pub(crate) fn tcgetattr(fd: BorrowedFd<'_>) -> io::Result<Termios> {
    let mut result = MaybeUninit::<Termios>::uninit();
    unsafe {
        ret(c::tcgetattr(borrowed_fd(fd), result.as_mut_ptr())).map(|()| result.assume_init())
    }
}

pub(crate) fn tcgetpgrp(fd: BorrowedFd<'_>) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetpgrp(borrowed_fd(fd)))?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

pub(crate) fn tcsetpgrp(fd: BorrowedFd<'_>, pid: Pid) -> io::Result<()> {
    unsafe { ret(c::tcsetpgrp(borrowed_fd(fd), pid.as_raw_nonzero().get())) }
}

pub(crate) fn tcsetattr(
    fd: BorrowedFd,
    optional_actions: OptionalActions,
    termios: &Termios,
) -> io::Result<()> {
    unsafe {
        ret(c::tcsetattr(
            borrowed_fd(fd),
            optional_actions as _,
            termios,
        ))
    }
}

pub(crate) fn tcsendbreak(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcsendbreak(borrowed_fd(fd), 0)) }
}

pub(crate) fn tcdrain(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcdrain(borrowed_fd(fd))) }
}

pub(crate) fn tcflush(fd: BorrowedFd, queue_selector: QueueSelector) -> io::Result<()> {
    unsafe { ret(c::tcflush(borrowed_fd(fd), queue_selector as _)) }
}

pub(crate) fn tcflow(fd: BorrowedFd, action: Action) -> io::Result<()> {
    unsafe { ret(c::tcflow(borrowed_fd(fd), action as _)) }
}

pub(crate) fn tcgetsid(fd: BorrowedFd) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetsid(borrowed_fd(fd)))?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

pub(crate) fn tcsetwinsize(fd: BorrowedFd, winsize: Winsize) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCSWINSZ, &winsize)) }
}

pub(crate) fn tcgetwinsize(fd: BorrowedFd) -> io::Result<Winsize> {
    unsafe {
        let mut buf = MaybeUninit::<Winsize>::uninit();
        ret(c::ioctl(
            borrowed_fd(fd),
            c::TIOCGWINSZ.into(),
            buf.as_mut_ptr(),
        ))?;
        Ok(buf.assume_init())
    }
}

#[inline]
#[must_use]
pub(crate) fn cfgetospeed(termios: &Termios) -> Speed {
    unsafe { c::cfgetospeed(termios) }
}

#[inline]
#[must_use]
pub(crate) fn cfgetispeed(termios: &Termios) -> Speed {
    unsafe { c::cfgetispeed(termios) }
}

#[inline]
pub(crate) fn cfmakeraw(termios: &mut Termios) {
    unsafe { c::cfmakeraw(termios) }
}

#[inline]
pub(crate) fn cfsetospeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetospeed(termios, speed)) }
}

#[inline]
pub(crate) fn cfsetispeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetispeed(termios, speed)) }
}

#[inline]
pub(crate) fn cfsetspeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetspeed(termios, speed)) }
}

pub(crate) fn isatty(fd: BorrowedFd<'_>) -> bool {
    let res = unsafe { c::isatty(borrowed_fd(fd)) };
    if res == 0 {
        match errno().0 {
            #[cfg(not(any(target_os = "android", target_os = "linux")))]
            c::ENOTTY => false,

            // Old Linux versions reportedly return `EINVAL`.
            // <https://man7.org/linux/man-pages/man3/isatty.3.html#ERRORS>
            #[cfg(any(target_os = "android", target_os = "linux"))]
            c::ENOTTY | c::EINVAL => false,

            // Darwin mysteriously returns `EOPNOTSUPP` sometimes.
            #[cfg(any(target_os = "ios", target_os = "macos"))]
            c::EOPNOTSUPP => false,

            err => panic!("unexpected error from isatty: {:?}", err),
        }
    } else {
        true
    }
}

#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub(crate) fn ttyname(dirfd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    unsafe {
        // `ttyname_r` returns its error status rather than using `errno`.
        match c::ttyname_r(borrowed_fd(dirfd), buf.as_mut_ptr().cast(), buf.len()) {
            0 => Ok(CStr::from_ptr(buf.as_ptr().cast()).to_bytes().len()),
            err => Err(io::Errno::from_raw_os_error(err)),
        }
    }
}
