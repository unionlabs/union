//! libc syscalls supporting `rustix::io_uring`.

use super::super::c;
use super::super::conv::{borrowed_fd, syscall_ret_owned_fd, syscall_ret_u32};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io;
use crate::io_uring::{io_uring_params, IoringEnterFlags, IoringRegisterOp};
use linux_raw_sys::general::{__NR_io_uring_enter, __NR_io_uring_register, __NR_io_uring_setup};

#[inline]
pub(crate) fn io_uring_setup(entries: u32, params: &mut io_uring_params) -> io::Result<OwnedFd> {
    unsafe {
        syscall_ret_owned_fd(c::syscall(
            __NR_io_uring_setup as _,
            entries as usize,
            params,
        ))
    }
}

#[inline]
pub(crate) unsafe fn io_uring_register(
    fd: BorrowedFd<'_>,
    opcode: IoringRegisterOp,
    arg: *const c::c_void,
    nr_args: u32,
) -> io::Result<u32> {
    syscall_ret_u32(c::syscall(
        __NR_io_uring_register as _,
        borrowed_fd(fd),
        opcode as u32 as usize,
        arg,
        nr_args as usize,
    ))
}

#[inline]
pub(crate) unsafe fn io_uring_enter(
    fd: BorrowedFd<'_>,
    to_submit: u32,
    min_complete: u32,
    flags: IoringEnterFlags,
    arg: *const c::c_void,
    size: usize,
) -> io::Result<u32> {
    syscall_ret_u32(c::syscall(
        __NR_io_uring_enter as _,
        borrowed_fd(fd),
        to_submit as usize,
        min_complete as usize,
        flags.bits() as usize,
        arg,
        size,
    ))
}
