// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for OpenBSD
use crate::{util_libc::last_os_error, Error};

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // getentropy(2) was added in OpenBSD 5.6, so we can use it unconditionally.
    for chunk in dest.chunks_mut(256) {
        let ret = unsafe { libc::getentropy(chunk.as_mut_ptr() as *mut libc::c_void, chunk.len()) };
        if ret == -1 {
            return Err(last_os_error());
        }
    }
    Ok(())
}
