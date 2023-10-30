// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Implementation for the Solaris family
//!
//! Read from `/dev/random`, with chunks of limited size (256 bytes).
//! `/dev/random` uses the Hash_DRBG with SHA512 algorithm from NIST SP 800-90A.
//! `/dev/urandom` uses the FIPS 186-2 algorithm, which is considered less
//! secure. We choose to read from `/dev/random`.
//!
//! Since Solaris 11.3 and mid-2015 illumos, the `getrandom` syscall is available.
//! To make sure we can compile on both Solaris and its derivatives, as well as
//! function, we check for the existence of getrandom(2) in libc by calling
//! libc::dlsym.
use crate::{
    use_file,
    util_libc::{sys_fill_exact, Weak},
    Error,
};
use core::mem;

#[cfg(target_os = "illumos")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::ssize_t;
#[cfg(target_os = "solaris")]
type GetRandomFn = unsafe extern "C" fn(*mut u8, libc::size_t, libc::c_uint) -> libc::c_int;

pub fn getrandom_inner(dest: &mut [u8]) -> Result<(), Error> {
    // getrandom(2) was introduced in Solaris 11.3 for Illumos in 2015.
    static GETRANDOM: Weak = unsafe { Weak::new("getrandom\0") };
    if let Some(fptr) = GETRANDOM.ptr() {
        let func: GetRandomFn = unsafe { mem::transmute(fptr) };
        // 256 bytes is the lowest common denominator across all the Solaris
        // derived platforms for atomically obtaining random data.
        for chunk in dest.chunks_mut(256) {
            sys_fill_exact(chunk, |buf| unsafe {
                func(buf.as_mut_ptr(), buf.len(), 0) as libc::ssize_t
            })?
        }
        Ok(())
    } else {
        use_file::getrandom_inner(dest)
    }
}
