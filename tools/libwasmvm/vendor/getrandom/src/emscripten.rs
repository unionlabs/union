//! Implementation for Emscripten
use crate::{util_libc::last_os_error, Error};
use core::mem::MaybeUninit;

pub fn getrandom_inner(dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    // Emscripten 2.0.5 added getentropy, so we can use it unconditionally.
    // Unlike other getentropy implementations, there is no max buffer length.
    let ret = unsafe { libc::getentropy(dest.as_mut_ptr() as *mut libc::c_void, dest.len()) };
    if ret < 0 {
        return Err(last_os_error());
    }
    Ok(())
}
