// Copyright 2021 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for Nintendo 3DS
use crate::util_libc::sys_fill_exact;
use crate::Error;
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    sys_fill_exact(dest, |buf| unsafe {
        libc::getrandom(buf.as_mut_ptr() as *mut libc::c_void, buf.len(), 0)
    })
}
