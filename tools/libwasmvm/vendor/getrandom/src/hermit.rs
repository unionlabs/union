use crate::Error;
use core::{cmp::min, mem::MaybeUninit, num::NonZeroU32};

extern "C" {
    fn sys_read_entropy(buffer: *mut u8, length: usize, flags: u32) -> isize;
}

pub fn getrandom_inner(mut dest: &mut [MaybeUninit<u8>]) -> Result<(), Error> {
    while !dest.is_empty() {
        let res = unsafe { sys_read_entropy(dest.as_mut_ptr() as *mut u8, dest.len(), 0) };
        if res < 0 {
            // SAFETY: all Hermit error codes use i32 under the hood:
            // https://github.com/hermitcore/libhermit-rs/blob/master/src/errno.rs
            let code = unsafe { NonZeroU32::new_unchecked((-res) as u32) };
            return Err(code.into());
        }
        let len = min(res as usize, dest.len());
        dest = &mut dest[len..];
    }
    Ok(())
}
