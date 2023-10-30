//! linux_raw syscalls supporting `rustix::thread`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use crate::backend::c;
use crate::backend::conv::{
    by_mut, by_ref, c_int, c_uint, ret, ret_c_int, ret_c_int_infallible, ret_usize,
    slice_just_addr, slice_just_addr_mut, zero,
};
use crate::fd::BorrowedFd;
use crate::io;
use crate::pid::Pid;
use crate::thread::{ClockId, FutexFlags, FutexOperation, NanosleepRelativeResult, Timespec};
use core::mem::MaybeUninit;
#[cfg(target_pointer_width = "32")]
use linux_raw_sys::general::timespec as __kernel_old_timespec;
use linux_raw_sys::general::{__kernel_timespec, TIMER_ABSTIME};

#[inline]
pub(crate) fn clock_nanosleep_relative(
    id: ClockId,
    req: &__kernel_timespec,
) -> NanosleepRelativeResult {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        let mut rem = MaybeUninit::<__kernel_timespec>::uninit();
        match ret(syscall!(
            __NR_clock_nanosleep_time64,
            id,
            c_int(0),
            by_ref(req),
            &mut rem
        ))
        .or_else(|err| {
            // See the comments in `rustix_clock_gettime_via_syscall` about
            // emulation.
            if err == io::Errno::NOSYS {
                clock_nanosleep_relative_old(id, req, &mut rem)
            } else {
                Err(err)
            }
        }) {
            Ok(()) => NanosleepRelativeResult::Ok,
            Err(io::Errno::INTR) => NanosleepRelativeResult::Interrupted(rem.assume_init()),
            Err(err) => NanosleepRelativeResult::Err(err),
        }
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        let mut rem = MaybeUninit::<__kernel_timespec>::uninit();
        match ret(syscall!(
            __NR_clock_nanosleep,
            id,
            c_int(0),
            by_ref(req),
            &mut rem
        )) {
            Ok(()) => NanosleepRelativeResult::Ok,
            Err(io::Errno::INTR) => NanosleepRelativeResult::Interrupted(rem.assume_init()),
            Err(err) => NanosleepRelativeResult::Err(err),
        }
    }
}

#[cfg(target_pointer_width = "32")]
unsafe fn clock_nanosleep_relative_old(
    id: ClockId,
    req: &__kernel_timespec,
    rem: &mut MaybeUninit<__kernel_timespec>,
) -> io::Result<()> {
    let old_req = __kernel_old_timespec {
        tv_sec: req.tv_sec.try_into().map_err(|_| io::Errno::INVAL)?,
        tv_nsec: req.tv_nsec.try_into().map_err(|_| io::Errno::INVAL)?,
    };
    let mut old_rem = MaybeUninit::<__kernel_old_timespec>::uninit();
    ret(syscall!(
        __NR_clock_nanosleep,
        id,
        c_int(0),
        by_ref(&old_req),
        &mut old_rem
    ))?;
    let old_rem = old_rem.assume_init();
    rem.write(__kernel_timespec {
        tv_sec: old_rem.tv_sec.into(),
        tv_nsec: old_rem.tv_nsec.into(),
    });
    Ok(())
}

#[inline]
pub(crate) fn clock_nanosleep_absolute(id: ClockId, req: &__kernel_timespec) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(
            __NR_clock_nanosleep_time64,
            id,
            c_uint(TIMER_ABSTIME),
            by_ref(req),
            zero()
        ))
        .or_else(|err| {
            // See the comments in `rustix_clock_gettime_via_syscall` about
            // emulation.
            if err == io::Errno::NOSYS {
                clock_nanosleep_absolute_old(id, req)
            } else {
                Err(err)
            }
        })
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_clock_nanosleep,
            id,
            c_uint(TIMER_ABSTIME),
            by_ref(req),
            zero()
        ))
    }
}

#[cfg(target_pointer_width = "32")]
unsafe fn clock_nanosleep_absolute_old(id: ClockId, req: &__kernel_timespec) -> io::Result<()> {
    let old_req = __kernel_old_timespec {
        tv_sec: req.tv_sec.try_into().map_err(|_| io::Errno::INVAL)?,
        tv_nsec: req.tv_nsec.try_into().map_err(|_| io::Errno::INVAL)?,
    };
    ret(syscall_readonly!(
        __NR_clock_nanosleep,
        id,
        c_int(0),
        by_ref(&old_req),
        zero()
    ))
}

#[inline]
pub(crate) fn nanosleep(req: &__kernel_timespec) -> NanosleepRelativeResult {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        let mut rem = MaybeUninit::<__kernel_timespec>::uninit();
        match ret(syscall!(
            __NR_clock_nanosleep_time64,
            ClockId::Realtime,
            c_int(0),
            by_ref(req),
            &mut rem
        ))
        .or_else(|err| {
            // See the comments in `rustix_clock_gettime_via_syscall` about
            // emulation.
            if err == io::Errno::NOSYS {
                nanosleep_old(req, &mut rem)
            } else {
                Err(err)
            }
        }) {
            Ok(()) => NanosleepRelativeResult::Ok,
            Err(io::Errno::INTR) => NanosleepRelativeResult::Interrupted(rem.assume_init()),
            Err(err) => NanosleepRelativeResult::Err(err),
        }
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        let mut rem = MaybeUninit::<__kernel_timespec>::uninit();
        match ret(syscall!(__NR_nanosleep, by_ref(req), &mut rem)) {
            Ok(()) => NanosleepRelativeResult::Ok,
            Err(io::Errno::INTR) => NanosleepRelativeResult::Interrupted(rem.assume_init()),
            Err(err) => NanosleepRelativeResult::Err(err),
        }
    }
}

#[cfg(target_pointer_width = "32")]
unsafe fn nanosleep_old(
    req: &__kernel_timespec,
    rem: &mut MaybeUninit<__kernel_timespec>,
) -> io::Result<()> {
    let old_req = __kernel_old_timespec {
        tv_sec: req.tv_sec.try_into().map_err(|_| io::Errno::INVAL)?,
        tv_nsec: req.tv_nsec.try_into().map_err(|_| io::Errno::INVAL)?,
    };
    let mut old_rem = MaybeUninit::<__kernel_old_timespec>::uninit();
    ret(syscall!(__NR_nanosleep, by_ref(&old_req), &mut old_rem))?;
    let old_rem = old_rem.assume_init();
    rem.write(__kernel_timespec {
        tv_sec: old_rem.tv_sec.into(),
        tv_nsec: old_rem.tv_nsec.into(),
    });
    Ok(())
}

#[inline]
pub(crate) fn gettid() -> Pid {
    unsafe {
        let tid = ret_c_int_infallible(syscall_readonly!(__NR_gettid));
        Pid::from_raw_unchecked(tid)
    }
}

// TODO: This could be de-multiplexed.
#[inline]
pub(crate) unsafe fn futex(
    uaddr: *mut u32,
    op: FutexOperation,
    flags: FutexFlags,
    val: u32,
    utime: *const Timespec,
    uaddr2: *mut u32,
    val3: u32,
) -> io::Result<usize> {
    #[cfg(target_pointer_width = "32")]
    {
        ret_usize(syscall!(
            __NR_futex_time64,
            uaddr,
            (op, flags),
            c_uint(val),
            utime,
            uaddr2,
            c_uint(val3)
        ))
        .or_else(|err| {
            // See the comments in `rustix_clock_gettime_via_syscall` about
            // emulation.
            if err == io::Errno::NOSYS {
                futex_old(uaddr, op, flags, val, utime, uaddr2, val3)
            } else {
                Err(err)
            }
        })
    }
    #[cfg(target_pointer_width = "64")]
    ret_usize(syscall!(
        __NR_futex,
        uaddr,
        (op, flags),
        c_uint(val),
        utime,
        uaddr2,
        c_uint(val3)
    ))
}

#[cfg(target_pointer_width = "32")]
unsafe fn futex_old(
    uaddr: *mut u32,
    op: FutexOperation,
    flags: FutexFlags,
    val: u32,
    utime: *const Timespec,
    uaddr2: *mut u32,
    val3: u32,
) -> io::Result<usize> {
    let old_utime = __kernel_old_timespec {
        tv_sec: (*utime).tv_sec.try_into().map_err(|_| io::Errno::INVAL)?,
        tv_nsec: (*utime).tv_nsec.try_into().map_err(|_| io::Errno::INVAL)?,
    };
    ret_usize(syscall!(
        __NR_futex,
        uaddr,
        (op, flags),
        c_uint(val),
        by_ref(&old_utime),
        uaddr2,
        c_uint(val3)
    ))
}

#[inline]
pub(crate) fn setns(fd: BorrowedFd, nstype: c::c_int) -> io::Result<c::c_int> {
    unsafe { ret_c_int(syscall_readonly!(__NR_setns, fd, c_int(nstype))) }
}

#[inline]
pub(crate) fn unshare(flags: crate::thread::UnshareFlags) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_unshare, flags)) }
}

#[inline]
pub(crate) fn capget(
    header: &mut linux_raw_sys::general::__user_cap_header_struct,
    data: &mut [MaybeUninit<linux_raw_sys::general::__user_cap_data_struct>],
) -> io::Result<()> {
    unsafe {
        ret(syscall!(
            __NR_capget,
            by_mut(header),
            slice_just_addr_mut(data)
        ))
    }
}

#[inline]
pub(crate) fn capset(
    header: &mut linux_raw_sys::general::__user_cap_header_struct,
    data: &[linux_raw_sys::general::__user_cap_data_struct],
) -> io::Result<()> {
    unsafe { ret(syscall!(__NR_capset, by_mut(header), slice_just_addr(data))) }
}

#[inline]
pub(crate) fn setuid_thread(uid: crate::ugid::Uid) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_setuid, uid)) }
}

#[inline]
pub(crate) fn setresuid_thread(
    ruid: crate::ugid::Uid,
    euid: crate::ugid::Uid,
    suid: crate::ugid::Uid,
) -> io::Result<()> {
    #[cfg(any(target_arch = "x86", target_arch = "arm", target_arch = "sparc"))]
    unsafe {
        ret(syscall_readonly!(__NR_setresuid32, ruid, euid, suid))
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "arm", target_arch = "sparc")))]
    unsafe {
        ret(syscall_readonly!(__NR_setresuid, ruid, euid, suid))
    }
}

#[inline]
pub(crate) fn setgid_thread(gid: crate::ugid::Gid) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_setgid, gid)) }
}

#[inline]
pub(crate) fn setresgid_thread(
    rgid: crate::ugid::Gid,
    egid: crate::ugid::Gid,
    sgid: crate::ugid::Gid,
) -> io::Result<()> {
    #[cfg(any(target_arch = "x86", target_arch = "arm", target_arch = "sparc"))]
    unsafe {
        ret(syscall_readonly!(__NR_setresgid32, rgid, egid, sgid))
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "arm", target_arch = "sparc")))]
    unsafe {
        ret(syscall_readonly!(__NR_setresgid, rgid, egid, sgid))
    }
}
