//! libc syscalls supporting `rustix::process`.

use super::super::c;
#[cfg(not(target_os = "wasi"))]
use super::super::conv::{borrowed_fd, ret_infallible, ret_pid_t, ret_usize};
use super::super::conv::{c_str, ret, ret_c_int, ret_discarded_char_ptr};
#[cfg(linux_kernel)]
use super::super::conv::{syscall_ret, syscall_ret_u32};
#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
use super::types::RawCpuSet;
#[cfg(not(target_os = "wasi"))]
use crate::fd::BorrowedFd;
#[cfg(target_os = "linux")]
use crate::fd::{AsRawFd, OwnedFd};
use crate::ffi::CStr;
#[cfg(feature = "fs")]
use crate::fs::Mode;
use crate::io;
#[cfg(linux_kernel)]
use crate::process::Sysinfo;
#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
use crate::process::{WaitId, WaitidOptions, WaitidStatus};
use core::mem::MaybeUninit;
#[cfg(target_os = "linux")]
use {super::super::conv::syscall_ret_owned_fd, crate::process::PidfdFlags};
#[cfg(linux_kernel)]
use {
    super::super::offset::libc_prlimit,
    crate::process::{Cpuid, MembarrierCommand, MembarrierQuery},
};
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
use {
    super::super::offset::{libc_getrlimit, libc_rlimit, libc_setrlimit, LIBC_RLIM_INFINITY},
    crate::process::{Resource, Rlimit},
};
#[cfg(not(target_os = "wasi"))]
use {
    super::types::RawUname,
    crate::process::{Gid, Pid, RawNonZeroPid, RawPid, Signal, Uid, WaitOptions, WaitStatus},
    core::convert::TryInto,
};

#[cfg(not(target_os = "wasi"))]
pub(crate) fn chdir(path: &CStr) -> io::Result<()> {
    unsafe { ret(c::chdir(c_str(path))) }
}

#[cfg(not(any(target_os = "wasi", target_os = "fuchsia")))]
pub(crate) fn fchdir(dirfd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::fchdir(borrowed_fd(dirfd))) }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
pub(crate) fn chroot(path: &CStr) -> io::Result<()> {
    unsafe { ret(c::chroot(c_str(path))) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn getcwd(buf: &mut [u8]) -> io::Result<()> {
    unsafe { ret_discarded_char_ptr(c::getcwd(buf.as_mut_ptr().cast(), buf.len())) }
}

#[cfg(linux_kernel)]
pub(crate) fn membarrier_query() -> MembarrierQuery {
    // glibc does not have a wrapper for `membarrier`; [the documentation]
    // says to use `syscall`.
    //
    // [the documentation]: https://man7.org/linux/man-pages/man2/membarrier.2.html#NOTES
    const MEMBARRIER_CMD_QUERY: u32 = 0;
    unsafe {
        match syscall_ret_u32(c::syscall(c::SYS_membarrier, MEMBARRIER_CMD_QUERY, 0)) {
            Ok(query) => MembarrierQuery::from_bits_unchecked(query),
            Err(_) => MembarrierQuery::empty(),
        }
    }
}

#[cfg(linux_kernel)]
pub(crate) fn membarrier(cmd: MembarrierCommand) -> io::Result<()> {
    unsafe { syscall_ret(c::syscall(c::SYS_membarrier, cmd as u32, 0)) }
}

#[cfg(linux_kernel)]
pub(crate) fn membarrier_cpu(cmd: MembarrierCommand, cpu: Cpuid) -> io::Result<()> {
    const MEMBARRIER_CMD_FLAG_CPU: u32 = 1;
    unsafe {
        syscall_ret(c::syscall(
            c::SYS_membarrier,
            cmd as u32,
            MEMBARRIER_CMD_FLAG_CPU,
            cpu.as_raw(),
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getuid() -> Uid {
    unsafe {
        let uid = c::getuid();
        Uid::from_raw(uid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn geteuid() -> Uid {
    unsafe {
        let uid = c::geteuid();
        Uid::from_raw(uid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getgid() -> Gid {
    unsafe {
        let gid = c::getgid();
        Gid::from_raw(gid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getegid() -> Gid {
    unsafe {
        let gid = c::getegid();
        Gid::from_raw(gid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getpid() -> Pid {
    unsafe {
        let pid = c::getpid();
        debug_assert_ne!(pid, 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getppid() -> Option<Pid> {
    unsafe {
        let pid: i32 = c::getppid();
        Pid::from_raw(pid)
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn getpgid(pid: Option<Pid>) -> io::Result<Pid> {
    unsafe {
        let pgid = ret_pid_t(c::getpgid(Pid::as_raw(pid) as _))?;
        debug_assert_ne!(pgid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pgid)))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn setpgid(pid: Option<Pid>, pgid: Option<Pid>) -> io::Result<()> {
    unsafe { ret(c::setpgid(Pid::as_raw(pid) as _, Pid::as_raw(pgid) as _)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub(crate) fn getpgrp() -> Pid {
    unsafe {
        let pgid = c::getpgrp();
        debug_assert_ne!(pgid, 0);
        Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pgid))
    }
}

#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
#[inline]
pub(crate) fn sched_getaffinity(pid: Option<Pid>, cpuset: &mut RawCpuSet) -> io::Result<()> {
    unsafe {
        ret(c::sched_getaffinity(
            Pid::as_raw(pid) as _,
            core::mem::size_of::<RawCpuSet>(),
            cpuset,
        ))
    }
}

#[cfg(any(linux_kernel, target_os = "dragonfly", target_os = "fuchsia"))]
#[inline]
pub(crate) fn sched_setaffinity(pid: Option<Pid>, cpuset: &RawCpuSet) -> io::Result<()> {
    unsafe {
        ret(c::sched_setaffinity(
            Pid::as_raw(pid) as _,
            core::mem::size_of::<RawCpuSet>(),
            cpuset,
        ))
    }
}

#[inline]
pub(crate) fn sched_yield() {
    unsafe {
        let _ = c::sched_yield();
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn uname() -> RawUname {
    let mut uname = MaybeUninit::<RawUname>::uninit();
    unsafe {
        ret_infallible(c::uname(uname.as_mut_ptr()));
        uname.assume_init()
    }
}

#[cfg(not(target_os = "wasi"))]
#[cfg(feature = "fs")]
#[inline]
pub(crate) fn umask(mask: Mode) -> Mode {
    // TODO: Use `from_bits_retain` when we switch to bitflags 2.0.
    unsafe { Mode::from_bits_truncate(c::umask(mask.bits() as _) as _) }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn nice(inc: i32) -> io::Result<i32> {
    libc_errno::set_errno(libc_errno::Errno(0));
    let r = unsafe { c::nice(inc) };
    if libc_errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_user(uid: Uid) -> io::Result<i32> {
    libc_errno::set_errno(libc_errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_USER, uid.as_raw() as _) };
    if libc_errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_pgrp(pgid: Option<Pid>) -> io::Result<i32> {
    libc_errno::set_errno(libc_errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_PGRP, Pid::as_raw(pgid) as _) };
    if libc_errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn getpriority_process(pid: Option<Pid>) -> io::Result<i32> {
    libc_errno::set_errno(libc_errno::Errno(0));
    let r = unsafe { c::getpriority(c::PRIO_PROCESS, Pid::as_raw(pid) as _) };
    if libc_errno::errno().0 != 0 {
        ret_c_int(r)
    } else {
        Ok(r)
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_user(uid: Uid, priority: i32) -> io::Result<()> {
    unsafe { ret(c::setpriority(c::PRIO_USER, uid.as_raw() as _, priority)) }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_pgrp(pgid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(c::setpriority(
            c::PRIO_PGRP,
            Pid::as_raw(pgid) as _,
            priority,
        ))
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[inline]
pub(crate) fn setpriority_process(pid: Option<Pid>, priority: i32) -> io::Result<()> {
    unsafe {
        ret(c::setpriority(
            c::PRIO_PROCESS,
            Pid::as_raw(pid) as _,
            priority,
        ))
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getrlimit(limit: Resource) -> Rlimit {
    let mut result = MaybeUninit::<libc_rlimit>::uninit();
    unsafe {
        ret_infallible(libc_getrlimit(limit as _, result.as_mut_ptr()));
        rlimit_from_libc(result.assume_init())
    }
}

#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn setrlimit(limit: Resource, new: Rlimit) -> io::Result<()> {
    let lim = rlimit_to_libc(new)?;
    unsafe { ret(libc_setrlimit(limit as _, &lim)) }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) fn prlimit(pid: Option<Pid>, limit: Resource, new: Rlimit) -> io::Result<Rlimit> {
    let lim = rlimit_to_libc(new)?;
    let mut result = MaybeUninit::<libc_rlimit>::uninit();
    unsafe {
        ret(libc_prlimit(
            Pid::as_raw(pid),
            limit as _,
            &lim,
            result.as_mut_ptr(),
        ))?;
        Ok(rlimit_from_libc(result.assume_init()))
    }
}

/// Convert a Rust [`Rlimit`] to a C `libc_rlimit`.
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
fn rlimit_from_libc(lim: libc_rlimit) -> Rlimit {
    let current = if lim.rlim_cur == LIBC_RLIM_INFINITY {
        None
    } else {
        Some(lim.rlim_cur.try_into().unwrap())
    };
    let maximum = if lim.rlim_max == LIBC_RLIM_INFINITY {
        None
    } else {
        Some(lim.rlim_max.try_into().unwrap())
    };
    Rlimit { current, maximum }
}

/// Convert a C `libc_rlimit` to a Rust `Rlimit`.
#[cfg(not(any(target_os = "fuchsia", target_os = "redox", target_os = "wasi")))]
fn rlimit_to_libc(lim: Rlimit) -> io::Result<libc_rlimit> {
    let Rlimit { current, maximum } = lim;
    let rlim_cur = match current {
        Some(r) => r.try_into().map_err(|_e| io::Errno::INVAL)?,
        None => LIBC_RLIM_INFINITY as _,
    };
    let rlim_max = match maximum {
        Some(r) => r.try_into().map_err(|_e| io::Errno::INVAL)?,
        None => LIBC_RLIM_INFINITY as _,
    };
    Ok(libc_rlimit { rlim_cur, rlim_max })
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn wait(waitopts: WaitOptions) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(!0, waitopts)
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn waitpid(
    pid: Option<Pid>,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    _waitpid(Pid::as_raw(pid), waitopts)
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn _waitpid(
    pid: RawPid,
    waitopts: WaitOptions,
) -> io::Result<Option<(Pid, WaitStatus)>> {
    unsafe {
        let mut status: c::c_int = 0;
        let pid = ret_c_int(c::waitpid(pid as _, &mut status, waitopts.bits() as _))?;
        Ok(RawNonZeroPid::new(pid).map(|non_zero| {
            (
                Pid::from_raw_nonzero(non_zero),
                WaitStatus::new(status as _),
            )
        }))
    }
}

#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[inline]
pub(crate) fn waitid(id: WaitId<'_>, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // Get the id to wait on.
    match id {
        WaitId::All => _waitid_all(options),
        WaitId::Pid(pid) => _waitid_pid(pid, options),
        #[cfg(target_os = "linux")]
        WaitId::PidFd(fd) => _waitid_pidfd(fd, options),
        #[cfg(not(target_os = "linux"))]
        WaitId::__EatLifetime(_) => unreachable!(),
    }
}

#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[inline]
fn _waitid_all(options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(c::waitid(
            c::P_ALL,
            0,
            status.as_mut_ptr(),
            options.bits() as _,
        ))?
    };

    Ok(unsafe { cvt_waitid_status(status) })
}

#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[inline]
fn _waitid_pid(pid: Pid, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(c::waitid(
            c::P_PID,
            Pid::as_raw(Some(pid)) as _,
            status.as_mut_ptr(),
            options.bits() as _,
        ))?
    };

    Ok(unsafe { cvt_waitid_status(status) })
}

#[cfg(target_os = "linux")]
#[inline]
fn _waitid_pidfd(fd: BorrowedFd<'_>, options: WaitidOptions) -> io::Result<Option<WaitidStatus>> {
    // `waitid` can return successfully without initializing the struct (no
    // children found when using `WNOHANG`)
    let mut status = MaybeUninit::<c::siginfo_t>::zeroed();
    unsafe {
        ret(c::waitid(
            c::P_PIDFD,
            fd.as_raw_fd() as _,
            status.as_mut_ptr(),
            options.bits() as _,
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
#[cfg(not(any(target_os = "wasi", target_os = "redox", target_os = "openbsd")))]
#[inline]
unsafe fn cvt_waitid_status(status: MaybeUninit<c::siginfo_t>) -> Option<WaitidStatus> {
    let status = status.assume_init();
    // `si_pid` is supposedly the better way to check that the struct has been
    // filled, e.g. the Linux manpage says about the `WNOHANG` case “zero out
    // the si_pid field before the call and check for a nonzero value”.
    // But e.g. NetBSD/OpenBSD don't have it exposed in the libc crate for now,
    // and some platforms don't have it at all. For simplicity, always check
    // `si_signo`. We have zero-initialized the whole struct, and all kernels
    // should set `SIGCHLD` here.
    if status.si_signo == 0 {
        None
    } else {
        Some(WaitidStatus(status))
    }
}

#[inline]
pub(crate) fn exit_group(code: c::c_int) -> ! {
    // `_exit` and `_Exit` are the same; it's just a matter of which ones
    // the libc bindings expose.
    #[cfg(any(target_os = "wasi", target_os = "solid"))]
    unsafe {
        c::_Exit(code)
    }
    #[cfg(unix)]
    unsafe {
        c::_exit(code)
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn getsid(pid: Option<Pid>) -> io::Result<Pid> {
    unsafe {
        let pid = ret_pid_t(c::getsid(Pid::as_raw(pid) as _))?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn setsid() -> io::Result<Pid> {
    unsafe {
        let pid = ret_c_int(c::setsid())?;
        debug_assert_ne!(pid, 0);
        Ok(Pid::from_raw_nonzero(RawNonZeroPid::new_unchecked(pid)))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_process(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe { ret(c::kill(pid.as_raw_nonzero().get(), sig as i32)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_process_group(pid: Pid, sig: Signal) -> io::Result<()> {
    unsafe {
        ret(c::kill(
            pid.as_raw_nonzero().get().wrapping_neg(),
            sig as i32,
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn kill_current_process_group(sig: Signal) -> io::Result<()> {
    unsafe { ret(c::kill(0, sig as i32)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn test_kill_process(pid: Pid) -> io::Result<()> {
    unsafe { ret(c::kill(pid.as_raw_nonzero().get(), 0)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn test_kill_process_group(pid: Pid) -> io::Result<()> {
    unsafe { ret(c::kill(pid.as_raw_nonzero().get().wrapping_neg(), 0)) }
}

#[cfg(not(target_os = "wasi"))]
#[inline]
pub(crate) fn test_kill_current_process_group() -> io::Result<()> {
    unsafe { ret(c::kill(0, 0)) }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) unsafe fn prctl(
    option: c::c_int,
    arg2: *mut c::c_void,
    arg3: *mut c::c_void,
    arg4: *mut c::c_void,
    arg5: *mut c::c_void,
) -> io::Result<c::c_int> {
    ret_c_int(c::prctl(option, arg2, arg3, arg4, arg5))
}

#[cfg(freebsdlike)]
#[inline]
pub(crate) unsafe fn procctl(
    idtype: c::idtype_t,
    id: c::id_t,
    option: c::c_int,
    data: *mut c::c_void,
) -> io::Result<()> {
    ret(c::procctl(idtype, id, option, data))
}

#[cfg(target_os = "linux")]
pub(crate) fn pidfd_open(pid: Pid, flags: PidfdFlags) -> io::Result<OwnedFd> {
    unsafe {
        syscall_ret_owned_fd(c::syscall(
            c::SYS_pidfd_open,
            pid.as_raw_nonzero().get(),
            flags.bits(),
        ))
    }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn getgroups(buf: &mut [Gid]) -> io::Result<usize> {
    let len = buf.len().try_into().map_err(|_| io::Errno::NOMEM)?;

    unsafe { ret_usize(c::getgroups(len, buf.as_mut_ptr().cast()) as isize) }
}

#[cfg(linux_kernel)]
pub(crate) fn sysinfo() -> Sysinfo {
    let mut info = MaybeUninit::<Sysinfo>::uninit();
    unsafe {
        ret_infallible(c::sysinfo(info.as_mut_ptr()));
        info.assume_init()
    }
}

#[cfg(not(any(target_os = "emscripten", target_os = "redox", target_os = "wasi")))]
pub(crate) fn sethostname(name: &[u8]) -> io::Result<()> {
    unsafe {
        ret(c::sethostname(
            name.as_ptr().cast(),
            name.len().try_into().map_err(|_| io::Errno::INVAL)?,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn ioctl_tiocsctty(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCSCTTY as _, &0_u32)) }
}
