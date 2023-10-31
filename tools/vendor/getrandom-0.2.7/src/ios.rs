// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for iOS
use crate::Error;
use core::{ffi::c_void, ptr::null};

#[link(name = "Security", kind = "framework")]
extern "C" {
    fn SecRandomCopyBytes(rnd: *const c_void, count: usize, bytes: *mut u8) -> i32;
}

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // Apple's documentation guarantees kSecRandomDefault is a synonym for NULL.
    let ret = unsafe { SecRandomCopyBytes(null(), dest.len(), dest.as_mut_ptr()) };
    // errSecSuccess (from SecBase.h) is always zero.
    if ret != 0 {
        Err(Error::IOS_SEC_RANDOM)
    } else {
        Ok(())
    }
}
