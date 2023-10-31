//! Implementation of `errno` functionality for Unix systems.
//!
//! Adapted from `src/libstd/sys/unix/os.rs` in the Rust distribution.

// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::str;
#[cfg(target_os = "dragonfly")]
use errno_dragonfly::errno_location;
use libc::{self, c_char, c_int, size_t, strlen};

use crate::Errno;

fn from_utf8_lossy(input: &[u8]) -> &str {
    match str::from_utf8(input) {
        Ok(valid) => valid,
        Err(error) => unsafe { str::from_utf8_unchecked(&input[..error.valid_up_to()]) },
    }
}

pub fn with_description<F, T>(err: Errno, callback: F) -> T
where
    F: FnOnce(Result<&str, Errno>) -> T,
{
    let mut buf = [0u8; 1024];
    let c_str = unsafe {
        if strerror_r(err.0, buf.as_mut_ptr() as *mut _, buf.len() as size_t) < 0 {
            let fm_err = errno();
            if fm_err != Errno(libc::ERANGE) {
                return callback(Err(fm_err));
            }
        }
        let c_str_len = strlen(buf.as_ptr() as *const _);
        &buf[..c_str_len]
    };
    callback(Ok(from_utf8_lossy(c_str)))
}

pub const STRERROR_NAME: &str = "strerror_r";

pub fn errno() -> Errno {
    unsafe { Errno(*errno_location()) }
}

pub fn set_errno(Errno(errno): Errno) {
    unsafe {
        *errno_location() = errno;
    }
}

extern "C" {
    #[cfg(not(target_os = "dragonfly"))]
    #[cfg_attr(
        any(target_os = "macos", target_os = "ios", target_os = "freebsd"),
        link_name = "__error"
    )]
    #[cfg_attr(
        any(
            target_os = "openbsd",
            target_os = "netbsd",
            target_os = "bitrig",
            target_os = "android"
        ),
        link_name = "__errno"
    )]
    #[cfg_attr(
        any(target_os = "solaris", target_os = "illumos"),
        link_name = "___errno"
    )]
    #[cfg_attr(target_os = "haiku", link_name = "_errnop")]
    #[cfg_attr(
        any(target_os = "linux", target_os = "redox"),
        link_name = "__errno_location"
    )]
    #[cfg_attr(target_os = "aix", link_name = "_Errno")]
    fn errno_location() -> *mut c_int;

    #[cfg_attr(target_os = "linux", link_name = "__xpg_strerror_r")]
    fn strerror_r(errnum: c_int, buf: *mut c_char, buflen: size_t) -> c_int;
}
