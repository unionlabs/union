//! Windows system calls in the `io` module.

use crate::backend::c;
use crate::backend::conv::{borrowed_fd, ret};
use crate::backend::fd::LibcFd;
use crate::fd::{BorrowedFd, RawFd};
use crate::io;
use core::mem::MaybeUninit;

pub(crate) unsafe fn close(raw_fd: RawFd) {
    let _ = c::close(raw_fd as LibcFd);
}

pub(crate) fn ioctl_fionread(fd: BorrowedFd<'_>) -> io::Result<u64> {
    let mut nread = MaybeUninit::<c::c_ulong>::uninit();
    unsafe {
        ret(c::ioctl(borrowed_fd(fd), c::FIONREAD, nread.as_mut_ptr()))?;
        Ok(u64::from(nread.assume_init()))
    }
}

pub(crate) fn ioctl_fionbio(fd: BorrowedFd<'_>, value: bool) -> io::Result<()> {
    unsafe {
        let mut data = value as c::c_uint;
        ret(c::ioctl(borrowed_fd(fd), c::FIONBIO, &mut data))
    }
}
