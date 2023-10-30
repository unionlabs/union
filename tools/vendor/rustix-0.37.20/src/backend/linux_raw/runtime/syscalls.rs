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
use super::super::conv::{
    by_ref, c_int, c_uint, ret, ret_c_int, ret_c_uint, ret_error, ret_usize_infallible, size_of,
    zero,
};
#[cfg(feature = "fs")]
use crate::fd::BorrowedFd;
use crate::ffi::CStr;
#[cfg(feature = "fs")]
use crate::fs::AtFlags;
use crate::io;
use crate::process::{Pid, RawNonZeroPid, Signal};
use crate::runtime::{How, Sigaction, Siginfo, Sigset, Stack, Timespec};
use crate::utils::optional_as_ptr;
#[cfg(target_pointer_width = "32")]
use core::convert::TryInto;
use core::mem::MaybeUninit;
#[cfg(target_pointer_width = "32")]
use linux_raw_sys::general::__kernel_old_timespec;
use linux_raw_sys::general::{__kernel_pid_t, kernel_sigset_t, PR_SET_NAME, SIGCHLD};
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

#[cfg(feature = "fs")]
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

#[inline]
pub(crate) unsafe fn sigaction(signal: Signal, new: Option<Sigaction>) -> io::Result<Sigaction> {
    let mut old = MaybeUninit::<Sigaction>::uninit();
    let new = optional_as_ptr(new.as_ref());
    ret(syscall!(
        __NR_rt_sigaction,
        signal,
        new,
        &mut old,
        size_of::<kernel_sigset_t, _>()
    ))?;
    Ok(old.assume_init())
}

#[inline]
pub(crate) unsafe fn sigaltstack(new: Option<Stack>) -> io::Result<Stack> {
    let mut old = MaybeUninit::<Stack>::uninit();
    let new = optional_as_ptr(new.as_ref());
    ret(syscall!(__NR_sigaltstack, new, &mut old))?;
    Ok(old.assume_init())
}

#[inline]
pub(crate) unsafe fn tkill(tid: Pid, sig: Signal) -> io::Result<()> {
    ret(syscall_readonly!(__NR_tkill, tid, sig))
}

#[inline]
pub(crate) unsafe fn sigprocmask(how: How, new: Option<&Sigset>) -> io::Result<Sigset> {
    let mut old = MaybeUninit::<Sigset>::uninit();
    let new = optional_as_ptr(new);
    ret(syscall!(
        __NR_rt_sigprocmask,
        how,
        new,
        &mut old,
        size_of::<kernel_sigset_t, _>()
    ))?;
    Ok(old.assume_init())
}

#[inline]
pub(crate) fn sigwait(set: &Sigset) -> io::Result<Signal> {
    unsafe {
        match Signal::from_raw(ret_c_int(syscall_readonly!(
            __NR_rt_sigtimedwait,
            by_ref(set),
            zero(),
            zero(),
            size_of::<kernel_sigset_t, _>()
        ))?) {
            Some(signum) => Ok(signum),
            None => Err(io::Errno::NOTSUP),
        }
    }
}

#[inline]
pub(crate) fn sigwaitinfo(set: &Sigset) -> io::Result<Siginfo> {
    let mut info = MaybeUninit::<Siginfo>::uninit();
    unsafe {
        let _signum = ret_c_int(syscall!(
            __NR_rt_sigtimedwait,
            by_ref(set),
            &mut info,
            zero(),
            size_of::<kernel_sigset_t, _>()
        ))?;
        Ok(info.assume_init())
    }
}

#[inline]
pub(crate) fn sigtimedwait(set: &Sigset, timeout: Option<Timespec>) -> io::Result<Siginfo> {
    let mut info = MaybeUninit::<Siginfo>::uninit();
    let timeout_ptr = optional_as_ptr(timeout.as_ref());

    // `rt_sigtimedwait_time64` was introduced in Linux 5.1. The old
    // `rt_sigtimedwait` syscall is not y2038-compatible on 32-bit
    // architectures.
    #[cfg(target_pointer_width = "32")]
    unsafe {
        match ret_c_int(syscall!(
            __NR_rt_sigtimedwait_time64,
            by_ref(set),
            &mut info,
            timeout_ptr,
            size_of::<kernel_sigset_t, _>()
        )) {
            Ok(_signum) => (),
            Err(io::Errno::NOSYS) => sigtimedwait_old(set, timeout, &mut info)?,
            Err(err) => return Err(err),
        }
        Ok(info.assume_init())
    }

    #[cfg(target_pointer_width = "64")]
    unsafe {
        let _signum = ret_c_int(syscall!(
            __NR_rt_sigtimedwait,
            by_ref(set),
            &mut info,
            timeout_ptr,
            size_of::<kernel_sigset_t, _>()
        ))?;
        Ok(info.assume_init())
    }
}

#[cfg(target_pointer_width = "32")]
unsafe fn sigtimedwait_old(
    set: &Sigset,
    timeout: Option<Timespec>,
    info: &mut MaybeUninit<Siginfo>,
) -> io::Result<()> {
    let old_timeout = match timeout {
        Some(timeout) => Some(__kernel_old_timespec {
            tv_sec: timeout.tv_sec.try_into().map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: timeout.tv_nsec as _,
        }),
        None => None,
    };

    let old_timeout_ptr = optional_as_ptr(old_timeout.as_ref());

    let _signum = ret_c_int(syscall!(
        __NR_rt_sigtimedwait,
        by_ref(set),
        info,
        old_timeout_ptr,
        size_of::<kernel_sigset_t, _>()
    ))?;

    Ok(())
}
