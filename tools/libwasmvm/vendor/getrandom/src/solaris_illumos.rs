// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for the Solaris family
//!
//! `/dev/random` uses the Hash_DRBG with SHA512 algorithm from NIST SP 800-90A.
//! `/dev/urandom` uses the FIPS 186-2 algorithm, which is considered less
//! secure. We choose to read from `/dev/random` (and use GRND_RANDOM).
//!
//! Solaris 11.3 and late-2018 illumos added the getrandom(2) libc function.
//! To make sure we can compile on both Solaris and its derivatives, as well as
//! function, we check for the existence of getrandom(2) in libc by calling
//! libc::dlsym.
use crate::{
    use_file,
    util_libc::{sys_fill_exact, Weak},
    Error,
};
use core::mem::{self, MaybeUninit};

static GETRANDOM: Weak = unsafe { Weak::new("getrandom\0") };
type GetRandomFn =
    unsafe extern "C" fn(*mut libc::c_void, libc::size_t, libc::c_uint) -> libc::ssize_t;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    if let Some(fptr) = GETRANDOM.ptr() {
        let func: GetRandomFn = unsafe { mem::transmute(fptr) };
        // 256 bytes is the lowest common denominator across all the Solaris
        // derived platforms for atomically obtaining random data.
        for chunk in dest.chunks_mut(256) {
            sys_fill_exact(chunk, |buf| unsafe {
                // A cast is needed for the flags as libc uses the wrong type.
                func(
                    buf.as_mut_ptr() as *mut libc::c_void,
                    buf.len(),
                    libc::GRND_RANDOM as libc::c_uint,
                )
            })?
        }
        Ok(())
    } else {
        use_file::getrandom_inner(dest)
    }
}
