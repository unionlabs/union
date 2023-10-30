//! libc syscalls supporting `rustix::termios`.
//!
//! # Safety
//!
//! See the `rustix::backend::syscalls` module documentation for details.

use super::super::c;
use super::super::conv::{borrowed_fd, ret, ret_pid_t};
use crate::fd::BorrowedFd;
#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
use crate::ffi::CStr;
use core::mem::MaybeUninit;
#[cfg(not(target_os = "wasi"))]
use {
    crate::io,
    crate::process::{Pid, RawNonZeroPid},
    crate::termios::{Action, OptionalActions, QueueSelector, Speed, Termios, Winsize},
};

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetattr(fd: BorrowedFd<'_>) -> io::Result<Termios> {
    let mut result = MaybeUninit::<Termios>::uninit();
    unsafe {
        ret(c::tcgetattr(borrowed_fd(fd), result.as_mut_ptr()))?;
        Ok(result.assume_init())
    }
}

#[cfg(all(
    linux_kernel,
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "riscv64",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "mips",
        target_arch = "mips64",
    )
))]
pub(crate) fn tcgetattr2(fd: BorrowedFd<'_>) -> io::Result<crate::termios::Termios2> {
    let mut result = MaybeUninit::<crate::termios::Termios2>::uninit();
    unsafe {
        ret(c::ioctl(borrowed_fd(fd), c::TCGETS2, result.as_mut_ptr()))?;
        Ok(result.assume_init())
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetpgrp(fd: BorrowedFd<'_>) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetpgrp(borrowed_fd(fd)))?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsetpgrp(fd: BorrowedFd<'_>, pid: Pid) -> io::Result<()> {
    unsafe { ret(c::tcsetpgrp(borrowed_fd(fd), pid.as_raw_nonzero().get())) }
}

#[cfg(not(target_os = "wasi"))]
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

#[cfg(all(
    linux_kernel,
    any(
        target_arch = "x86",
        target_arch = "x86_64",
        target_arch = "x32",
        target_arch = "riscv64",
        target_arch = "aarch64",
        target_arch = "arm",
        target_arch = "mips",
        target_arch = "mips64",
    )
))]
pub(crate) fn tcsetattr2(
    fd: BorrowedFd,
    optional_actions: OptionalActions,
    termios: &crate::termios::Termios2,
) -> io::Result<()> {
    unsafe {
        ret(c::ioctl(
            borrowed_fd(fd),
            (c::TCSETS2 as u32 + optional_actions as u32) as _,
            termios,
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsendbreak(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcsendbreak(borrowed_fd(fd), 0)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcdrain(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::tcdrain(borrowed_fd(fd))) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcflush(fd: BorrowedFd, queue_selector: QueueSelector) -> io::Result<()> {
    unsafe { ret(c::tcflush(borrowed_fd(fd), queue_selector as _)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcflow(fd: BorrowedFd, action: Action) -> io::Result<()> {
    unsafe { ret(c::tcflow(borrowed_fd(fd), action as _)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcgetsid(fd: BorrowedFd) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::tcgetsid(borrowed_fd(fd)))?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn tcsetwinsize(fd: BorrowedFd, winsize: Winsize) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCSWINSZ, &winsize)) }
}

#[cfg(not(target_os = "wasi"))]
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

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn cfgetospeed(termios: &Termios) -> Speed {
    unsafe { c::cfgetospeed(termios) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn cfgetispeed(termios: &Termios) -> Speed {
    unsafe { c::cfgetispeed(termios) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn cfmakeraw(termios: &mut Termios) {
    unsafe { c::cfmakeraw(termios) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn cfsetospeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetospeed(termios, speed)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn cfsetispeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetispeed(termios, speed)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn cfsetspeed(termios: &mut Termios, speed: Speed) -> io::Result<()> {
    unsafe { ret(c::cfsetspeed(termios, speed)) }
}

pub(crate) fn isatty(fd: BorrowedFd<'_>) -> bool {
    // Use the return value of `isatty` alone. We don't check `errno` because
    // we return `bool` rather than `io::Result<bool>`, because we assume
    // `BorrrowedFd` protects us from `EBADF`, and any other reasonably
    // anticipated `errno` value would end up interpreted as “assume it's not a
    // terminal” anyway.
    unsafe { c::isatty(borrowed_fd(fd)) != 0 }
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
