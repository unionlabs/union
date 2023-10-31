//! linux_raw syscalls supporting `rustix::process`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
use super::super::conv::{
    by_mut, by_ref, c_int, c_uint, negative_pid, pass_usize, ret, ret_c_int, ret_c_uint,
    ret_infallible, ret_usize, ret_usize_infallible, size_of, slice, slice_just_addr,
    slice_just_addr_mut, slice_mut, zero,
};
use super::types::{RawCpuSet, RawUname};
use crate::backend::conv::ret_owned_fd;
use crate::fd::{AsRawFd, BorrowedFd, OwnedFd};
use crate::ffi::CStr;
use crate::io;
use crate::process::{
    Cpuid, Gid, MembarrierCommand, MembarrierQuery, Pid, PidfdFlags, RawNonZeroPid, RawPid,
    Resource, Rlimit, Signal, Sysinfo, Uid, WaitId, WaitOptions, WaitStatus, WaitidOptions,
    WaitidStatus,
};
use crate::utils::as_mut_ptr;
use core::convert::TryInto;
use core::mem::MaybeUninit;
use core::num::NonZeroU32;
use core::ptr::{null, null_mut};
use linux_raw_sys::general::{
    __kernel_gid_t, __kernel_pid_t, __kernel_uid_t, membarrier_cmd, membarrier_cmd_flag, rlimit,
    rlimit64, PRIO_PGRP, PRIO_PROCESS, PRIO_USER, RLIM64_INFINITY, RLIM_INFINITY,
};
use linux_raw_sys::ioctl::TIOCSCTTY;
#[cfg(not(target_os = "wasi"))]
#[cfg(feature = "fs")]
use {super::super::conv::ret_c_uint_infallible, crate::fs::Mode};

#[inline]
pub(crate) fn chdir(filename: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_chdir, filename)) }
}

#[inline]
pub(crate) fn fchdir(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_fchdir, fd)) }
}

#[inline]
pub(crate) fn chroot(filename: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_chroot, filename)) }
}

#[inline]
pub(crate) fn getcwd(buf: &mut [u8]) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);
    unsafe { ret_usize(syscall!(__NR_getcwd, buf_addr_mut, buf_len)) }
}

#[inline]
pub(crate) fn membarrier_query() -> MembarrierQuery {
    unsafe {
        match ret_c_uint(syscall!(
            __NR_membarrier,
            c_int(membarrier_cmd::MEMBARRIER_CMD_QUERY as _),
            c_uint(0)
        )) {
            Ok(query) => {
                // SAFETY: The safety of `from_bits_unchecked` is discussed
                // [here]. Our "source of truth" is Linux, and here, the
                // `query` value is coming from Linux, so we know it only
                // contains "source of truth" valid bits.
                //
                // [here]: https://github.com/bitflags/bitflags/pull/207#issuecomment-671668662
                MembarrierQuery::from_bits_unchecked(query)
            }
            Err(_) => MembarrierQuery::empty(),
        }
    }
}

#[inline]
pub(crate) fn membarrier(cmd: MembarrierCommand) -> io::Result<()> {
    unsafe { ret(syscall!(__NR_membarrier, cmd, c_uint(0))) }
}

#[inline]
pub(crate) fn membarrier_cpu(cmd: MembarrierCommand, cpu: Cpuid) -> io::Result<()> {
    unsafe {
        ret(syscall!(
            __NR_membarrier,
            cmd,
            c_uint(membarrier_cmd_flag::MEMBARRIER_CMD_FLAG_CPU as _),
            cpu
        ))
    }
}

#[inline]
pub(crate) fn getpid() -> Pid {
    unsafe {
        let pid: i32 = ret_usize_infallible(syscall_readonly!(__NR_getpid)) as __kernel_pid_t;
        debug_assert!(pid > 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid as u32))
    }
}

#[inline]
pub(crate) fn getppid() -> Option<Pid> {
    unsafe {
        let ppid: i32 = ret_usize_infallible(syscall_readonly!(__NR_getppid)) as __kernel_pid_t;
        Pid::from_raw(ppid as u32)
    }
}

#[inline]
pub(crate) fn getpgid(pid: Option<Pid>) -> io::Result<Pid> {
    unsafe {
        let pgid: i32 =
            ret_usize(syscall_readonly!(__NR_getpgid, c_uint(Pid::as_raw(pid))))? as __kernel_pid_t;
        Ok(Pid::from_raw_nonzero(NonZeroU32::new_unchecked(
            pgid as u32,
        )))
    }
}

#[inline]
pub(crate) fn setpgid(pid: Option<Pid>, pgid: Option<Pid>) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_setpgid,
            c_uint(Pid::as_raw(pid)),
            c_uint(Pid::as_raw(pgid))
        ))
    }
}

#[inline]
pub(crate) fn getpgrp() -> Pid {
    // Use the `getpgrp` syscall if available.
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        let pgid: i32 = ret_usize_infallible(syscall_readonly!(__NR_getpgrp)) as __kernel_pid_t;
        debug_assert!(pgid > 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pgid as u32))
    }

    // Otherwise use `getpgrp` and pass it zero.
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    unsafe {
        let pgid: i32 =
            ret_usize_infallible(syscall_readonly!(__NR_getpgid, c_uint(0))) as __kernel_pid_t;
        debug_assert!(pgid > 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pgid as u32))
    }
}

#[inline]
pub(crate) fn getgid() -> Gid {
    #[cfg(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm"))]
    unsafe {
        let gid: i32 =
            (ret_usize_infallible(syscall_readonly!(__NR_getgid32)) as __kernel_gid_t).into();
        Gid::from_raw(gid as u32)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm")))]
    unsafe {
        let gid = ret_usize_infallible(syscall_readonly!(__NR_getgid)) as __kernel_gid_t;
        Gid::from_raw(gid as u32)
    }
}

#[inline]
pub(crate) fn getegid() -> Gid {
    #[cfg(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm"))]
    unsafe {
        let gid: i32 =
            (ret_usize_infallible(syscall_readonly!(__NR_getegid32)) as __kernel_gid_t).into();
        Gid::from_raw(gid as u32)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm")))]
    unsafe {
        let gid = ret_usize_infallible(syscall_readonly!(__NR_getegid)) as __kernel_gid_t;
        Gid::from_raw(gid as u32)
    }
}

#[inline]
pub(crate) fn getuid() -> Uid {
    #[cfg(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm"))]
    unsafe {
        let uid = (ret_usize_infallible(syscall_readonly!(__NR_getuid32)) as __kernel_uid_t).into();
        Uid::from_raw(uid)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm")))]
    unsafe {
        let uid = ret_usize_infallible(syscall_readonly!(__NR_getuid)) as __kernel_uid_t;
        Uid::from_raw(uid as u32)
    }
}

#[inline]
pub(crate) fn geteuid() -> Uid {
    #[cfg(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm"))]
    unsafe {
        let uid: i32 =
            (ret_usize_infallible(syscall_readonly!(__NR_geteuid32)) as __kernel_uid_t).into();
        Uid::from_raw(uid as u32)
    }
    #[cfg(not(any(target_arch = "x86", target_arch = "sparc", target_arch = "arm")))]
    unsafe {
        let uid = ret_usize_infallible(syscall_readonly!(__NR_geteuid)) as __kernel_uid_t;
        Uid::from_raw(uid as u32)
    }
}

#[inline]
pub(crate) fn sched_getaffinity(pid: Option<Pid>, cpuset: &mut RawCpuSet) -> io::Result<()> {
    unsafe {
        // The raw linux syscall returns the size (in bytes) of the `cpumask_t`
        // data type that is used internally by the kernel to represent the CPU
        // set bit mask.
        let size = ret_usize(syscall!(
            __NR_sched_getaffinity,
            c_uint(Pid::as_raw(pid)),
            size_of::<RawCpuSet, _>(),
            by_mut(&mut cpuset.bits)
        ))?;
        let bytes = as_mut_ptr(cpuset).cast::<u8>();
        let rest = bytes.wrapping_add(size);
        // Zero every byte in the cpuset not set by the kernel.
        rest.write_bytes(0, core::mem::size_of::<RawCpuSet>() - size);
        Ok(())
    }
}

#[inline]
pub(crate) fn sched_setaffinity(pid: Option<Pid>, cpuset: &RawCpuSet) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_sched_setaffinity,
            c_uint(Pid::as_raw(pid)),
            size_of::<RawCpuSet, _>(),
            slice_just_addr(&cpuset.bits)
        ))
    }
}

#[inline]
pub(crate) fn sched_yield() {
    unsafe {
        // See the documentation for [`crate::process::sched_yield`] for why
        // errors are ignored.
        syscall_readonly!(__NR_sched_yield).decode_void();
    }
}

#[inline]
pub(crate) fn uname() -> RawUname {
    let mut uname = MaybeUninit::<RawUname>::uninit();
    unsafe {
        ret_infallible(syscall!(__NR_uname, &mut uname));
        uname.assume_init()
    }
}

#[cfg(feature = "fs")]
#[inline]
pub(crate) fn umask(mode: Mode) -> Mode {
    unsafe {
        // TODO: Use `from_bits_retain` when we switch to bitflags 2.0.
        Mode::from_bits_truncate(ret_c_uint_infallible(syscall_readonly!(__NR_umask, mode)))
    }
}

#[inline]
pub(crate) fn nice(inc: i32) -> io::Result<i32> {
    let priority = if inc > -40 && inc < 40 {
        inc + getpriority_process(None)?
    } else {
        inc
    }
    // TODO: With Rust 1.50, use `.clamp` instead of `.min` and `.max`.
    //.clamp(-20, 19);
    .min(19)
    .max(-20);
    setpriority_process(None, priority)?;
    Ok(priority)
}

#[inline]
pub(crate) fn getpriority_user(uid: Uid) -> io::Result<i32> {
    unsafe {
        Ok(20
            - ret_c_int(syscall_readonly!(
                __NR_getpriority,
                c_uint(PRIO_USER),
                c_uint(uid.as_raw())
            ))?)
    }
}

#[inline]
pub(crate) fn getpriority_pgrp(pgid: Option<Pid>) -> io::Result<i32> {
    unsafe {
        Ok(20
            - ret_c_int(syscall_readonly!(
                __NR_getpriority,
                c_uint(PRIO_PGRP),
                c_uint(Pid::as_raw(pgid))
            ))?)
    }
}

#[inline]
pub(crate) fn getpriority_process(pid: Option<Pid>) -> io::Result<i32> {
    unsafe {
        Ok(20
            - ret_c_int(syscall_readonly!(
                __NR_getpriority,
                c_uint(PRIO_PROCESS),
                c_uint(Pid::as_raw(pid))
            ))?)
    }
}

#[inline]
pub(crate) fn setpriority_user(uid: Uid, priority: i32) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_setpriority,
            c_uint(PRIO_USER),
            c_uint(uid.as_raw()),
            c_int(priority)
        ))
    }
}

#[inline]
pub(crate) fn setpriority_pgrp(pgid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_setpriority,
            c_uint(PRIO_PGRP),
            c_uint(Pid::as_raw(pgid)),
            c_int(priority)
        ))
    }
}

#[inline]
pub(crate) fn setpriority_process(pid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_setpriority,
            c_uint(PRIO_PROCESS),
            c_uint(Pid::as_raw(pid)),
            c_int(priority)
        ))
    }
}

#[inline]
pub(crate) fn getrlimit(limit: Resource) -> Rlimit {
    let mut result = MaybeUninit::<rlimit64>::uninit();
    unsafe {
        match ret(syscall!(
            __NR_prlimit64,
            c_uint(0),
            limit,
            null::<c::c_void>(),
            &mut result
        )) {
            Ok(()) => rlimit_from_linux(result.assume_init()),
            Err(err) => {
                debug_assert_eq!(err, io::Errno::NOSYS);
                getrlimit_old(limit)
            }
        }
    }
}

/// The old 32-bit-only `getrlimit` syscall, for when we lack the new
/// `prlimit64`.
unsafe fn getrlimit_old(limit: Resource) -> Rlimit {
    let mut result = MaybeUninit::<rlimit>::uninit();

    // On these platforms, `__NR_getrlimit` is called `__NR_ugetrlimit`.
    #[cfg(any(
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "x86",
    ))]
    {
        ret_infallible(syscall!(__NR_ugetrlimit, limit, &mut result));
    }

    // On these platforms, it's just `__NR_getrlimit`.
    #[cfg(not(any(
        target_arch = "arm",
        target_arch = "powerpc",
        target_arch = "powerpc64",
        target_arch = "x86",
    )))]
    {
        ret_infallible(syscall!(__NR_getrlimit, limit, &mut result));
    }

    rlimit_from_linux_old(result.assume_init())
}

#[inline]
pub(crate) fn setrlimit(limit: Resource, new: Rlimit) -> io::Result<()> {
    unsafe {
        let lim = rlimit_to_linux(new.clone());
        match ret(syscall_readonly!(
            __NR_prlimit64,
            c_uint(0),
            limit,
            by_ref(&lim),
            null_mut::<c::c_void>()
        )) {
            Ok(()) => Ok(()),
            Err(io::Errno::NOSYS) => setrlimit_old(limit, new),
            Err(err) => Err(err),
        }
    }
}

/// The old 32-bit-only `setrlimit` syscall, for when we lack the new
/// `prlimit64`.
unsafe fn setrlimit_old(limit: Resource, new: Rlimit) -> io::Result<()> {
    let lim = rlimit_to_linux_old(new)?;
    ret(syscall_readonly!(__NR_setrlimit, limit, by_ref(&lim)))
}

#[inline]
pub(crate) fn prlimit(pid: Option<Pid>, limit: Resource, new: Rlimit) -> io::Result<Rlimit> {
    let lim = rlimit_to_linux(new);
    let mut result = MaybeUninit::<rlimit64>::uninit();
    unsafe {
        match ret(syscall!(
            __NR_prlimit64,
            c_uint(Pid::as_raw(pid)),
            limit,
            by_ref(&lim),
            &mut result
        )) {
            Ok(()) => Ok(rlimit_from_linux(result.assume_init())),
            Err(err) => Err(err),
        }
    }
}

/// Convert a Rust [`Rlimit`] to a C `rlimit64`.
#[inline]
fn rlimit_from_linux(lim: rlimit64) -> Rlimit {
    let current = if lim.rlim_cur == RLIM64_INFINITY as _ {
        None
    } else {
        Some(lim.rlim_cur)
    };
    let maximum = if lim.rlim_max == RLIM64_INFINITY as _ {
        None
    } else {
        Some(lim.rlim_max)
    };
    Rlimit { current, maximum }
}

/// Convert a C `rlimit64` to a Rust `Rlimit`.
#[inline]
fn rlimit_to_linux(lim: Rlimit) -> rlimit64 {
    let rlim_cur = match lim.current {
        Some(r) => r,
        None => RLIM64_INFINITY as _,
    };
    let rlim_max = match lim.maximum {
        Some(r) => r,
        None => RLIM64_INFINITY as _,
    };
    rlimit64 { rlim_cur, rlim_max }
}

/// Like `rlimit_from_linux` but uses Linux's old 32-bit `rlimit`.
#[allow(clippy::useless_conversion)]
fn rlimit_from_linux_old(lim: rlimit) -> Rlimit {
    let current = if lim.rlim_cur == RLIM_INFINITY as _ {
        None
    } else {
        Some(lim.rlim_cur.into())
    };
    let maximum = if lim.rlim_max == RLIM_INFINITY as _ {
        None
    } else {
        Some(lim.rlim_max.into())
    };
    Rlimit { current, maximum }
}

/// Like `rlimit_to_linux` but uses Linux's old 32-bit `rlimit`.
#[allow(clippy::useless_conversion)]
fn rlimit_to_linux_old(lim: Rlimit) -> io::Result<rlimit> {
    let rlim_cur = match lim.current {
        Some(r) => r.try_into().map_err(|_e| io::Errno::INVAL)?,
        None => RLIM_INFINITY as _,
    };
    let rlim_max = match lim.maximum {
        Some(r) => r.try_into().map_err(|_e| io::Errno::INVAL)?,
        None => RLIM_INFINITY as _,
    };
    Ok(rlimit { rlim_cur, rlim_max })
}

#[inline]
pub(crate) fn wait(waitopts: WaitOptions) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(!0, waitopts)
}

#[inline]
pub(crate) fn waitpid(
    pid: Option<Pid>,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(Pid::as_raw(pid), waitopts)
}

#[inline]
pub(crate) fn _waitpid(
    pid: RawPid,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    unsafe {
        let mut status = MaybeUninit::<u32>::uninit();
        let pid = ret_c_uint(syscall!(
            __NR_wait4,
            c_int(pid as _),
            &mut status,
            c_int(waitopts.bits() as _),
            zero()
        ))?;
        Ok(RawNonZeroPid::new(pid).map(|non_zero| {
            (
                Pid::from_raw_nonzero(non_zero),
                WaitStatus::new(status.assume_init()),
            )
        }))
    }
}

#[inline]
pub(crate) fn waitid(id: WaitId<'_>, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // Get the id to wait on.
    match id {
        WaitId::All => _waitid_all(options),
        WaitId::Pid(pid) => _waitid_pid(pid, options),
        WaitId::PidFd(fd) => _waitid_pidfd(fd, options),
    }
}

#[inline]
fn _waitid_all(options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(syscall!(
            __NR_waitid,
            c_uint(c::P_ALL),
            c_uint(0),
            by_mut(&mut status),
            c_int(options.bits() as _),
            zero()
        ))?
    };

    Ok(unsafe { cvt_waitid_status(status) })
}

#[inline]
fn _waitid_pid(pid: Pid, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(syscall!(
            __NR_waitid,
            c_uint(c::P_PID),
            c_uint(Pid::as_raw(Some(pid))),
            by_mut(&mut status),
            c_int(options.bits() as _),
            zero()
        ))?
    };

    Ok(unsafe { cvt_waitid_status(status) })
}

#[inline]
fn _waitid_pidfd(fd: BorrowedFd<'_>, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(syscall!(
            __NR_waitid,
            c_uint(c::P_PIDFD),
            c_uint(fd.as_raw_fd() as _),
            by_mut(&mut status),
            c_int(options.bits() as _),
            zero()
        ))?
    };

    Ok(unsafe { cvt_waitid_status(status) })
}

/// Convert a `siginfo_t` to a `WaitidStatus`.
///
/// # Safety
///
/// The caller must ensure that `status` is initialized and that `waitid`
/// returned successfully.
#[inline]
#[rustfmt::skip]
unsafe fn cvt_waitid_status(status: MaybeUninit<c::siginfo_t>) -> Option<WaitidStatus> {
    let status = status.assume_init();
    if status.__bindgen_anon_1.__bindgen_anon_1._sifields._sigchld._pid == 0 {
        None
    } else {
        Some(WaitidStatus(status))
    }
}

#[cfg(feature = "runtime")]
#[inline]
pub(crate) fn exit_group(code: c::c_int) -> ! {
    unsafe { syscall_noreturn!(__NR_exit_group, c_int(code)) }
}

#[inline]
pub(crate) fn getsid(pid: Option<Pid>) -> io::Result<Pid> {
    unsafe {
        let pid = ret_usize(syscall_readonly!(__NR_getsid, c_uint(Pid::as_raw(pid))))?;
        debug_assert!(pid > 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(
            pid as u32,
        )))
    }
}

#[inline]
pub(crate) fn setsid() -> io::Result<Pid> {
    unsafe {
        let pid = ret_usize(syscall_readonly!(__NR_setsid))?;
        debug_assert!(pid > 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(
            pid as u32,
        )))
    }
}

#[inline]
pub(crate) fn kill_process(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_kill, pid, sig)) }
}

#[inline]
pub(crate) fn kill_process_group(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_kill, negative_pid(pid), sig)) }
}

#[inline]
pub(crate) fn kill_current_process_group(sig: Signal) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_kill, pass_usize(0), sig)) }
}

#[inline]
pub(crate) fn test_kill_process(pid: Pid) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_kill, pid, pass_usize(0))) }
}

#[inline]
pub(crate) fn test_kill_process_group(pid: Pid) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_kill,
            negative_pid(pid),
            pass_usize(0)
        ))
    }
}

#[inline]
pub(crate) fn test_kill_current_process_group() -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_kill, pass_usize(0), pass_usize(0))) }
}

#[inline]
pub(crate) unsafe fn prctl(
    option: c::c_int,
    arg2: *mut c::c_void,
    arg3: *mut c::c_void,
    arg4: *mut c::c_void,
    arg5: *mut c::c_void,
) -> io::Result<c::c_int> {
    ret_c_int(syscall!(__NR_prctl, c_int(option), arg2, arg3, arg4, arg5))
}

#[inline]
pub(crate) fn pidfd_open(pid: Pid, flags: PidfdFlags) -> io::Result<OwnedFd> {
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_pidfd_open,
            pid,
            c_int(flags.bits() as _)
        ))
    }
}

#[inline]
pub(crate) fn getgroups(buf: &mut [Gid]) -> io::Result<usize> {
    let len = buf.len().try_into().map_err(|_| io::Errno::NOMEM)?;

    unsafe {
        ret_usize(syscall!(
            __NR_getgroups,
            c_int(len),
            slice_just_addr_mut(buf)
        ))
    }
}

#[inline]
pub(crate) fn sysinfo() -> Sysinfo {
    let mut info = MaybeUninit::<Sysinfo>::uninit();
    unsafe {
        ret_infallible(syscall!(__NR_sysinfo, &mut info));
        info.assume_init()
    }
}

#[inline]
pub(crate) fn sethostname(name: &[u8]) -> io::Result<()> {
    let (ptr, len) = slice(name);
    unsafe { ret(syscall_readonly!(__NR_sethostname, ptr, len)) }
}

#[inline]
pub(crate) fn ioctl_tiocsctty(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_ioctl,
            fd,
            c_uint(TIOCSCTTY),
            by_ref(&0_u32)
        ))
    }
}
