// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Linux / Android
use crate::{
    util::LazyBool,
    util_libc::{last_os_error, sys_fill_exact},
    {use_file, Error},
};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // getrandom(2) was introduced in Linux 3.17
    static HAS_GETRANDOM: LazyBool = LazyBool::new();
    if HAS_GETRANDOM.unsync_init(is_getrandom_available) {
        sys_fill_exact(dest, |buf| unsafe {
            getrandom(buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
        })
    } else {
        use_file::getrandom_inner(dest)
    }
}

fn is_getrandom_available() -> bool {
    let res = unsafe { getrandom(core::ptr::null_mut(), 0, libc::GRND_NONBLOCK) };
    if res < 0 {
        match last_os_error().raw_os_error() {
            Some(libc::ENOSYS) => false, // No kernel support
            Some(libc::EPERM) => false,  // Blocked by seccomp
            _ => true,
        }
    } else {
        true
    }
}

unsafe fn getrandom(
    buf: *mut libc::c_void,
    buflen: libc::size_t,
    flags: libc::c_uint,
) -> libc::ssize_t {
    libc::syscall(libc::SYS_getrandom, buf, buflen, flags) as libc::ssize_t
}
