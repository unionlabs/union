// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for WASI
use crate::Error;
use core::{
    mem::MaybeUninit,
    num::{NonZeroU16, NonZeroU32},
};
use wasi::random_get;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    unsafe { random_get(dest.as_mut_ptr() as *mut u8, dest.len()) }.map_err(|e| {
        // The WASI errno will always be non-zero, but we check just in case.
        match NonZeroU16::new(e.raw()) {
            Some(r) => Error::from(NonZeroU32::from(r)),
            None => Error::ERRNO_NOT_POSITIVE,
        }
    })
}
