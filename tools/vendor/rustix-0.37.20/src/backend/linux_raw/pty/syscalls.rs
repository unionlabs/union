//! linux_raw syscalls supporting `rustix::pty`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
use super::super::conv::{by_ref, c_uint, ret, ret_owned_fd};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::ffi::CString;
use crate::io;
use crate::path::DecInt;
use crate::pty::OpenptFlags;
#[cfg(any(apple, freebsdlike, linux_like, target_os = "fuchsia"))]
use alloc::vec::Vec;
use core::mem::MaybeUninit;
use linux_raw_sys::ioctl::{TIOCGPTN, TIOCGPTPEER, TIOCSPTLCK};

#[cfg(any(apple, freebsdlike, linux_like, target_os = "fuchsia"))]
#[inline]
pub(crate) fn ptsname(fd: BorrowedFd, mut buffer: Vec<u8>) -> io::Result<CString> {
    unsafe {
        let mut n = MaybeUninit::<c::c_int>::uninit();
        ret(syscall!(__NR_ioctl, fd, c_uint(TIOCGPTN), &mut n))?;

        buffer.clear();
        buffer.extend_from_slice(b"/dev/pts/");
        buffer.extend_from_slice(DecInt::new(n.assume_init()).as_bytes());
        // With Rust 1.58 we can append a '\0' ourselves and use
        // `from_vec_with_nul_unchecked`.
        Ok(CString::from_vec_unchecked(buffer))
    }
}

#[inline]
pub(crate) fn unlockpt(fd: BorrowedFd) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_ioctl,
            fd,
            c_uint(TIOCSPTLCK),
            by_ref(&0)
        ))
    }
}

#[cfg(target_os = "linux")]
#[inline]
pub(crate) fn ioctl_tiocgptpeer(fd: BorrowedFd, flags: OpenptFlags) -> io::Result<OwnedFd> {
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_ioctl,
            fd,
            c_uint(TIOCGPTPEER),
            c_uint(flags.bits())
        ))
    }
}
