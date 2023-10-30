//! The libc backend.
//!
//! On most platforms, this uses the `libc` crate to make system calls. On
//! Windows, this uses the Winsock2 API in `windows-sys`, which can be adapted
//! to have a very `libc`-like interface.

// Every FFI call requires an unsafe block, and there are a lot of FFI
// calls. For now, set this to allow for the libc backend.
#![allow(clippy::undocumented_unsafe_blocks)]
// Lots of libc types vary between platforms, so we often need a `.into()` on
// one platform where it's redundant on another.
#![allow(clippy::useless_conversion)]

mod conv;
mod offset;

#[cfg(windows)]
mod io_lifetimes;
#[cfg(not(windows))]
#[cfg(not(feature = "std"))]
pub(crate) mod fd {
    pub(crate) use super::c::c_int as LibcFd;
    pub use crate::io::fd::*;
}
#[cfg(windows)]
pub(crate) mod fd {
    pub use super::io_lifetimes::*;
}
#[cfg(not(windows))]
#[cfg(feature = "std")]
pub(crate) mod fd {
    pub use io_lifetimes::*;

    #[cfg(target_os = "wasi")]
    #[allow(unused_imports)]
    pub(crate) use super::c::c_int as LibcFd;
    #[cfg(unix)]
    #[allow(unused_imports)]
    pub(crate) use std::os::unix::io::RawFd as LibcFd;
    #[cfg(unix)]
    pub use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
    #[cfg(target_os = "wasi")]
    pub use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd, RawFd};
}

// On Windows we emulate selected libc-compatible interfaces. On non-Windows,
// we just use libc here, since this is the libc backend.
#[cfg_attr(windows, path = "winsock_c.rs")]
pub(crate) mod c;

#[cfg(not(windows))]
#[cfg(feature = "fs")]
pub(crate) mod fs;
pub(crate) mod io;
#[cfg(linux_kernel)]
#[cfg(feature = "io_uring")]
pub(crate) mod io_uring;
#[cfg(not(any(windows, target_os = "wasi")))]
#[cfg(feature = "mm")]
pub(crate) mod mm;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[cfg(feature = "net")]
pub(crate) mod net;
#[cfg(not(windows))]
#[cfg(any(
    feature = "param",
    feature = "runtime",
    feature = "time",
    target_arch = "x86",
))]
pub(crate) mod param;
#[cfg(not(windows))]
pub(crate) mod process;
#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
#[cfg(feature = "pty")]
pub(crate) mod pty;
#[cfg(not(windows))]
#[cfg(feature = "rand")]
pub(crate) mod rand;
#[cfg(not(windows))]
#[cfg(feature = "termios")]
pub(crate) mod termios;
#[cfg(not(windows))]
#[cfg(feature = "thread")]
pub(crate) mod thread;
#[cfg(not(windows))]
pub(crate) mod time;

/// If the host libc is glibc, return `true` if it is less than version 2.25.
///
/// To restate and clarify, this function returning true does not mean the libc
/// is glibc just that if it is glibc, it is less than version 2.25.
///
/// For now, this function is only available on Linux, but if it ends up being
/// used beyond that, this could be changed to e.g. `#[cfg(unix)]`.
#[cfg(all(unix, target_env = "gnu"))]
pub(crate) fn if_glibc_is_less_than_2_25() -> bool {
    // This is also defined inside `weak_or_syscall!` in
    // backend/libc/rand/syscalls.rs, but it's not convenient to re-export the
    // weak symbol from that macro, so we duplicate it at a small cost here.
    weak! { fn getrandom(*mut c::c_void, c::size_t, c::c_uint) -> c::ssize_t }

    // glibc 2.25 has `getrandom`, which is how we satisfy the API contract of
    // this function. But, there are likely other libc versions which have it.
    getrandom.get().is_none()
}
