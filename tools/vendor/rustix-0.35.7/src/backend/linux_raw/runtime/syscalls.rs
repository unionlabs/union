//! linux_raw syscalls supporting `rustix::runtime`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
#[cfg(target_arch = "x86")]
use super::super::conv::by_mut;
use super::super::conv::{c_int, c_uint, ret, ret_c_uint, ret_error, ret_usize_infallible, zero};
use crate::fd::BorrowedFd;
use crate::ffi::CStr;
use crate::fs::AtFlags;
use crate::io;
use crate::process::{Pid, RawNonZeroPid};
use linux_raw_sys::general::{__kernel_pid_t, PR_SET_NAME, SIGCHLD};
#[cfg(target_arch = "x86_64")]
use {super::super::conv::ret_infallible, linux_raw_sys::general::ARCH_SET_FS};

#[inline]
pub(crate) unsafe fn fork() -> io::Result<Option<Pid>> {
    let pid = ret_c_uint(syscall_readonly!(
        __NR_clone,
        c_uint(SIGCHLD),
        zero(),
        zero(),
        zero(),
        zero()
    ))?;
    Ok(Pid::from_raw(pid))
}

pub(crate) unsafe fn execveat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    args: *const *const u8,
    env_vars: *const *const u8,
    flags: AtFlags,
) -> io::Errno {
    ret_error(syscall_readonly!(
        __NR_execveat,
        dirfd,
        path,
        args,
        env_vars,
        flags
    ))
}

pub(crate) unsafe fn execve(
    path: &CStr,
    args: *const *const u8,
    env_vars: *const *const u8,
) -> io::Errno {
    ret_error(syscall_readonly!(__NR_execve, path, args, env_vars))
}

pub(crate) mod tls {
    #[cfg(target_arch = "x86")]
    use super::super::tls::UserDesc;
    use super::*;

    #[cfg(target_arch = "x86")]
    #[inline]
    pub(crate) unsafe fn set_thread_area(u_info: &mut UserDesc) -> io::Result<()> {
        ret(syscall!(__NR_set_thread_area, by_mut(u_info)))
    }

    #[cfg(target_arch = "arm")]
    #[inline]
    pub(crate) unsafe fn arm_set_tls(data: *mut c::c_void) -> io::Result<()> {
        ret(syscall_readonly!(__ARM_NR_set_tls, data))
    }

    #[cfg(target_arch = "x86_64")]
    #[inline]
    pub(crate) unsafe fn set_fs(data: *mut c::c_void) {
        ret_infallible(syscall_readonly!(
            __NR_arch_prctl,
            c_uint(ARCH_SET_FS),
            data
        ))
    }

    #[inline]
    pub(crate) unsafe fn set_tid_address(data: *mut c::c_void) -> Pid {
        let tid: i32 =
            ret_usize_infallible(syscall_readonly!(__NR_set_tid_address, data)) as __kernel_pid_t;
        debug_assert_ne!(tid, 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(tid as u32))
    }

    #[inline]
    pub(crate) unsafe fn set_thread_name(name: &CStr) -> io::Result<()> {
        ret(syscall_readonly!(__NR_prctl, c_uint(PR_SET_NAME), name))
    }

    #[inline]
    pub(crate) fn exit_thread(code: c::c_int) -> ! {
        unsafe { syscall_noreturn!(__NR_exit, c_int(code)) }
    }
}
