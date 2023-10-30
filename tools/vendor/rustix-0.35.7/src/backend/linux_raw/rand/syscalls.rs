//! linux_raw syscalls supporting `rustix::rand`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::conv::{ret_usize, slice_mut};
use crate::io;
use crate::rand::GetRandomFlags;

#[inline]
pub(crate) fn getrandom(buf: &mut [u8], flags: GetRandomFlags) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);
    unsafe { ret_usize(syscall!(__NR_getrandom, buf_addr_mut, buf_len, flags)) }
}
