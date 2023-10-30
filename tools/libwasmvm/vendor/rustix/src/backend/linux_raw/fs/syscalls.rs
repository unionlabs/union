//! linux_raw syscalls supporting `rustix::fs`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(dead_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
use super::super::conv::{
    by_ref, c_int, c_uint, dev_t, oflags_for_open_how, opt_mut, pass_usize, raw_fd, ret, ret_c_int,
    ret_c_uint, ret_infallible, ret_owned_fd, ret_usize, size_of, slice, slice_mut, zero,
};
#[cfg(target_pointer_width = "64")]
use super::super::conv::{loff_t, loff_t_from_u64, ret_u64};
#[cfg(any(
    target_arch = "aarch64",
    target_arch = "riscv64",
    target_arch = "mips64",
    target_pointer_width = "32",
))]
use crate::fd::AsFd;
use crate::fd::{BorrowedFd, OwnedFd};
use crate::ffi::CStr;
use crate::fs::{
    inotify, Access, Advice, AtFlags, FallocateFlags, FileType, FlockOperation, MemfdFlags, Mode,
    OFlags, RenameFlags, ResolveFlags, SealFlags, Stat, StatFs, StatVfs, StatVfsMountFlags,
    StatxFlags, Timestamps, XattrFlags,
};
use crate::io::{self, SeekFrom};
use crate::process::{Gid, Uid};
#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
use core::convert::TryInto;
use core::mem::{transmute, zeroed, MaybeUninit};
#[cfg(target_arch = "mips64")]
use linux_raw_sys::general::stat as linux_stat64;
use linux_raw_sys::general::{
    __kernel_fsid_t, __kernel_timespec, open_how, statx, AT_EACCESS, AT_FDCWD, AT_REMOVEDIR,
    AT_SYMLINK_NOFOLLOW, F_ADD_SEALS, F_GETFL, F_GETLEASE, F_GETOWN, F_GETPIPE_SZ, F_GETSIG,
    F_GET_SEALS, F_SETFL, F_SETPIPE_SZ, SEEK_CUR, SEEK_DATA, SEEK_END, SEEK_HOLE, SEEK_SET,
    STATX__RESERVED,
};
#[cfg(target_pointer_width = "32")]
use {
    super::super::conv::{hi, lo, slice_just_addr},
    linux_raw_sys::general::stat64 as linux_stat64,
    linux_raw_sys::general::timespec as __kernel_old_timespec,
};

#[inline]
pub(crate) fn open(filename: &CStr, flags: OFlags, mode: Mode) -> io::Result<OwnedFd> {
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    {
        openat(crate::fs::cwd().as_fd(), filename, flags, mode)
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_arch = "aarch64", target_arch = "riscv64")),
    ))]
    unsafe {
        ret_owned_fd(syscall_readonly!(__NR_open, filename, flags, mode))
    }
    #[cfg(all(
        target_pointer_width = "64",
        not(any(target_arch = "aarch64", target_arch = "riscv64")),
    ))]
    unsafe {
        ret_owned_fd(syscall_readonly!(__NR_open, filename, flags, mode))
    }
}

#[inline]
pub(crate) fn openat(
    dirfd: BorrowedFd<'_>,
    filename: &CStr,
    flags: OFlags,
    mode: Mode,
) -> io::Result<OwnedFd> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_owned_fd(syscall_readonly!(__NR_openat, dirfd, filename, flags, mode))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_owned_fd(syscall_readonly!(__NR_openat, dirfd, filename, flags, mode))
    }
}

#[inline]
pub(crate) fn openat2(
    dirfd: BorrowedFd<'_>,
    pathname: &CStr,
    flags: OFlags,
    mode: Mode,
    resolve: ResolveFlags,
) -> io::Result<OwnedFd> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_openat2,
            dirfd,
            pathname,
            by_ref(&open_how {
                flags: oflags_for_open_how(flags),
                mode: u64::from(mode.bits()),
                resolve: resolve.bits(),
            }),
            size_of::<open_how, _>()
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_openat2,
            dirfd,
            pathname,
            by_ref(&open_how {
                flags: oflags_for_open_how(flags),
                mode: u64::from(mode.bits()),
                resolve: resolve.bits(),
            }),
            size_of::<open_how, _>()
        ))
    }
}

#[inline]
pub(crate) fn chmod(filename: &CStr, mode: Mode) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_fchmodat,
            raw_fd(AT_FDCWD),
            filename,
            mode
        ))
    }
}

#[inline]
pub(crate) fn chmodat(
    dirfd: BorrowedFd<'_>,
    filename: &CStr,
    mode: Mode,
    flags: AtFlags,
) -> io::Result<()> {
    if flags == AtFlags::SYMLINK_NOFOLLOW {
        return Err(io::Errno::OPNOTSUPP);
    }
    if !flags.is_empty() {
        return Err(io::Errno::INVAL);
    }
    unsafe { ret(syscall_readonly!(__NR_fchmodat, dirfd, filename, mode)) }
}

#[inline]
pub(crate) fn fchmod(fd: BorrowedFd<'_>, mode: Mode) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_fchmod, fd, mode)) }
}

#[inline]
pub(crate) fn chownat(
    dirfd: BorrowedFd<'_>,
    filename: &CStr,
    owner: Option<Uid>,
    group: Option<Gid>,
    flags: AtFlags,
) -> io::Result<()> {
    unsafe {
        let (ow, gr) = crate::process::translate_fchown_args(owner, group);
        ret(syscall_readonly!(
            __NR_fchownat,
            dirfd,
            filename,
            c_uint(ow),
            c_uint(gr),
            flags
        ))
    }
}

#[inline]
pub(crate) fn fchown(fd: BorrowedFd<'_>, owner: Option<Uid>, group: Option<Gid>) -> io::Result<()> {
    unsafe {
        let (ow, gr) = crate::process::translate_fchown_args(owner, group);
        ret(syscall_readonly!(__NR_fchown, fd, c_uint(ow), c_uint(gr)))
    }
}

#[inline]
pub(crate) fn mknodat(
    dirfd: BorrowedFd<'_>,
    filename: &CStr,
    file_type: FileType,
    mode: Mode,
    dev: u64,
) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(
            __NR_mknodat,
            dirfd,
            filename,
            (mode, file_type),
            dev_t(dev)?
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_mknodat,
            dirfd,
            filename,
            (mode, file_type),
            dev_t(dev)
        ))
    }
}

#[inline]
pub(crate) fn seek(fd: BorrowedFd<'_>, pos: SeekFrom) -> io::Result<u64> {
    let (whence, offset) = match pos {
        SeekFrom::Start(pos) => {
            let pos: u64 = pos;
            // Silently cast; we'll get `EINVAL` if the value is negative.
            (SEEK_SET, pos as i64)
        }
        SeekFrom::End(offset) => (SEEK_END, offset),
        SeekFrom::Current(offset) => (SEEK_CUR, offset),
        #[cfg(any(freebsdlike, target_os = "linux", target_os = "solaris"))]
        SeekFrom::Data(offset) => (SEEK_DATA, offset),
        #[cfg(any(freebsdlike, target_os = "linux", target_os = "solaris"))]
        SeekFrom::Hole(offset) => (SEEK_HOLE, offset),
    };
    _seek(fd, offset, whence)
}

#[inline]
pub(crate) fn _seek(fd: BorrowedFd<'_>, offset: i64, whence: c::c_uint) -> io::Result<u64> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        let mut result = MaybeUninit::<u64>::uninit();
        ret(syscall!(
            __NR__llseek,
            fd,
            // Don't use the hi/lo functions here because Linux's llseek
            // takes its 64-bit argument differently from everything else.
            pass_usize((offset >> 32) as usize),
            pass_usize(offset as usize),
            &mut result,
            c_uint(whence)
        ))?;
        Ok(result.assume_init())
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_u64(syscall_readonly!(
            __NR_lseek,
            fd,
            loff_t(offset),
            c_uint(whence)
        ))
    }
}

#[inline]
pub(crate) fn tell(fd: BorrowedFd<'_>) -> io::Result<u64> {
    _seek(fd, 0, SEEK_CUR).map(|x| x as u64)
}

#[inline]
pub(crate) fn ftruncate(fd: BorrowedFd<'_>, length: u64) -> io::Result<()> {
    // <https://github.com/torvalds/linux/blob/fcadab740480e0e0e9fa9bd272acd409884d431a/arch/arm64/kernel/sys32.c#L81-L83>
    #[cfg(all(
        target_pointer_width = "32",
        any(target_arch = "arm", target_arch = "mips", target_arch = "powerpc"),
    ))]
    unsafe {
        ret(syscall_readonly!(
            __NR_ftruncate64,
            fd,
            zero(),
            hi(length),
            lo(length)
        ))
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_arch = "arm", target_arch = "mips", target_arch = "powerpc")),
    ))]
    unsafe {
        ret(syscall_readonly!(
            __NR_ftruncate64,
            fd,
            hi(length),
            lo(length)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_ftruncate,
            fd,
            loff_t_from_u64(length)
        ))
    }
}

#[inline]
pub(crate) fn fallocate(
    fd: BorrowedFd<'_>,
    mode: FallocateFlags,
    offset: u64,
    len: u64,
) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fallocate,
            fd,
            mode,
            hi(offset),
            lo(offset),
            hi(len),
            lo(len)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fallocate,
            fd,
            mode,
            loff_t_from_u64(offset),
            loff_t_from_u64(len)
        ))
    }
}

#[inline]
pub(crate) fn fadvise(fd: BorrowedFd<'_>, pos: u64, len: u64, advice: Advice) -> io::Result<()> {
    // On ARM, the arguments are reordered so that the len and pos argument
    // pairs are aligned. And ARM has a custom syscall code for this.
    #[cfg(target_arch = "arm")]
    unsafe {
        ret(syscall_readonly!(
            __NR_arm_fadvise64_64,
            fd,
            advice,
            hi(pos),
            lo(pos),
            hi(len),
            lo(len)
        ))
    }

    // On powerpc, the arguments are reordered as on ARM.
    #[cfg(target_arch = "powerpc")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fadvise64_64,
            fd,
            advice,
            hi(pos),
            lo(pos),
            hi(len),
            lo(len)
        ))
    }
    // On mips, the arguments are not reordered, and padding is inserted
    // instead to ensure alignment.
    #[cfg(target_arch = "mips")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fadvise64,
            fd,
            zero(),
            hi(pos),
            lo(pos),
            hi(len),
            lo(len),
            advice
        ))
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_arch = "arm", target_arch = "mips", target_arch = "powerpc")),
    ))]
    unsafe {
        ret(syscall_readonly!(
            __NR_fadvise64_64,
            fd,
            hi(pos),
            lo(pos),
            hi(len),
            lo(len),
            advice
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fadvise64,
            fd,
            loff_t_from_u64(pos),
            loff_t_from_u64(len),
            advice
        ))
    }
}

#[inline]
pub(crate) fn fsync(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_fsync, fd)) }
}

#[inline]
pub(crate) fn fdatasync(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_fdatasync, fd)) }
}

#[inline]
pub(crate) fn flock(fd: BorrowedFd<'_>, operation: FlockOperation) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_flock,
            fd,
            c_uint(operation as c::c_uint)
        ))
    }
}

#[inline]
pub(crate) fn syncfs(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_syncfs, fd)) }
}

#[inline]
pub(crate) fn sync() {
    unsafe { ret_infallible(syscall_readonly!(__NR_sync)) }
}

#[inline]
pub(crate) fn fstat(fd: BorrowedFd<'_>) -> io::Result<Stat> {
    // 32-bit and mips64 Linux: `struct stat64` is not y2038 compatible; use
    // `statx`.
    //
    // And, some old platforms don't support `statx`, and some fail with a
    // confusing error code, so we call `crate::fs::statx` to handle that. If
    // `statx` isn't available, fall back to the buggy system call.
    #[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
    {
        match crate::fs::statx(fd, cstr!(""), AtFlags::EMPTY_PATH, StatxFlags::BASIC_STATS) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => fstat_old(fd),
            Err(err) => Err(err),
        }
    }

    #[cfg(all(target_pointer_width = "64", not(target_arch = "mips64")))]
    unsafe {
        let mut result = MaybeUninit::<Stat>::uninit();
        ret(syscall!(__NR_fstat, fd, &mut result))?;
        Ok(result.assume_init())
    }
}

#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
fn fstat_old(fd: BorrowedFd<'_>) -> io::Result<Stat> {
    let mut result = MaybeUninit::<linux_stat64>::uninit();

    #[cfg(target_arch = "mips64")]
    unsafe {
        ret(syscall!(__NR_fstat, fd, &mut result))?;
        stat_to_stat(result.assume_init())
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall!(__NR_fstat64, fd, &mut result))?;
        stat_to_stat(result.assume_init())
    }
}

#[inline]
pub(crate) fn stat(filename: &CStr) -> io::Result<Stat> {
    // See the comments in `fstat` about using `crate::fs::statx` here.
    #[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
    {
        match crate::fs::statx(
            crate::fs::cwd().as_fd(),
            filename,
            AtFlags::empty(),
            StatxFlags::BASIC_STATS,
        ) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => stat_old(filename),
            Err(err) => Err(err),
        }
    }

    #[cfg(all(target_pointer_width = "64", not(target_arch = "mips64")))]
    unsafe {
        let mut result = MaybeUninit::<Stat>::uninit();
        ret(syscall!(
            __NR_newfstatat,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(0)
        ))?;
        Ok(result.assume_init())
    }
}

#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
fn stat_old(filename: &CStr) -> io::Result<Stat> {
    let mut result = MaybeUninit::<linux_stat64>::uninit();

    #[cfg(target_arch = "mips64")]
    unsafe {
        ret(syscall!(
            __NR_newfstatat,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(0)
        ))?;
        stat_to_stat(result.assume_init())
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall!(
            __NR_fstatat64,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(0)
        ))?;
        stat_to_stat(result.assume_init())
    }
}

#[inline]
pub(crate) fn statat(dirfd: BorrowedFd<'_>, filename: &CStr, flags: AtFlags) -> io::Result<Stat> {
    // See the comments in `fstat` about using `crate::fs::statx` here.
    #[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
    {
        match crate::fs::statx(dirfd, filename, flags, StatxFlags::BASIC_STATS) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => statat_old(dirfd, filename, flags),
            Err(err) => Err(err),
        }
    }

    #[cfg(all(target_pointer_width = "64", not(target_arch = "mips64")))]
    unsafe {
        let mut result = MaybeUninit::<Stat>::uninit();
        ret(syscall!(
            __NR_newfstatat,
            dirfd,
            filename,
            &mut result,
            flags
        ))?;
        Ok(result.assume_init())
    }
}

#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
fn statat_old(dirfd: BorrowedFd<'_>, filename: &CStr, flags: AtFlags) -> io::Result<Stat> {
    let mut result = MaybeUninit::<linux_stat64>::uninit();

    #[cfg(target_arch = "mips64")]
    unsafe {
        ret(syscall!(
            __NR_newfstatat,
            dirfd,
            filename,
            &mut result,
            flags
        ))?;
        stat_to_stat(result.assume_init())
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall!(
            __NR_fstatat64,
            dirfd,
            filename,
            &mut result,
            flags
        ))?;
        stat_to_stat(result.assume_init())
    }
}

#[inline]
pub(crate) fn lstat(filename: &CStr) -> io::Result<Stat> {
    // See the comments in `fstat` about using `crate::fs::statx` here.
    #[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
    {
        match crate::fs::statx(
            crate::fs::cwd().as_fd(),
            filename,
            AtFlags::SYMLINK_NOFOLLOW,
            StatxFlags::BASIC_STATS,
        ) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => lstat_old(filename),
            Err(err) => Err(err),
        }
    }

    #[cfg(all(target_pointer_width = "64", not(target_arch = "mips64")))]
    unsafe {
        let mut result = MaybeUninit::<Stat>::uninit();
        ret(syscall!(
            __NR_newfstatat,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(AT_SYMLINK_NOFOLLOW)
        ))?;
        Ok(result.assume_init())
    }
}

#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
fn lstat_old(filename: &CStr) -> io::Result<Stat> {
    let mut result = MaybeUninit::<linux_stat64>::uninit();

    #[cfg(target_arch = "mips64")]
    unsafe {
        ret(syscall!(
            __NR_newfstatat,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(AT_SYMLINK_NOFOLLOW)
        ))?;
        stat_to_stat(result.assume_init())
    }

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall!(
            __NR_fstatat64,
            raw_fd(AT_FDCWD),
            filename,
            &mut result,
            c_uint(AT_SYMLINK_NOFOLLOW)
        ))?;
        stat_to_stat(result.assume_init())
    }
}

/// Convert from a Linux `statx` value to rustix's `Stat`.
#[cfg(any(target_pointer_width = "32", target_arch = "mips64"))]
fn statx_to_stat(x: crate::fs::Statx) -> io::Result<Stat> {
    Ok(Stat {
        st_dev: crate::fs::makedev(x.stx_dev_major, x.stx_dev_minor),
        st_mode: x.stx_mode.into(),
        st_nlink: x.stx_nlink.into(),
        st_uid: x.stx_uid.into(),
        st_gid: x.stx_gid.into(),
        st_rdev: crate::fs::makedev(x.stx_rdev_major, x.stx_rdev_minor),
        st_size: x.stx_size.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blksize: x.stx_blksize.into(),
        st_blocks: x.stx_blocks.into(),
        st_atime: x
            .stx_atime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_atime_nsec: x.stx_atime.tv_nsec.into(),
        st_mtime: x
            .stx_mtime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime_nsec: x.stx_mtime.tv_nsec.into(),
        st_ctime: x
            .stx_ctime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime_nsec: x.stx_ctime.tv_nsec.into(),
        st_ino: x.stx_ino.into(),
    })
}

/// Convert from a Linux `stat64` value to rustix's `Stat`.
#[cfg(target_pointer_width = "32")]
fn stat_to_stat(s64: linux_raw_sys::general::stat64) -> io::Result<Stat> {
    Ok(Stat {
        st_dev: s64.st_dev.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_mode: s64.st_mode.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_nlink: s64.st_nlink.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_uid: s64.st_uid.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_gid: s64.st_gid.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_rdev: s64.st_rdev.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_size: s64.st_size.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blksize: s64.st_blksize.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blocks: s64.st_blocks.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_atime: s64.st_atime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_atime_nsec: s64
            .st_atime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime: s64.st_mtime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime_nsec: s64
            .st_mtime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime: s64.st_ctime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime_nsec: s64
            .st_ctime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ino: s64.st_ino.try_into().map_err(|_| io::Errno::OVERFLOW)?,
    })
}

/// Convert from a Linux `stat` value to rustix's `Stat`.
#[cfg(target_arch = "mips64")]
fn stat_to_stat(s: linux_raw_sys::general::stat) -> io::Result<Stat> {
    Ok(Stat {
        st_dev: s.st_dev.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_mode: s.st_mode.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_nlink: s.st_nlink.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_uid: s.st_uid.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_gid: s.st_gid.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_rdev: s.st_rdev.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_size: s.st_size.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blksize: s.st_blksize.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blocks: s.st_blocks.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_atime: s.st_atime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_atime_nsec: s
            .st_atime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime: s.st_mtime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime_nsec: s
            .st_mtime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime: s.st_ctime.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime_nsec: s
            .st_ctime_nsec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ino: s.st_ino.try_into().map_err(|_| io::Errno::OVERFLOW)?,
    })
}

#[inline]
pub(crate) fn statx(
    dirfd: BorrowedFd<'_>,
    pathname: &CStr,
    flags: AtFlags,
    mask: StatxFlags,
) -> io::Result<statx> {
    // If a future Linux kernel adds more fields to `struct statx` and users
    // passing flags unknown to rustix in `StatxFlags`, we could end up
    // writing outside of the buffer. To prevent this possibility, we mask off
    // any flags that we don't know about.
    //
    // This includes `STATX__RESERVED`, which has a value that we know, but
    // which could take on arbitrary new meaning in the future. Linux currently
    // rejects this flag with `EINVAL`, so we do the same.
    //
    // This doesn't rely on `STATX_ALL` because [it's deprecated] and already
    // doesn't represent all the known flags.
    //
    // [it's deprecated]: https://patchwork.kernel.org/project/linux-fsdevel/patch/20200505095915.11275-7-mszeredi@redhat.com/
    if (mask.bits() & STATX__RESERVED) == STATX__RESERVED {
        return Err(io::Errno::INVAL);
    }
    let mask = mask & StatxFlags::all();

    unsafe {
        let mut statx_buf = MaybeUninit::<statx>::uninit();
        ret(syscall!(
            __NR_statx,
            dirfd,
            pathname,
            flags,
            mask,
            &mut statx_buf
        ))?;
        Ok(statx_buf.assume_init())
    }
}

#[inline]
pub(crate) fn is_statx_available() -> bool {
    unsafe {
        // Call `statx` with null pointers so that if it fails for any reason
        // other than `EFAULT`, we know it's not supported.
        matches!(
            ret(syscall!(
                __NR_statx,
                raw_fd(AT_FDCWD),
                zero(),
                zero(),
                zero(),
                zero()
            )),
            Err(io::Errno::FAULT)
        )
    }
}

#[inline]
pub(crate) fn fstatfs(fd: BorrowedFd<'_>) -> io::Result<StatFs> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        let mut result = MaybeUninit::<StatFs>::uninit();
        ret(syscall!(
            __NR_fstatfs64,
            fd,
            size_of::<StatFs, _>(),
            &mut result
        ))?;
        Ok(result.assume_init())
    }

    #[cfg(target_pointer_width = "64")]
    unsafe {
        let mut result = MaybeUninit::<StatFs>::uninit();
        ret(syscall!(__NR_fstatfs, fd, &mut result))?;
        Ok(result.assume_init())
    }
}

#[inline]
pub(crate) fn fstatvfs(fd: BorrowedFd<'_>) -> io::Result<StatVfs> {
    // Linux doesn't have an `fstatvfs` syscall; we have to do `fstatfs` and
    // translate the fields as best we can.
    let statfs = fstatfs(fd)?;

    Ok(statfs_to_statvfs(statfs))
}

#[inline]
pub(crate) fn statfs(filename: &CStr) -> io::Result<StatFs> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        let mut result = MaybeUninit::<StatFs>::uninit();
        ret(syscall!(
            __NR_statfs64,
            filename,
            size_of::<StatFs, _>(),
            &mut result
        ))?;
        Ok(result.assume_init())
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        let mut result = MaybeUninit::<StatFs>::uninit();
        ret(syscall!(__NR_statfs, filename, &mut result))?;
        Ok(result.assume_init())
    }
}

#[inline]
pub(crate) fn statvfs(filename: &CStr) -> io::Result<StatVfs> {
    // Linux doesn't have a `statvfs` syscall; we have to do `statfs` and
    // translate the fields as best we can.
    let statfs = statfs(filename)?;

    Ok(statfs_to_statvfs(statfs))
}

fn statfs_to_statvfs(statfs: StatFs) -> StatVfs {
    let __kernel_fsid_t { val } = statfs.f_fsid;
    let [f_fsid_val0, f_fsid_val1]: [i32; 2] = val;

    StatVfs {
        f_bsize: statfs.f_bsize as u64,
        f_frsize: if statfs.f_frsize != 0 {
            statfs.f_frsize
        } else {
            statfs.f_bsize
        } as u64,
        f_blocks: statfs.f_blocks as u64,
        f_bfree: statfs.f_bfree as u64,
        f_bavail: statfs.f_bavail as u64,
        f_files: statfs.f_files as u64,
        f_ffree: statfs.f_ffree as u64,
        f_favail: statfs.f_ffree as u64,
        f_fsid: f_fsid_val0 as u32 as u64 | ((f_fsid_val1 as u32 as u64) << 32),
        f_flag: unsafe { StatVfsMountFlags::from_bits_unchecked(statfs.f_flags as u64) },
        f_namemax: statfs.f_namelen as u64,
    }
}

#[inline]
pub(crate) fn readlink(path: &CStr, buf: &mut [u8]) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);
    unsafe {
        ret_usize(syscall!(
            __NR_readlinkat,
            raw_fd(AT_FDCWD),
            path,
            buf_addr_mut,
            buf_len
        ))
    }
}

#[inline]
pub(crate) fn readlinkat(dirfd: BorrowedFd<'_>, path: &CStr, buf: &mut [u8]) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);
    unsafe {
        ret_usize(syscall!(
            __NR_readlinkat,
            dirfd,
            path,
            buf_addr_mut,
            buf_len
        ))
    }
}

#[inline]
pub(crate) fn fcntl_getfl(fd: BorrowedFd<'_>) -> io::Result<OFlags> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_uint(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETFL)))
            .map(OFlags::from_bits_truncate)
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_uint(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETFL)))
            .map(OFlags::from_bits_truncate)
    }
}

#[inline]
pub(crate) fn fcntl_setfl(fd: BorrowedFd<'_>, flags: OFlags) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_SETFL), flags))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(__NR_fcntl, fd, c_uint(F_SETFL), flags))
    }
}

#[inline]
pub(crate) fn fcntl_getlease(fd: BorrowedFd<'_>) -> io::Result<c::c_int> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETLEASE)))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETLEASE)))
    }
}

#[inline]
pub(crate) fn fcntl_getown(fd: BorrowedFd<'_>) -> io::Result<c::c_int> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETOWN)))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETOWN)))
    }
}

#[inline]
pub(crate) fn fcntl_getsig(fd: BorrowedFd<'_>) -> io::Result<c::c_int> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETSIG)))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETSIG)))
    }
}

#[inline]
pub(crate) fn fcntl_getpipe_sz(fd: BorrowedFd<'_>) -> io::Result<usize> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETPIPE_SZ)))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETPIPE_SZ)))
    }
}

#[inline]
pub(crate) fn fcntl_setpipe_sz(fd: BorrowedFd<'_>, size: c::c_int) -> io::Result<usize> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_fcntl64,
            fd,
            c_uint(F_SETPIPE_SZ),
            c_int(size)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_fcntl,
            fd,
            c_uint(F_SETPIPE_SZ),
            c_int(size)
        ))
    }
}

#[inline]
pub(crate) fn fcntl_get_seals(fd: BorrowedFd<'_>) -> io::Result<SealFlags> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GET_SEALS)))
            .map(|seals| SealFlags::from_bits_unchecked(seals as u32))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_int(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GET_SEALS)))
            .map(|seals| SealFlags::from_bits_unchecked(seals as u32))
    }
}

#[inline]
pub(crate) fn fcntl_add_seals(fd: BorrowedFd<'_>, seals: SealFlags) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fcntl64,
            fd,
            c_uint(F_ADD_SEALS),
            seals
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_fcntl,
            fd,
            c_uint(F_ADD_SEALS),
            seals
        ))
    }
}

#[inline]
pub(crate) fn fcntl_lock(fd: BorrowedFd<'_>, operation: FlockOperation) -> io::Result<()> {
    #[cfg(target_pointer_width = "64")]
    use linux_raw_sys::general::{flock, F_SETLK, F_SETLKW};
    #[cfg(target_pointer_width = "32")]
    use linux_raw_sys::general::{flock64 as flock, F_SETLK64 as F_SETLK, F_SETLKW64 as F_SETLKW};
    use linux_raw_sys::general::{F_RDLCK, F_UNLCK, F_WRLCK};

    let (cmd, l_type) = match operation {
        FlockOperation::LockShared => (F_SETLKW, F_RDLCK),
        FlockOperation::LockExclusive => (F_SETLKW, F_WRLCK),
        FlockOperation::Unlock => (F_SETLKW, F_UNLCK),
        FlockOperation::NonBlockingLockShared => (F_SETLK, F_RDLCK),
        FlockOperation::NonBlockingLockExclusive => (F_SETLK, F_WRLCK),
        FlockOperation::NonBlockingUnlock => (F_SETLK, F_UNLCK),
    };

    unsafe {
        let lock = flock {
            l_type: l_type as _,

            // When `l_len` is zero, this locks all the bytes from
            // `l_whence`/`l_start` to the end of the file, even as the
            // file grows dynamically.
            l_whence: SEEK_SET as _,
            l_start: 0,
            l_len: 0,

            ..zeroed()
        };

        #[cfg(target_pointer_width = "32")]
        {
            ret(syscall_readonly!(
                __NR_fcntl64,
                fd,
                c_uint(cmd),
                by_ref(&lock)
            ))
        }
        #[cfg(target_pointer_width = "64")]
        {
            ret(syscall_readonly!(
                __NR_fcntl,
                fd,
                c_uint(cmd),
                by_ref(&lock)
            ))
        }
    }
}

#[inline]
pub(crate) fn rename(oldname: &CStr, newname: &CStr) -> io::Result<()> {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_renameat2,
            raw_fd(AT_FDCWD),
            oldname,
            raw_fd(AT_FDCWD),
            newname,
            c_uint(0)
        ))
    }
    #[cfg(not(target_arch = "riscv64"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_renameat,
            raw_fd(AT_FDCWD),
            oldname,
            raw_fd(AT_FDCWD),
            newname
        ))
    }
}

#[inline]
pub(crate) fn renameat(
    old_dirfd: BorrowedFd<'_>,
    oldname: &CStr,
    new_dirfd: BorrowedFd<'_>,
    newname: &CStr,
) -> io::Result<()> {
    #[cfg(target_arch = "riscv64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_renameat2,
            old_dirfd,
            oldname,
            new_dirfd,
            newname,
            c_uint(0)
        ))
    }
    #[cfg(not(target_arch = "riscv64"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_renameat,
            old_dirfd,
            oldname,
            new_dirfd,
            newname
        ))
    }
}

#[inline]
pub(crate) fn renameat2(
    old_dirfd: BorrowedFd<'_>,
    oldname: &CStr,
    new_dirfd: BorrowedFd<'_>,
    newname: &CStr,
    flags: RenameFlags,
) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_renameat2,
            old_dirfd,
            oldname,
            new_dirfd,
            newname,
            flags
        ))
    }
}

#[inline]
pub(crate) fn unlink(pathname: &CStr) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_unlinkat,
            raw_fd(AT_FDCWD),
            pathname,
            c_uint(0)
        ))
    }
}

#[inline]
pub(crate) fn unlinkat(dirfd: BorrowedFd<'_>, pathname: &CStr, flags: AtFlags) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_unlinkat, dirfd, pathname, flags)) }
}

#[inline]
pub(crate) fn rmdir(pathname: &CStr) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_unlinkat,
            raw_fd(AT_FDCWD),
            pathname,
            c_uint(AT_REMOVEDIR)
        ))
    }
}

#[inline]
pub(crate) fn link(oldname: &CStr, newname: &CStr) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_linkat,
            raw_fd(AT_FDCWD),
            oldname,
            raw_fd(AT_FDCWD),
            newname,
            c_uint(0)
        ))
    }
}

#[inline]
pub(crate) fn linkat(
    old_dirfd: BorrowedFd<'_>,
    oldname: &CStr,
    new_dirfd: BorrowedFd<'_>,
    newname: &CStr,
    flags: AtFlags,
) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_linkat,
            old_dirfd,
            oldname,
            new_dirfd,
            newname,
            flags
        ))
    }
}

#[inline]
pub(crate) fn symlink(oldname: &CStr, newname: &CStr) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_symlinkat,
            oldname,
            raw_fd(AT_FDCWD),
            newname
        ))
    }
}

#[inline]
pub(crate) fn symlinkat(oldname: &CStr, dirfd: BorrowedFd<'_>, newname: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_symlinkat, oldname, dirfd, newname)) }
}

#[inline]
pub(crate) fn mkdir(pathname: &CStr, mode: Mode) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_mkdirat,
            raw_fd(AT_FDCWD),
            pathname,
            mode
        ))
    }
}

#[inline]
pub(crate) fn mkdirat(dirfd: BorrowedFd<'_>, pathname: &CStr, mode: Mode) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_mkdirat, dirfd, pathname, mode)) }
}

#[inline]
pub(crate) fn getdents(fd: BorrowedFd<'_>, dirent: &mut [u8]) -> io::Result<usize> {
    let (dirent_addr_mut, dirent_len) = slice_mut(dirent);

    unsafe { ret_usize(syscall!(__NR_getdents64, fd, dirent_addr_mut, dirent_len)) }
}

#[inline]
pub(crate) fn getdents_uninit(
    fd: BorrowedFd<'_>,
    dirent: &mut [MaybeUninit<u8>],
) -> io::Result<usize> {
    let (dirent_addr_mut, dirent_len) = slice_mut(dirent);

    unsafe { ret_usize(syscall!(__NR_getdents64, fd, dirent_addr_mut, dirent_len)) }
}

#[inline]
pub(crate) fn utimensat(
    dirfd: BorrowedFd<'_>,
    pathname: &CStr,
    times: &Timestamps,
    flags: AtFlags,
) -> io::Result<()> {
    _utimensat(dirfd, Some(pathname), times, flags)
}

#[inline]
fn _utimensat(
    dirfd: BorrowedFd<'_>,
    pathname: Option<&CStr>,
    times: &Timestamps,
    flags: AtFlags,
) -> io::Result<()> {
    // Assert that `Timestamps` has the expected layout.
    let _ = unsafe { transmute::<Timestamps, [__kernel_timespec; 2]>(times.clone()) };

    // `utimensat_time64` was introduced in Linux 5.1. The old `utimensat`
    // syscall is not y2038-compatible on 32-bit architectures.
    #[cfg(target_pointer_width = "32")]
    unsafe {
        match ret(syscall_readonly!(
            __NR_utimensat_time64,
            dirfd,
            pathname,
            by_ref(times),
            flags
        )) {
            Err(io::Errno::NOSYS) => _utimensat_old(dirfd, pathname, times, flags),
            otherwise => otherwise,
        }
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(
            __NR_utimensat,
            dirfd,
            pathname,
            by_ref(times),
            flags
        ))
    }
}

#[cfg(target_pointer_width = "32")]
unsafe fn _utimensat_old(
    dirfd: BorrowedFd<'_>,
    pathname: Option<&CStr>,
    times: &Timestamps,
    flags: AtFlags,
) -> io::Result<()> {
    // See the comments in `rustix_clock_gettime_via_syscall` about
    // emulation.
    let old_times = [
        __kernel_old_timespec {
            tv_sec: times
                .last_access
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times
                .last_access
                .tv_nsec
                .try_into()
                .map_err(|_| io::Errno::INVAL)?,
        },
        __kernel_old_timespec {
            tv_sec: times
                .last_modification
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times
                .last_modification
                .tv_nsec
                .try_into()
                .map_err(|_| io::Errno::INVAL)?,
        },
    ];
    // The length of the array is fixed and not passed into the syscall.
    let old_times_addr = slice_just_addr(&old_times);
    ret(syscall_readonly!(
        __NR_utimensat,
        dirfd,
        pathname,
        old_times_addr,
        flags
    ))
}

#[inline]
pub(crate) fn futimens(fd: BorrowedFd<'_>, times: &Timestamps) -> io::Result<()> {
    _utimensat(fd, None, times, AtFlags::empty())
}

pub(crate) fn accessat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    access: Access,
    flags: AtFlags,
) -> io::Result<()> {
    if !flags
        .difference(AtFlags::EACCESS | AtFlags::SYMLINK_NOFOLLOW)
        .is_empty()
    {
        return Err(io::Errno::INVAL);
    }

    // Linux's `faccessat` syscall doesn't have a flags argument, so if we have
    // any flags, use the newer `faccessat2` introduced in Linux 5.8 which
    // does. Unless we're on Android where using newer system calls can cause
    // seccomp to abort the process.
    #[cfg(not(target_os = "android"))]
    if !flags.is_empty() {
        unsafe {
            match ret(syscall_readonly!(
                __NR_faccessat2,
                dirfd,
                path,
                access,
                flags
            )) {
                Ok(()) => return Ok(()),
                Err(io::Errno::NOSYS) => {}
                Err(other) => return Err(other),
            }
        }
    }

    // Linux's `faccessat` doesn't have a flags parameter. If we have
    // `AT_EACCESS` and we're not setuid or setgid, we can emulate it.
    if flags.is_empty()
        || (flags.bits() == AT_EACCESS
            && crate::process::getuid() == crate::process::geteuid()
            && crate::process::getgid() == crate::process::getegid())
    {
        return unsafe { ret(syscall_readonly!(__NR_faccessat, dirfd, path, access)) };
    }

    Err(io::Errno::NOSYS)
}

#[inline]
pub(crate) fn copy_file_range(
    fd_in: BorrowedFd<'_>,
    off_in: Option<&mut u64>,
    fd_out: BorrowedFd<'_>,
    off_out: Option<&mut u64>,
    len: usize,
) -> io::Result<usize> {
    unsafe {
        ret_usize(syscall!(
            __NR_copy_file_range,
            fd_in,
            opt_mut(off_in),
            fd_out,
            opt_mut(off_out),
            pass_usize(len),
            c_uint(0)
        ))
    }
}

#[inline]
pub(crate) fn memfd_create(name: &CStr, flags: MemfdFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_memfd_create, name, flags)) }
}

#[inline]
pub(crate) fn sendfile(
    out_fd: BorrowedFd<'_>,
    in_fd: BorrowedFd<'_>,
    offset: Option<&mut u64>,
    count: usize,
) -> io::Result<usize> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall!(
            __NR_sendfile64,
            out_fd,
            in_fd,
            opt_mut(offset),
            pass_usize(count)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall!(
            __NR_sendfile,
            out_fd,
            in_fd,
            opt_mut(offset),
            pass_usize(count)
        ))
    }
}

#[inline]
#[cfg(linux_kernel)]
pub(crate) fn mount(
    source: Option<&CStr>,
    target: &CStr,
    file_system_type: Option<&CStr>,
    flags: super::types::MountFlagsArg,
    data: Option<&CStr>,
) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_mount,
            source,
            target,
            file_system_type,
            flags,
            data
        ))
    }
}

#[inline]
#[cfg(linux_kernel)]
pub(crate) fn unmount(target: &CStr, flags: super::types::UnmountFlags) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_umount2, target, flags)) }
}

#[inline]
pub(crate) fn inotify_init1(flags: inotify::CreateFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_inotify_init1, flags)) }
}

#[inline]
pub(crate) fn inotify_add_watch(
    infd: BorrowedFd<'_>,
    path: &CStr,
    flags: inotify::WatchFlags,
) -> io::Result<i32> {
    unsafe { ret_c_int(syscall_readonly!(__NR_inotify_add_watch, infd, path, flags)) }
}

#[inline]
pub(crate) fn inotify_rm_watch(infd: BorrowedFd<'_>, wfd: i32) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_inotify_rm_watch, infd, c_int(wfd))) }
}

#[inline]
pub(crate) fn getxattr(path: &CStr, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let (value_addr_mut, value_len) = slice_mut(value);
    unsafe {
        ret_usize(syscall!(
            __NR_getxattr,
            path,
            name,
            value_addr_mut,
            value_len
        ))
    }
}

#[inline]
pub(crate) fn lgetxattr(path: &CStr, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let (value_addr_mut, value_len) = slice_mut(value);
    unsafe {
        ret_usize(syscall!(
            __NR_lgetxattr,
            path,
            name,
            value_addr_mut,
            value_len
        ))
    }
}

#[inline]
pub(crate) fn fgetxattr(fd: BorrowedFd<'_>, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let (value_addr_mut, value_len) = slice_mut(value);
    unsafe {
        ret_usize(syscall!(
            __NR_fgetxattr,
            fd,
            name,
            value_addr_mut,
            value_len
        ))
    }
}

#[inline]
pub(crate) fn setxattr(
    path: &CStr,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    let (value_addr, value_len) = slice(value);
    unsafe {
        ret(syscall_readonly!(
            __NR_setxattr,
            path,
            name,
            value_addr,
            value_len,
            flags
        ))
    }
}

#[inline]
pub(crate) fn lsetxattr(
    path: &CStr,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    let (value_addr, value_len) = slice(value);
    unsafe {
        ret(syscall_readonly!(
            __NR_lsetxattr,
            path,
            name,
            value_addr,
            value_len,
            flags
        ))
    }
}

#[inline]
pub(crate) fn fsetxattr(
    fd: BorrowedFd<'_>,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    let (value_addr, value_len) = slice(value);
    unsafe {
        ret(syscall_readonly!(
            __NR_fsetxattr,
            fd,
            name,
            value_addr,
            value_len,
            flags
        ))
    }
}

#[inline]
pub(crate) fn listxattr(path: &CStr, list: &mut [c::c_char]) -> io::Result<usize> {
    let (list_addr_mut, list_len) = slice_mut(list);
    unsafe { ret_usize(syscall!(__NR_listxattr, path, list_addr_mut, list_len)) }
}

#[inline]
pub(crate) fn llistxattr(path: &CStr, list: &mut [c::c_char]) -> io::Result<usize> {
    let (list_addr_mut, list_len) = slice_mut(list);
    unsafe { ret_usize(syscall!(__NR_llistxattr, path, list_addr_mut, list_len)) }
}

#[inline]
pub(crate) fn flistxattr(fd: BorrowedFd<'_>, list: &mut [c::c_char]) -> io::Result<usize> {
    let (list_addr_mut, list_len) = slice_mut(list);
    unsafe { ret_usize(syscall!(__NR_flistxattr, fd, list_addr_mut, list_len)) }
}

#[inline]
pub(crate) fn removexattr(path: &CStr, name: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_removexattr, path, name)) }
}

#[inline]
pub(crate) fn lremovexattr(path: &CStr, name: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_lremovexattr, path, name)) }
}

#[inline]
pub(crate) fn fremovexattr(fd: BorrowedFd<'_>, name: &CStr) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_fremovexattr, fd, name)) }
}
