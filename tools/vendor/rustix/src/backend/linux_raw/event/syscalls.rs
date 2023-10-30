//! linux_raw syscalls supporting `rustix::event`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use crate::backend::c;
use crate::backend::conv::{
    by_ref, c_int, c_uint, pass_usize, raw_fd, ret, ret_owned_fd, ret_usize, slice_mut, zero,
};
use crate::event::{epoll, EventfdFlags, PollFd};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io;
use linux_raw_sys::general::{EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD};
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
use {
    crate::backend::conv::{opt_ref, size_of},
    linux_raw_sys::general::{__kernel_timespec, kernel_sigset_t},
};

#[inline]
pub(crate) fn poll(fds: &mut [PollFd<'_>], timeout: c::c_int) -> io::Result<usize> {
    let (fds_addr_mut, fds_len) = slice_mut(fds);

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    unsafe {
        let timeout = if timeout >= 0 {
            Some(__kernel_timespec {
                tv_sec: (timeout as i64) / 1000,
                tv_nsec: (timeout as i64) % 1000 * 1_000_000,
            })
        } else {
            None
        };
        ret_usize(syscall!(
            __NR_ppoll,
            fds_addr_mut,
            fds_len,
            opt_ref(timeout.as_ref()),
            zero(),
            size_of::<kernel_sigset_t, _>()
        ))
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        ret_usize(syscall!(__NR_poll, fds_addr_mut, fds_len, c_int(timeout)))
    }
}

#[inline]
pub(crate) fn epoll_create(flags: epoll::CreateFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_epoll_create1, flags)) }
}

#[inline]
pub(crate) unsafe fn epoll_add(
    epfd: BorrowedFd<'_>,
    fd: c::c_int,
    event: &epoll::Event,
) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_ADD),
        raw_fd(fd),
        by_ref(event)
    ))
}

#[inline]
pub(crate) unsafe fn epoll_mod(
    epfd: BorrowedFd<'_>,
    fd: c::c_int,
    event: &epoll::Event,
) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_MOD),
        raw_fd(fd),
        by_ref(event)
    ))
}

#[inline]
pub(crate) unsafe fn epoll_del(epfd: BorrowedFd<'_>, fd: c::c_int) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_DEL),
        raw_fd(fd),
        zero()
    ))
}

#[inline]
pub(crate) fn epoll_wait(
    epfd: BorrowedFd<'_>,
    events: *mut epoll::Event,
    num_events: usize,
    timeout: c::c_int,
) -> io::Result<usize> {
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        ret_usize(syscall!(
            __NR_epoll_wait,
            epfd,
            events,
            pass_usize(num_events),
            c_int(timeout)
        ))
    }
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    unsafe {
        ret_usize(syscall!(
            __NR_epoll_pwait,
            epfd,
            events,
            pass_usize(num_events),
            c_int(timeout),
            zero()
        ))
    }
}

#[inline]
pub(crate) fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_eventfd2, c_uint(initval), flags)) }
}
