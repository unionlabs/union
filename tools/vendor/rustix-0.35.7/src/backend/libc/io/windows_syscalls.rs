//! Windows system calls in the `io` module.

use super::super::c;
use super::super::conv::{borrowed_fd, ret, ret_c_int};
use super::super::fd::LibcFd;
use crate::fd::{BorrowedFd, RawFd};
use crate::io;
use crate::io::PollFd;
use core::convert::TryInto;
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

pub(crate) fn poll(fds: &mut [PollFd<'_>], timeout: c::c_int) -> io::Result<usize> {
    let nfds = fds
        .len()
        .try_into()
        .map_err(|_convert_err| io::Errno::INVAL)?;

    ret_c_int(unsafe { c::poll(fds.as_mut_ptr().cast(), nfds, timeout) })
        .map(|nready| nready as usize)
}
