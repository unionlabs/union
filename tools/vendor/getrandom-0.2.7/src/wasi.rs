// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASI
use crate::Error;
use core::num::NonZeroU32;
use wasi::wasi_snapshot_preview1::random_get;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    match unsafe { random_get(dest.as_mut_ptr() as i32, dest.len() as i32) } {
        0 => Ok(()),
        err => Err(unsafe { NonZeroU32::new_unchecked(err as u32) }.into()),
    }
}
