//! Libc call arguments and return values are often things like `c_int`,
//! `c_uint`, or libc-specific pointer types. This module provides functions
//! for converting between rustix's types and libc types.

#![allow(dead_code)]

use super::c;
use super::fd::{AsRawFd, BorrowedFd, FromRawFd, IntoRawFd, LibcFd, RawFd};
#[cfg(not(windows))]
use super::offset::libc_off_t;
#[cfg(not(windows))]
use crate::ffi::CStr;
use crate::io::{self, OwnedFd};
#[cfg(windows)]
use core::convert::TryInto;

#[cfg(not(windows))]
#[inline]
pub(super) fn c_str(c: &CStr) -> *const c::c_char {
    c.as_ptr()
}

#[cfg(not(windows))]
#[inline]
pub(super) fn no_fd() -> LibcFd {
    -1
}

#[inline]
pub(super) fn borrowed_fd(fd: BorrowedFd<'_>) -> LibcFd {
    fd.as_raw_fd() as LibcFd
}

#[inline]
pub(super) fn owned_fd(fd: OwnedFd) -> LibcFd {
    fd.into_raw_fd() as LibcFd
}

#[inline]
pub(super) fn ret(raw: c::c_int) -> io::Result<()> {
    if raw == 0 {
        Ok(())
    } else {
        Err(io::Errno::last_os_error())
    }
}

#[inline]
pub(super) fn syscall_ret(raw: c::c_long) -> io::Result<()> {
    if raw == 0 {
        Ok(())
    } else {
        Err(io::Errno::last_os_error())
    }
}

#[inline]
pub(super) fn nonnegative_ret(raw: c::c_int) -> io::Result<()> {
    if raw >= 0 {
        Ok(())
    } else {
        Err(io::Errno::last_os_error())
    }
}

#[inline]
pub(super) unsafe fn ret_infallible(raw: c::c_int) {
    debug_assert_eq!(raw, 0, "unexpected error: {:?}", io::Errno::last_os_error());
}

#[inline]
pub(super) fn ret_c_int(raw: c::c_int) -> io::Result<c::c_int> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw)
    }
}

#[inline]
pub(super) fn ret_u32(raw: c::c_int) -> io::Result<u32> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw as u32)
    }
}

#[inline]
pub(super) fn ret_ssize_t(raw: c::ssize_t) -> io::Result<c::ssize_t> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw)
    }
}

#[inline]
pub(super) fn syscall_ret_ssize_t(raw: c::c_long) -> io::Result<c::ssize_t> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw as c::ssize_t)
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(super) fn syscall_ret_u32(raw: c::c_long) -> io::Result<u32> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        let r32 = raw as u32;

        // Converting `raw` to `u32` should be lossless.
        debug_assert_eq!(r32 as c::c_long, raw);

        Ok(r32)
    }
}

#[cfg(not(windows))]
#[inline]
pub(super) fn ret_off_t(raw: libc_off_t) -> io::Result<libc_off_t> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw)
    }
}

#[cfg(not(windows))]
#[inline]
pub(super) fn ret_pid_t(raw: c::pid_t) -> io::Result<c::pid_t> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(raw)
    }
}

/// Convert a `c_int` returned from a libc function to an `OwnedFd`, if valid.
///
/// # Safety
///
/// The caller must ensure that this is the return value of a libc function
/// which returns an owned file descriptor.
#[inline]
pub(super) unsafe fn ret_owned_fd(raw: LibcFd) -> io::Result<OwnedFd> {
    if raw == !0 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(OwnedFd::from_raw_fd(raw as RawFd))
    }
}

#[inline]
pub(super) fn ret_discarded_fd(raw: LibcFd) -> io::Result<()> {
    if raw == !0 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(())
    }
}

#[inline]
pub(super) fn ret_discarded_char_ptr(raw: *mut c::c_char) -> io::Result<()> {
    if raw.is_null() {
        Err(io::Errno::last_os_error())
    } else {
        Ok(())
    }
}

/// Convert a `c_long` returned from `syscall` to an `OwnedFd`, if valid.
///
/// # Safety
///
/// The caller must ensure that this is the return value of a `syscall` call
/// which returns an owned file descriptor.
#[cfg(not(windows))]
#[inline]
pub(super) unsafe fn syscall_ret_owned_fd(raw: c::c_long) -> io::Result<OwnedFd> {
    if raw == -1 {
        Err(io::Errno::last_os_error())
    } else {
        Ok(OwnedFd::from_raw_fd(raw as RawFd))
    }
}

/// Convert the buffer-length argument value of a `send` or `recv` call.
#[cfg(not(windows))]
#[inline]
pub(super) fn send_recv_len(len: usize) -> usize {
    len
}

/// Convert the buffer-length argument value of a `send` or `recv` call.
#[cfg(windows)]
#[inline]
pub(super) fn send_recv_len(len: usize) -> i32 {
    // On Windows, the length argument has type `i32`; saturate the length,
    // since `send` and `recv` are allowed to send and recv less data than
    // requested.
    len.try_into().unwrap_or(i32::MAX)
}

/// Convert the return value of a `send` or `recv` call.
#[cfg(not(windows))]
#[inline]
pub(super) fn ret_send_recv(len: isize) -> io::Result<c::ssize_t> {
    ret_ssize_t(len)
}

/// Convert the return value of a `send` or `recv` call.
#[cfg(windows)]
#[inline]
pub(super) fn ret_send_recv(len: i32) -> io::Result<c::ssize_t> {
    ret_ssize_t(len as isize)
}
