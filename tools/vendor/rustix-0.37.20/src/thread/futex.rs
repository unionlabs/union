//! Linux `futex`.
//!
//! # Safety
//!
//! Futex is a very low-level mechanism for implementing concurrency
//! primitives.
#![allow(unsafe_code)]

use crate::thread::Timespec;
use crate::{backend, io};

pub use backend::thread::{FutexFlags, FutexOperation};

/// `futex(uaddr, op, val, utime, uaddr2, val3)`
///
/// # References
///  - [Linux `futex` system call]
///  - [Linux `futex` feature]
///
/// # Safety
///
/// This is a very low-level feature for implementing synchronization
/// primitives. See the references links above.
///
/// [Linux `futex` system call]: https://man7.org/linux/man-pages/man2/futex.2.html
/// [Linux `futex` feature]: https://man7.org/linux/man-pages/man7/futex.7.html
#[inline]
pub unsafe fn futex(
    uaddr: *mut u32,
    op: FutexOperation,
    flags: FutexFlags,
    val: u32,
    utime: *const Timespec,
    uaddr2: *mut u32,
    val3: u32,
) -> io::Result<usize> {
    backend::thread::syscalls::futex(uaddr, op, flags, val, utime, uaddr2, val3)
}
