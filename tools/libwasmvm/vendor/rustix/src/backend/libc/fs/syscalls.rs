//! libc syscalls supporting `rustix::fs`.

use super::super::c;
use super::super::conv::{borrowed_fd, c_str, ret, ret_c_int, ret_off_t, ret_owned_fd, ret_usize};
#[cfg(any(linux_kernel, target_os = "fuchsia"))]
use super::super::offset::libc_fallocate;
#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
use super::super::offset::libc_posix_fadvise;
#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    linux_kernel,
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "redox",
)))]
use super::super::offset::libc_posix_fallocate;
use super::super::offset::{libc_fstat, libc_fstatat, libc_ftruncate, libc_lseek, libc_off_t};
#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
use super::super::offset::{libc_fstatfs, libc_statfs};
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
use super::super::offset::{libc_fstatvfs, libc_statvfs};
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
use super::super::time::types::LibcTimespec;
use crate::fd::{BorrowedFd, OwnedFd};
use crate::ffi::CStr;
#[cfg(apple)]
use crate::ffi::CString;
#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
use crate::fs::Advice;
#[cfg(not(any(
    netbsdlike,
    solarish,
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "redox",
)))]
use crate::fs::FallocateFlags;
#[cfg(not(target_os = "wasi"))]
use crate::fs::FlockOperation;
#[cfg(any(linux_kernel, target_os = "freebsd"))]
use crate::fs::MemfdFlags;
#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "fuchsia"))]
use crate::fs::SealFlags;
#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
use crate::fs::StatFs;
use crate::fs::{Access, Mode, OFlags, Stat, Timestamps};
#[cfg(not(any(apple, target_os = "redox", target_os = "wasi")))]
use crate::fs::{Dev, FileType};
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
use crate::fs::{StatVfs, StatVfsMountFlags};
use crate::io::{self, SeekFrom};
#[cfg(not(target_os = "wasi"))]
use crate::process::{Gid, Uid};
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
use crate::utils::as_ptr;
#[cfg(apple)]
use alloc::vec;
use core::convert::TryInto;
use core::mem::MaybeUninit;
#[cfg(apple)]
use {
    super::super::conv::nonnegative_ret,
    crate::fs::{copyfile_state_t, CloneFlags, CopyfileFlags},
};
#[cfg(linux_kernel)]
use {
    super::super::conv::{syscall_ret, syscall_ret_owned_fd, syscall_ret_usize},
    crate::fs::{cwd, RenameFlags, ResolveFlags, Statx, StatxFlags},
    core::ptr::null,
};
#[cfg(not(target_os = "redox"))]
use {super::super::offset::libc_openat, crate::fs::AtFlags};
#[cfg(any(apple, linux_kernel))]
use {crate::fs::XattrFlags, core::mem::size_of, core::ptr::null_mut};

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
weak!(fn __utimensat64(c::c_int, *const c::c_char, *const LibcTimespec, c::c_int) -> c::c_int);
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
weak!(fn __futimens64(c::c_int, *const LibcTimespec) -> c::c_int);

/// Use a direct syscall (via libc) for `openat`.
///
/// This is only currently necessary as a workaround for old glibc; see below.
#[cfg(all(unix, target_env = "gnu"))]
fn openat_via_syscall(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    oflags: OFlags,
    mode: Mode,
) -> io::Result<OwnedFd> {
    unsafe {
        let dirfd = borrowed_fd(dirfd);
        let path = c_str(path);
        let oflags = oflags.bits();
        let mode = c::c_uint::from(mode.bits());
        ret_owned_fd(c::syscall(
            c::SYS_openat,
            c::c_long::from(dirfd),
            path,
            c::c_long::from(oflags),
            mode as c::c_long,
        ) as c::c_int)
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn openat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    oflags: OFlags,
    mode: Mode,
) -> io::Result<OwnedFd> {
    // Work around <https://sourceware.org/bugzilla/show_bug.cgi?id=17523>.
    // glibc versions before 2.25 don't handle `O_TMPFILE` correctly.
    #[cfg(all(unix, target_env = "gnu"))]
    if oflags.contains(OFlags::TMPFILE) && crate::backend::if_glibc_is_less_than_2_25() {
        return openat_via_syscall(dirfd, path, oflags, mode);
    }
    unsafe {
        // Pass `mode` as a `c_uint` even if `mode_t` is narrower, since
        // `libc_openat` is declared as a variadic function and narrower
        // arguments are promoted.
        ret_owned_fd(libc_openat(
            borrowed_fd(dirfd),
            c_str(path),
            oflags.bits(),
            c::c_uint::from(mode.bits()),
        ))
    }
}

#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
#[inline]
pub(crate) fn statfs(filename: &CStr) -> io::Result<StatFs> {
    unsafe {
        let mut result = MaybeUninit::<StatFs>::uninit();
        ret(libc_statfs(c_str(filename), result.as_mut_ptr()))?;
        Ok(result.assume_init())
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
#[inline]
pub(crate) fn statvfs(filename: &CStr) -> io::Result<StatVfs> {
    unsafe {
        let mut result = MaybeUninit::<libc_statvfs>::uninit();
        ret(libc_statvfs(c_str(filename), result.as_mut_ptr()))?;
        Ok(libc_statvfs_to_statvfs(result.assume_init()))
    }
}

#[cfg(not(target_os = "redox"))]
#[inline]
pub(crate) fn readlinkat(dirfd: BorrowedFd<'_>, path: &CStr, buf: &mut [u8]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::readlinkat(
            borrowed_fd(dirfd),
            c_str(path),
            buf.as_mut_ptr().cast::<c::c_char>(),
            buf.len(),
        ))
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn mkdirat(dirfd: BorrowedFd<'_>, path: &CStr, mode: Mode) -> io::Result<()> {
    unsafe {
        ret(c::mkdirat(
            borrowed_fd(dirfd),
            c_str(path),
            mode.bits() as c::mode_t,
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn getdents_uninit(
    fd: BorrowedFd<'_>,
    buf: &mut [MaybeUninit<u8>],
) -> io::Result<usize> {
    unsafe {
        syscall_ret_usize(c::syscall(
            c::SYS_getdents64,
            fd,
            buf.as_mut_ptr().cast::<c::c_char>(),
            buf.len(),
        ))
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn linkat(
    old_dirfd: BorrowedFd<'_>,
    old_path: &CStr,
    new_dirfd: BorrowedFd<'_>,
    new_path: &CStr,
    flags: AtFlags,
) -> io::Result<()> {
    // macOS <= 10.9 lacks `linkat`.
    #[cfg(target_os = "macos")]
    unsafe {
        weak! {
            fn linkat(
                c::c_int,
                *const c::c_char,
                c::c_int,
                *const c::c_char,
                c::c_int
            ) -> c::c_int
        }
        // If we have `linkat`, use it.
        if let Some(libc_linkat) = linkat.get() {
            return ret(libc_linkat(
                borrowed_fd(old_dirfd),
                c_str(old_path),
                borrowed_fd(new_dirfd),
                c_str(new_path),
                flags.bits(),
            ));
        }
        // Otherwise, see if we can emulate the `AT_FDCWD` case.
        if borrowed_fd(old_dirfd) != c::AT_FDCWD || borrowed_fd(new_dirfd) != c::AT_FDCWD {
            return Err(io::Errno::NOSYS);
        }
        if flags.intersects(!AtFlags::SYMLINK_FOLLOW) {
            return Err(io::Errno::INVAL);
        }
        if !flags.is_empty() {
            return Err(io::Errno::OPNOTSUPP);
        }
        ret(c::link(c_str(old_path), c_str(new_path)))
    }

    #[cfg(not(target_os = "macos"))]
    unsafe {
        ret(c::linkat(
            borrowed_fd(old_dirfd),
            c_str(old_path),
            borrowed_fd(new_dirfd),
            c_str(new_path),
            flags.bits(),
        ))
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn unlinkat(dirfd: BorrowedFd<'_>, path: &CStr, flags: AtFlags) -> io::Result<()> {
    // macOS <= 10.9 lacks `unlinkat`.
    #[cfg(target_os = "macos")]
    unsafe {
        weak! {
            fn unlinkat(
                c::c_int,
                *const c::c_char,
                c::c_int
            ) -> c::c_int
        }
        // If we have `unlinkat`, use it.
        if let Some(libc_unlinkat) = unlinkat.get() {
            return ret(libc_unlinkat(borrowed_fd(dirfd), c_str(path), flags.bits()));
        }
        // Otherwise, see if we can emulate the `AT_FDCWD` case.
        if borrowed_fd(dirfd) != c::AT_FDCWD {
            return Err(io::Errno::NOSYS);
        }
        if flags.intersects(!AtFlags::REMOVEDIR) {
            return Err(io::Errno::INVAL);
        }
        if flags.contains(AtFlags::REMOVEDIR) {
            ret(c::rmdir(c_str(path)))
        } else {
            ret(c::unlink(c_str(path)))
        }
    }

    #[cfg(not(target_os = "macos"))]
    unsafe {
        ret(c::unlinkat(borrowed_fd(dirfd), c_str(path), flags.bits()))
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn renameat(
    old_dirfd: BorrowedFd<'_>,
    old_path: &CStr,
    new_dirfd: BorrowedFd<'_>,
    new_path: &CStr,
) -> io::Result<()> {
    // macOS <= 10.9 lacks `renameat`.
    #[cfg(target_os = "macos")]
    unsafe {
        weak! {
            fn renameat(
                c::c_int,
                *const c::c_char,
                c::c_int,
                *const c::c_char
            ) -> c::c_int
        }
        // If we have `renameat`, use it.
        if let Some(libc_renameat) = renameat.get() {
            return ret(libc_renameat(
                borrowed_fd(old_dirfd),
                c_str(old_path),
                borrowed_fd(new_dirfd),
                c_str(new_path),
            ));
        }
        // Otherwise, see if we can emulate the `AT_FDCWD` case.
        if borrowed_fd(old_dirfd) != c::AT_FDCWD || borrowed_fd(new_dirfd) != c::AT_FDCWD {
            return Err(io::Errno::NOSYS);
        }
        ret(c::rename(c_str(old_path), c_str(new_path)))
    }

    #[cfg(not(target_os = "macos"))]
    unsafe {
        ret(c::renameat(
            borrowed_fd(old_dirfd),
            c_str(old_path),
            borrowed_fd(new_dirfd),
            c_str(new_path),
        ))
    }
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn renameat2(
    old_dirfd: BorrowedFd<'_>,
    old_path: &CStr,
    new_dirfd: BorrowedFd<'_>,
    new_path: &CStr,
    flags: RenameFlags,
) -> io::Result<()> {
    // `renameat2` wasn't supported in glibc until 2.28.
    weak_or_syscall! {
        fn renameat2(
            olddirfd: c::c_int,
            oldpath: *const c::c_char,
            newdirfd: c::c_int,
            newpath: *const c::c_char,
            flags: c::c_uint
        ) via SYS_renameat2 -> c::c_int
    }

    unsafe {
        ret(renameat2(
            borrowed_fd(old_dirfd),
            c_str(old_path),
            borrowed_fd(new_dirfd),
            c_str(new_path),
            flags.bits(),
        ))
    }
}

/// At present, `libc` only has `renameat2` defined for glibc. On other
/// ABIs, `RenameFlags` has no flags defined, and we use plain `renameat`.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
#[inline]
pub(crate) fn renameat2(
    old_dirfd: BorrowedFd<'_>,
    old_path: &CStr,
    new_dirfd: BorrowedFd<'_>,
    new_path: &CStr,
    flags: RenameFlags,
) -> io::Result<()> {
    assert!(flags.is_empty());
    renameat(old_dirfd, old_path, new_dirfd, new_path)
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn symlinkat(
    old_path: &CStr,
    new_dirfd: BorrowedFd<'_>,
    new_path: &CStr,
) -> io::Result<()> {
    unsafe {
        ret(c::symlinkat(
            c_str(old_path),
            borrowed_fd(new_dirfd),
            c_str(new_path),
        ))
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn statat(dirfd: BorrowedFd<'_>, path: &CStr, flags: AtFlags) -> io::Result<Stat> {
    // See the comments in `fstat` about using `crate::fs::statx` here.
    #[cfg(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64")))]
    {
        match crate::fs::statx(dirfd, path, flags, StatxFlags::BASIC_STATS) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => statat_old(dirfd, path, flags),
            Err(err) => Err(err),
        }
    }

    // Main version: libc is y2038 safe. Or, the platform is not y2038 safe and
    // there's nothing practical we can do.
    #[cfg(not(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64"))))]
    unsafe {
        let mut stat = MaybeUninit::<Stat>::uninit();
        ret(libc_fstatat(
            borrowed_fd(dirfd),
            c_str(path),
            stat.as_mut_ptr(),
            flags.bits(),
        ))?;
        Ok(stat.assume_init())
    }
}

#[cfg(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64")))]
fn statat_old(dirfd: BorrowedFd<'_>, path: &CStr, flags: AtFlags) -> io::Result<Stat> {
    unsafe {
        let mut result = MaybeUninit::<c::stat64>::uninit();
        ret(libc_fstatat(
            borrowed_fd(dirfd),
            c_str(path),
            result.as_mut_ptr(),
            flags.bits(),
        ))?;
        stat64_to_stat(result.assume_init())
    }
}

#[cfg(not(any(target_os = "emscripten", target_os = "redox")))]
pub(crate) fn accessat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    access: Access,
    flags: AtFlags,
) -> io::Result<()> {
    // macOS <= 10.9 lacks `faccessat`.
    #[cfg(target_os = "macos")]
    unsafe {
        weak! {
            fn faccessat(
                c::c_int,
                *const c::c_char,
                c::c_int,
                c::c_int
            ) -> c::c_int
        }
        // If we have `faccessat`, use it.
        if let Some(libc_faccessat) = faccessat.get() {
            return ret(libc_faccessat(
                borrowed_fd(dirfd),
                c_str(path),
                access.bits(),
                flags.bits(),
            ));
        }
        // Otherwise, see if we can emulate the `AT_FDCWD` case.
        if borrowed_fd(dirfd) != c::AT_FDCWD {
            return Err(io::Errno::NOSYS);
        }
        if flags.intersects(!(AtFlags::EACCESS | AtFlags::SYMLINK_NOFOLLOW)) {
            return Err(io::Errno::INVAL);
        }
        if !flags.is_empty() {
            return Err(io::Errno::OPNOTSUPP);
        }
        ret(c::access(c_str(path), access.bits()))
    }

    #[cfg(not(target_os = "macos"))]
    unsafe {
        ret(c::faccessat(
            borrowed_fd(dirfd),
            c_str(path),
            access.bits(),
            flags.bits(),
        ))
    }
}

#[cfg(target_os = "emscripten")]
pub(crate) fn accessat(
    _dirfd: BorrowedFd<'_>,
    _path: &CStr,
    _access: Access,
    _flags: AtFlags,
) -> io::Result<()> {
    Ok(())
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn utimensat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    times: &Timestamps,
    flags: AtFlags,
) -> io::Result<()> {
    // 32-bit gnu version: libc has `utimensat` but it is not y2038 safe by
    // default.
    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_utimensat) = __utimensat64.get() {
            let libc_times: [LibcTimespec; 2] = [
                times.last_access.clone().into(),
                times.last_modification.clone().into(),
            ];

            ret(libc_utimensat(
                borrowed_fd(dirfd),
                c_str(path),
                libc_times.as_ptr(),
                flags.bits(),
            ))
        } else {
            utimensat_old(dirfd, path, times, flags)
        }
    }

    // Main version: libc is y2038 safe and has `utimensat`. Or, the platform
    // is not y2038 safe and there's nothing practical we can do.
    #[cfg(not(any(
        apple,
        all(
            any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
            target_env = "gnu",
        )
    )))]
    unsafe {
        // Assert that `Timestamps` has the expected layout.
        let _ = core::mem::transmute::<Timestamps, [c::timespec; 2]>(times.clone());

        ret(c::utimensat(
            borrowed_fd(dirfd),
            c_str(path),
            as_ptr(times).cast(),
            flags.bits(),
        ))
    }

    // `utimensat` was introduced in macOS 10.13.
    #[cfg(apple)]
    unsafe {
        // ABI details
        weak! {
            fn utimensat(
                c::c_int,
                *const c::c_char,
                *const c::timespec,
                c::c_int
            ) -> c::c_int
        }
        extern "C" {
            fn setattrlist(
                path: *const c::c_char,
                attr_list: *const Attrlist,
                attr_buf: *const c::c_void,
                attr_buf_size: c::size_t,
                options: c::c_ulong,
            ) -> c::c_int;
        }
        const FSOPT_NOFOLLOW: c::c_ulong = 0x0000_0001;

        // If we have `utimensat`, use it.
        if let Some(have_utimensat) = utimensat.get() {
            // Assert that `Timestamps` has the expected layout.
            let _ = core::mem::transmute::<Timestamps, [c::timespec; 2]>(times.clone());

            return ret(have_utimensat(
                borrowed_fd(dirfd),
                c_str(path),
                as_ptr(times).cast(),
                flags.bits(),
            ));
        }

        // `setattrlistat` was introduced in 10.13 along with `utimensat`, so if
        // we don't have `utimensat`, we don't have `setattrlistat` either.
        // Emulate it using `fork`, and `fchdir` and [`setattrlist`].
        //
        // [`setattrlist`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setattrlist.2.html
        match c::fork() {
            -1 => Err(io::Errno::IO),
            0 => {
                if c::fchdir(borrowed_fd(dirfd)) != 0 {
                    let code = match libc_errno::errno().0 {
                        c::EACCES => 2,
                        c::ENOTDIR => 3,
                        _ => 1,
                    };
                    c::_exit(code);
                }

                let mut flags_arg = 0;
                if flags.contains(AtFlags::SYMLINK_NOFOLLOW) {
                    flags_arg |= FSOPT_NOFOLLOW;
                }

                let (attrbuf_size, times, attrs) = times_to_attrlist(times);

                if setattrlist(
                    c_str(path),
                    &attrs,
                    as_ptr(&times).cast(),
                    attrbuf_size,
                    flags_arg,
                ) != 0
                {
                    // Translate expected `errno` codes into ad-hoc integer
                    // values suitable for exit statuses.
                    let code = match libc_errno::errno().0 {
                        c::EACCES => 2,
                        c::ENOTDIR => 3,
                        c::EPERM => 4,
                        c::EROFS => 5,
                        c::ELOOP => 6,
                        c::ENOENT => 7,
                        c::ENAMETOOLONG => 8,
                        c::EINVAL => 9,
                        c::ESRCH => 10,
                        c::ENOTSUP => 11,
                        _ => 1,
                    };
                    c::_exit(code);
                }

                c::_exit(0);
            }
            child_pid => {
                let mut wstatus = 0;
                let _ = ret_c_int(c::waitpid(child_pid, &mut wstatus, 0))?;
                if c::WIFEXITED(wstatus) {
                    // Translate our ad-hoc exit statuses back to `errno`
                    // codes.
                    match c::WEXITSTATUS(wstatus) {
                        0 => Ok(()),
                        2 => Err(io::Errno::ACCESS),
                        3 => Err(io::Errno::NOTDIR),
                        4 => Err(io::Errno::PERM),
                        5 => Err(io::Errno::ROFS),
                        6 => Err(io::Errno::LOOP),
                        7 => Err(io::Errno::NOENT),
                        8 => Err(io::Errno::NAMETOOLONG),
                        9 => Err(io::Errno::INVAL),
                        10 => Err(io::Errno::SRCH),
                        11 => Err(io::Errno::NOTSUP),
                        _ => Err(io::Errno::IO),
                    }
                } else {
                    Err(io::Errno::IO)
                }
            }
        }
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
unsafe fn utimensat_old(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    times: &Timestamps,
    flags: AtFlags,
) -> io::Result<()> {
    let old_times = [
        c::timespec {
            tv_sec: times
                .last_access
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times.last_access.tv_nsec,
        },
        c::timespec {
            tv_sec: times
                .last_modification
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times.last_modification.tv_nsec,
        },
    ];
    ret(c::utimensat(
        borrowed_fd(dirfd),
        c_str(path),
        old_times.as_ptr(),
        flags.bits(),
    ))
}

#[cfg(not(any(linux_kernel, target_os = "redox", target_os = "wasi")))]
pub(crate) fn chmodat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    mode: Mode,
    flags: AtFlags,
) -> io::Result<()> {
    unsafe {
        ret(c::fchmodat(
            borrowed_fd(dirfd),
            c_str(path),
            mode.bits() as c::mode_t,
            flags.bits(),
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn chmodat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    mode: Mode,
    flags: AtFlags,
) -> io::Result<()> {
    // Linux's `fchmodat` does not have a flags argument.
    //
    // Use `c::syscall` rather than `c::fchmodat` because some libc
    // implementations, such as musl, add extra logic to `fchmod` to emulate
    // support for `AT_SYMLINK_NOFOLLOW`, which uses `/proc` outside our
    // control.
    if flags == AtFlags::SYMLINK_NOFOLLOW {
        return Err(io::Errno::OPNOTSUPP);
    }
    if !flags.is_empty() {
        return Err(io::Errno::INVAL);
    }
    unsafe {
        // Pass `mode` as a `c_uint` even if `mode_t` is narrower, since
        // `libc_openat` is declared as a variadic function and narrower
        // arguments are promoted.
        syscall_ret(c::syscall(
            c::SYS_fchmodat,
            borrowed_fd(dirfd),
            c_str(path),
            c::c_uint::from(mode.bits()),
        ))
    }
}

#[cfg(apple)]
pub(crate) fn fclonefileat(
    srcfd: BorrowedFd<'_>,
    dst_dirfd: BorrowedFd<'_>,
    dst: &CStr,
    flags: CloneFlags,
) -> io::Result<()> {
    syscall! {
        fn fclonefileat(
            srcfd: BorrowedFd<'_>,
            dst_dirfd: BorrowedFd<'_>,
            dst: *const c::c_char,
            flags: c::c_int
        ) via SYS_fclonefileat -> c::c_int
    }

    unsafe { ret(fclonefileat(srcfd, dst_dirfd, c_str(dst), flags.bits())) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn chownat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    owner: Option<Uid>,
    group: Option<Gid>,
    flags: AtFlags,
) -> io::Result<()> {
    unsafe {
        let (ow, gr) = crate::process::translate_fchown_args(owner, group);
        ret(c::fchownat(
            borrowed_fd(dirfd),
            c_str(path),
            ow,
            gr,
            flags.bits(),
        ))
    }
}

#[cfg(not(any(apple, target_os = "redox", target_os = "wasi")))]
pub(crate) fn mknodat(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    file_type: FileType,
    mode: Mode,
    dev: Dev,
) -> io::Result<()> {
    unsafe {
        ret(c::mknodat(
            borrowed_fd(dirfd),
            c_str(path),
            (mode.bits() | file_type.as_raw_mode()) as c::mode_t,
            dev.try_into().map_err(|_e| io::Errno::PERM)?,
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn copy_file_range(
    fd_in: BorrowedFd<'_>,
    off_in: Option<&mut u64>,
    fd_out: BorrowedFd<'_>,
    off_out: Option<&mut u64>,
    len: usize,
) -> io::Result<usize> {
    assert_eq!(size_of::<c::loff_t>(), size_of::<u64>());

    let mut off_in_val: c::loff_t = 0;
    let mut off_out_val: c::loff_t = 0;
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let off_in_ptr = if let Some(off_in) = &off_in {
        off_in_val = (**off_in) as i64;
        &mut off_in_val
    } else {
        null_mut()
    };
    let off_out_ptr = if let Some(off_out) = &off_out {
        off_out_val = (**off_out) as i64;
        &mut off_out_val
    } else {
        null_mut()
    };
    let copied = unsafe {
        syscall_ret_usize(c::syscall(
            c::SYS_copy_file_range,
            borrowed_fd(fd_in),
            off_in_ptr,
            borrowed_fd(fd_out),
            off_out_ptr,
            len,
            0, // no flags are defined yet
        ))?
    };
    if let Some(off_in) = off_in {
        *off_in = off_in_val as u64;
    }
    if let Some(off_out) = off_out {
        *off_out = off_out_val as u64;
    }
    Ok(copied)
}

#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
pub(crate) fn fadvise(fd: BorrowedFd<'_>, offset: u64, len: u64, advice: Advice) -> io::Result<()> {
    let offset = offset as i64;
    let len = len as i64;

    // FreeBSD returns `EINVAL` on invalid offsets; emulate the POSIX behavior.
    #[cfg(target_os = "freebsd")]
    let offset = if (offset as i64) < 0 {
        i64::MAX
    } else {
        offset
    };

    // FreeBSD returns `EINVAL` on overflow; emulate the POSIX behavior.
    #[cfg(target_os = "freebsd")]
    let len = if len > 0 && offset.checked_add(len).is_none() {
        i64::MAX - offset
    } else {
        len
    };

    let err = unsafe { libc_posix_fadvise(borrowed_fd(fd), offset, len, advice as c::c_int) };

    // `posix_fadvise` returns its error status rather than using `errno`.
    if err == 0 {
        Ok(())
    } else {
        Err(io::Errno(err))
    }
}

pub(crate) fn fcntl_getfl(fd: BorrowedFd<'_>) -> io::Result<OFlags> {
    unsafe { ret_c_int(c::fcntl(borrowed_fd(fd), c::F_GETFL)).map(OFlags::from_bits_truncate) }
}

pub(crate) fn fcntl_setfl(fd: BorrowedFd<'_>, flags: OFlags) -> io::Result<()> {
    unsafe { ret(c::fcntl(borrowed_fd(fd), c::F_SETFL, flags.bits())) }
}

#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "fuchsia"))]
pub(crate) fn fcntl_get_seals(fd: BorrowedFd<'_>) -> io::Result<SealFlags> {
    unsafe {
        ret_c_int(c::fcntl(borrowed_fd(fd), c::F_GET_SEALS))
            .map(|flags| SealFlags::from_bits_unchecked(flags))
    }
}

#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "fuchsia"))]
pub(crate) fn fcntl_add_seals(fd: BorrowedFd<'_>, seals: SealFlags) -> io::Result<()> {
    unsafe { ret(c::fcntl(borrowed_fd(fd), c::F_ADD_SEALS, seals.bits())) }
}

#[cfg(not(any(
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "redox",
    target_os = "wasi"
)))]
#[inline]
pub(crate) fn fcntl_lock(fd: BorrowedFd<'_>, operation: FlockOperation) -> io::Result<()> {
    use c::{flock, F_RDLCK, F_SETLK, F_SETLKW, F_UNLCK, F_WRLCK, SEEK_SET};

    let (cmd, l_type) = match operation {
        FlockOperation::LockShared => (F_SETLKW, F_RDLCK),
        FlockOperation::LockExclusive => (F_SETLKW, F_WRLCK),
        FlockOperation::Unlock => (F_SETLKW, F_UNLCK),
        FlockOperation::NonBlockingLockShared => (F_SETLK, F_RDLCK),
        FlockOperation::NonBlockingLockExclusive => (F_SETLK, F_WRLCK),
        FlockOperation::NonBlockingUnlock => (F_SETLK, F_UNLCK),
    };

    unsafe {
        let mut lock: flock = core::mem::zeroed();
        lock.l_type = l_type as _;

        // When `l_len` is zero, this locks all the bytes from
        // `l_whence`/`l_start` to the end of the file, even as the
        // file grows dynamically.
        lock.l_whence = SEEK_SET as _;
        lock.l_start = 0;
        lock.l_len = 0;

        ret(c::fcntl(borrowed_fd(fd), cmd, &lock))
    }
}

pub(crate) fn seek(fd: BorrowedFd<'_>, pos: SeekFrom) -> io::Result<u64> {
    let (whence, offset): (c::c_int, libc_off_t) = match pos {
        SeekFrom::Start(pos) => {
            let pos: u64 = pos;
            // Silently cast; we'll get `EINVAL` if the value is negative.
            (c::SEEK_SET, pos as i64)
        }
        SeekFrom::End(offset) => (c::SEEK_END, offset),
        SeekFrom::Current(offset) => (c::SEEK_CUR, offset),
        #[cfg(any(freebsdlike, target_os = "linux", target_os = "solaris"))]
        SeekFrom::Data(offset) => (c::SEEK_DATA, offset),
        #[cfg(any(freebsdlike, target_os = "linux", target_os = "solaris"))]
        SeekFrom::Hole(offset) => (c::SEEK_HOLE, offset),
    };
    let offset = unsafe { ret_off_t(libc_lseek(borrowed_fd(fd), offset, whence))? };
    Ok(offset as u64)
}

pub(crate) fn tell(fd: BorrowedFd<'_>) -> io::Result<u64> {
    let offset = unsafe { ret_off_t(libc_lseek(borrowed_fd(fd), 0, c::SEEK_CUR))? };
    Ok(offset as u64)
}

#[cfg(not(any(linux_kernel, target_os = "wasi")))]
pub(crate) fn fchmod(fd: BorrowedFd<'_>, mode: Mode) -> io::Result<()> {
    unsafe { ret(c::fchmod(borrowed_fd(fd), mode.bits())) }
}

#[cfg(linux_kernel)]
pub(crate) fn fchmod(fd: BorrowedFd<'_>, mode: Mode) -> io::Result<()> {
    // Use `c::syscall` rather than `c::fchmod` because some libc
    // implementations, such as musl, add extra logic to `fchmod` to emulate
    // support for `O_PATH`, which uses `/proc` outside our control and
    // interferes with our own use of `O_PATH`.
    unsafe {
        syscall_ret(c::syscall(
            c::SYS_fchmod,
            borrowed_fd(fd),
            c::c_uint::from(mode.bits()),
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn fchown(fd: BorrowedFd<'_>, owner: Option<Uid>, group: Option<Gid>) -> io::Result<()> {
    // Use `c::syscall` rather than `c::fchown` because some libc
    // implementations, such as musl, add extra logic to `fchown` to emulate
    // support for `O_PATH`, which uses `/proc` outside our control and
    // interferes with our own use of `O_PATH`.
    unsafe {
        let (ow, gr) = crate::process::translate_fchown_args(owner, group);
        syscall_ret(c::syscall(c::SYS_fchown, borrowed_fd(fd), ow, gr))
    }
}

#[cfg(not(any(linux_kernel, target_os = "wasi")))]
pub(crate) fn fchown(fd: BorrowedFd<'_>, owner: Option<Uid>, group: Option<Gid>) -> io::Result<()> {
    unsafe {
        let (ow, gr) = crate::process::translate_fchown_args(owner, group);
        ret(c::fchown(borrowed_fd(fd), ow, gr))
    }
}

#[cfg(not(any(target_os = "solaris", target_os = "wasi")))]
pub(crate) fn flock(fd: BorrowedFd<'_>, operation: FlockOperation) -> io::Result<()> {
    unsafe { ret(c::flock(borrowed_fd(fd), operation as c::c_int)) }
}

#[cfg(linux_kernel)]
pub(crate) fn syncfs(fd: BorrowedFd<'_>) -> io::Result<()> {
    // Some versions of Android libc lack a `syncfs` function.
    #[cfg(target_os = "android")]
    syscall! {
        fn syncfs(fd: c::c_int) via SYS_syncfs -> c::c_int
    }

    // `syncfs` was added to glibc in 2.20.
    #[cfg(not(target_os = "android"))]
    weak_or_syscall! {
        fn syncfs(fd: c::c_int) via SYS_syncfs -> c::c_int
    }

    unsafe { ret(syncfs(borrowed_fd(fd))) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn sync() {
    unsafe { c::sync() }
}

pub(crate) fn fstat(fd: BorrowedFd<'_>) -> io::Result<Stat> {
    // 32-bit and mips64 Linux: `struct stat64` is not y2038 compatible; use
    // `statx`.
    //
    // And, some old platforms don't support `statx`, and some fail with a
    // confusing error code, so we call `crate::fs::statx` to handle that. If
    // `statx` isn't available, fall back to the buggy system call.
    #[cfg(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64")))]
    {
        match crate::fs::statx(fd, cstr!(""), AtFlags::EMPTY_PATH, StatxFlags::BASIC_STATS) {
            Ok(x) => statx_to_stat(x),
            Err(io::Errno::NOSYS) => fstat_old(fd),
            Err(err) => Err(err),
        }
    }

    // Main version: libc is y2038 safe. Or, the platform is not y2038 safe and
    // there's nothing practical we can do.
    #[cfg(not(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64"))))]
    unsafe {
        let mut stat = MaybeUninit::<Stat>::uninit();
        ret(libc_fstat(borrowed_fd(fd), stat.as_mut_ptr()))?;
        Ok(stat.assume_init())
    }
}

#[cfg(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64")))]
fn fstat_old(fd: BorrowedFd<'_>) -> io::Result<Stat> {
    unsafe {
        let mut result = MaybeUninit::<c::stat64>::uninit();
        ret(libc_fstat(borrowed_fd(fd), result.as_mut_ptr()))?;
        stat64_to_stat(result.assume_init())
    }
}

#[cfg(not(any(
    solarish,
    target_os = "haiku",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(crate) fn fstatfs(fd: BorrowedFd<'_>) -> io::Result<StatFs> {
    let mut statfs = MaybeUninit::<StatFs>::uninit();
    unsafe {
        ret(libc_fstatfs(borrowed_fd(fd), statfs.as_mut_ptr()))?;
        Ok(statfs.assume_init())
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub(crate) fn fstatvfs(fd: BorrowedFd<'_>) -> io::Result<StatVfs> {
    let mut statvfs = MaybeUninit::<libc_statvfs>::uninit();
    unsafe {
        ret(libc_fstatvfs(borrowed_fd(fd), statvfs.as_mut_ptr()))?;
        Ok(libc_statvfs_to_statvfs(statvfs.assume_init()))
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
fn libc_statvfs_to_statvfs(from: libc_statvfs) -> StatVfs {
    StatVfs {
        f_bsize: from.f_bsize as u64,
        f_frsize: from.f_frsize as u64,
        f_blocks: from.f_blocks as u64,
        f_bfree: from.f_bfree as u64,
        f_bavail: from.f_bavail as u64,
        f_files: from.f_files as u64,
        f_ffree: from.f_ffree as u64,
        f_favail: from.f_ffree as u64,
        f_fsid: from.f_fsid as u64,
        f_flag: unsafe { StatVfsMountFlags::from_bits_unchecked(from.f_flag as u64) },
        f_namemax: from.f_namemax as u64,
    }
}

pub(crate) fn futimens(fd: BorrowedFd<'_>, times: &Timestamps) -> io::Result<()> {
    // 32-bit gnu version: libc has `futimens` but it is not y2038 safe by default.
    #[cfg(all(
        any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
        target_env = "gnu",
    ))]
    unsafe {
        if let Some(libc_futimens) = __futimens64.get() {
            let libc_times: [LibcTimespec; 2] = [
                times.last_access.clone().into(),
                times.last_modification.clone().into(),
            ];

            ret(libc_futimens(borrowed_fd(fd), libc_times.as_ptr()))
        } else {
            futimens_old(fd, times)
        }
    }

    // Main version: libc is y2038 safe and has `futimens`. Or, the platform
    // is not y2038 safe and there's nothing practical we can do.
    #[cfg(not(any(
        apple,
        all(
            any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
            target_env = "gnu",
        )
    )))]
    unsafe {
        // Assert that `Timestamps` has the expected layout.
        let _ = core::mem::transmute::<Timestamps, [c::timespec; 2]>(times.clone());

        ret(c::futimens(borrowed_fd(fd), as_ptr(times).cast()))
    }

    // `futimens` was introduced in macOS 10.13.
    #[cfg(apple)]
    unsafe {
        // ABI details.
        weak! {
            fn futimens(c::c_int, *const c::timespec) -> c::c_int
        }
        extern "C" {
            fn fsetattrlist(
                fd: c::c_int,
                attr_list: *const Attrlist,
                attr_buf: *const c::c_void,
                attr_buf_size: c::size_t,
                options: c::c_ulong,
            ) -> c::c_int;
        }

        // If we have `futimens`, use it.
        if let Some(have_futimens) = futimens.get() {
            // Assert that `Timestamps` has the expected layout.
            let _ = core::mem::transmute::<Timestamps, [c::timespec; 2]>(times.clone());

            return ret(have_futimens(borrowed_fd(fd), as_ptr(times).cast()));
        }

        // Otherwise use `fsetattrlist`.
        let (attrbuf_size, times, attrs) = times_to_attrlist(times);

        ret(fsetattrlist(
            borrowed_fd(fd),
            &attrs,
            as_ptr(&times).cast(),
            attrbuf_size,
            0,
        ))
    }
}

#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
unsafe fn futimens_old(fd: BorrowedFd<'_>, times: &Timestamps) -> io::Result<()> {
    let old_times = [
        c::timespec {
            tv_sec: times
                .last_access
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times.last_access.tv_nsec,
        },
        c::timespec {
            tv_sec: times
                .last_modification
                .tv_sec
                .try_into()
                .map_err(|_| io::Errno::OVERFLOW)?,
            tv_nsec: times.last_modification.tv_nsec,
        },
    ];

    ret(c::futimens(borrowed_fd(fd), old_times.as_ptr()))
}

#[cfg(not(any(
    apple,
    netbsdlike,
    solarish,
    target_os = "aix",
    target_os = "dragonfly",
    target_os = "redox",
)))]
pub(crate) fn fallocate(
    fd: BorrowedFd<'_>,
    mode: FallocateFlags,
    offset: u64,
    len: u64,
) -> io::Result<()> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    let len = len as i64;

    #[cfg(any(linux_kernel, target_os = "fuchsia"))]
    unsafe {
        ret(libc_fallocate(borrowed_fd(fd), mode.bits(), offset, len))
    }

    #[cfg(not(any(linux_kernel, target_os = "fuchsia")))]
    {
        assert!(mode.is_empty());
        let err = unsafe { libc_posix_fallocate(borrowed_fd(fd), offset, len) };

        // `posix_fallocate` returns its error status rather than using `errno`.
        if err == 0 {
            Ok(())
        } else {
            Err(io::Errno(err))
        }
    }
}

#[cfg(apple)]
pub(crate) fn fallocate(
    fd: BorrowedFd<'_>,
    mode: FallocateFlags,
    offset: u64,
    len: u64,
) -> io::Result<()> {
    let offset: i64 = offset.try_into().map_err(|_e| io::Errno::INVAL)?;
    let len = len as i64;

    assert!(mode.is_empty());

    let new_len = offset.checked_add(len).ok_or(io::Errno::FBIG)?;
    let mut store = c::fstore_t {
        fst_flags: c::F_ALLOCATECONTIG,
        fst_posmode: c::F_PEOFPOSMODE,
        fst_offset: 0,
        fst_length: new_len,
        fst_bytesalloc: 0,
    };
    unsafe {
        if c::fcntl(borrowed_fd(fd), c::F_PREALLOCATE, &store) == -1 {
            // Unable to allocate contiguous disk space; attempt to allocate
            // non-contiguously.
            store.fst_flags = c::F_ALLOCATEALL;
            let _ = ret_c_int(c::fcntl(borrowed_fd(fd), c::F_PREALLOCATE, &store))?;
        }
        ret(c::ftruncate(borrowed_fd(fd), new_len))
    }
}

pub(crate) fn fsync(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::fsync(borrowed_fd(fd))) }
}

#[cfg(not(any(
    apple,
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
)))]
pub(crate) fn fdatasync(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::fdatasync(borrowed_fd(fd))) }
}

pub(crate) fn ftruncate(fd: BorrowedFd<'_>, length: u64) -> io::Result<()> {
    let length = length.try_into().map_err(|_overflow_err| io::Errno::FBIG)?;
    unsafe { ret(libc_ftruncate(borrowed_fd(fd), length)) }
}

#[cfg(any(linux_kernel, target_os = "freebsd"))]
pub(crate) fn memfd_create(path: &CStr, flags: MemfdFlags) -> io::Result<OwnedFd> {
    #[cfg(target_os = "freebsd")]
    weakcall! {
        fn memfd_create(
            name: *const c::c_char,
            flags: c::c_uint
        ) -> c::c_int
    }

    #[cfg(linux_kernel)]
    weak_or_syscall! {
        fn memfd_create(
            name: *const c::c_char,
            flags: c::c_uint
        ) via SYS_memfd_create -> c::c_int
    }

    unsafe { ret_owned_fd(memfd_create(c_str(path), flags.bits())) }
}

#[cfg(linux_kernel)]
pub(crate) fn openat2(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    oflags: OFlags,
    mode: Mode,
    resolve: ResolveFlags,
) -> io::Result<OwnedFd> {
    let oflags: i32 = oflags.bits();
    let open_how = OpenHow {
        oflag: u64::from(oflags as u32),
        mode: u64::from(mode.bits()),
        resolve: resolve.bits(),
    };

    unsafe {
        syscall_ret_owned_fd(c::syscall(
            SYS_OPENAT2,
            borrowed_fd(dirfd),
            c_str(path),
            &open_how,
            SIZEOF_OPEN_HOW,
        ))
    }
}
#[cfg(all(linux_kernel, target_pointer_width = "32"))]
const SYS_OPENAT2: i32 = 437;
#[cfg(all(linux_kernel, target_pointer_width = "64"))]
const SYS_OPENAT2: i64 = 437;

#[cfg(linux_kernel)]
#[repr(C)]
#[derive(Debug)]
struct OpenHow {
    oflag: u64,
    mode: u64,
    resolve: u64,
}
#[cfg(linux_kernel)]
const SIZEOF_OPEN_HOW: usize = size_of::<OpenHow>();

#[cfg(target_os = "linux")]
pub(crate) fn sendfile(
    out_fd: BorrowedFd<'_>,
    in_fd: BorrowedFd<'_>,
    offset: Option<&mut u64>,
    count: usize,
) -> io::Result<usize> {
    unsafe {
        ret_usize(c::sendfile64(
            borrowed_fd(out_fd),
            borrowed_fd(in_fd),
            offset.map_or(null_mut(), crate::utils::as_mut_ptr).cast(),
            count,
        ))
    }
}

/// Convert from a Linux `statx` value to rustix's `Stat`.
#[cfg(all(linux_kernel, target_pointer_width = "32"))]
fn statx_to_stat(x: crate::fs::Statx) -> io::Result<Stat> {
    Ok(Stat {
        st_dev: crate::fs::makedev(x.stx_dev_major, x.stx_dev_minor).into(),
        st_mode: x.stx_mode.into(),
        st_nlink: x.stx_nlink.into(),
        st_uid: x.stx_uid.into(),
        st_gid: x.stx_gid.into(),
        st_rdev: crate::fs::makedev(x.stx_rdev_major, x.stx_rdev_minor).into(),
        st_size: x.stx_size.try_into().map_err(|_| io::Errno::OVERFLOW)?,
        st_blksize: x.stx_blksize.into(),
        st_blocks: x.stx_blocks.into(),
        st_atime: x
            .stx_atime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_atime_nsec: x.stx_atime.tv_nsec as _,
        st_mtime: x
            .stx_mtime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_mtime_nsec: x.stx_mtime.tv_nsec as _,
        st_ctime: x
            .stx_ctime
            .tv_sec
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        st_ctime_nsec: x.stx_ctime.tv_nsec as _,
        st_ino: x.stx_ino.into(),
    })
}

/// Convert from a Linux `statx` value to rustix's `Stat`.
///
/// mips64' `struct stat64` in libc has private fields, and `stx_blocks`
#[cfg(all(linux_kernel, target_arch = "mips64"))]
fn statx_to_stat(x: crate::fs::Statx) -> io::Result<Stat> {
    let mut result: Stat = unsafe { core::mem::zeroed() };

    result.st_dev = crate::fs::makedev(x.stx_dev_major, x.stx_dev_minor);
    result.st_mode = x.stx_mode.into();
    result.st_nlink = x.stx_nlink.into();
    result.st_uid = x.stx_uid.into();
    result.st_gid = x.stx_gid.into();
    result.st_rdev = crate::fs::makedev(x.stx_rdev_major, x.stx_rdev_minor);
    result.st_size = x.stx_size.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_blksize = x.stx_blksize.into();
    result.st_blocks = x.stx_blocks.try_into().map_err(|_e| io::Errno::OVERFLOW)?;
    result.st_atime = x
        .stx_atime
        .tv_sec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_atime_nsec = x.stx_atime.tv_nsec as _;
    result.st_mtime = x
        .stx_mtime
        .tv_sec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_mtime_nsec = x.stx_mtime.tv_nsec as _;
    result.st_ctime = x
        .stx_ctime
        .tv_sec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_ctime_nsec = x.stx_ctime.tv_nsec as _;
    result.st_ino = x.stx_ino.into();

    Ok(result)
}

/// Convert from a Linux `stat64` value to rustix's `Stat`.
#[cfg(all(linux_kernel, target_pointer_width = "32"))]
fn stat64_to_stat(s64: c::stat64) -> io::Result<Stat> {
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

/// Convert from a Linux `stat64` value to rustix's `Stat`.
///
/// mips64' `struct stat64` in libc has private fields, and `st_blocks` has
/// type `i64`.
#[cfg(all(linux_kernel, target_arch = "mips64"))]
fn stat64_to_stat(s64: c::stat64) -> io::Result<Stat> {
    let mut result: Stat = unsafe { core::mem::zeroed() };

    result.st_dev = s64.st_dev.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_mode = s64.st_mode.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_nlink = s64.st_nlink.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_uid = s64.st_uid.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_gid = s64.st_gid.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_rdev = s64.st_rdev.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_size = s64.st_size.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_blksize = s64.st_blksize.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_blocks = s64.st_blocks.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_atime = s64.st_atime.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_atime_nsec = s64
        .st_atime_nsec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_mtime = s64.st_mtime.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_mtime_nsec = s64
        .st_mtime_nsec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_ctime = s64.st_ctime.try_into().map_err(|_| io::Errno::OVERFLOW)?;
    result.st_ctime_nsec = s64
        .st_ctime_nsec
        .try_into()
        .map_err(|_| io::Errno::OVERFLOW)?;
    result.st_ino = s64.st_ino.try_into().map_err(|_| io::Errno::OVERFLOW)?;

    Ok(result)
}

#[cfg(linux_kernel)]
#[allow(non_upper_case_globals)]
mod sys {
    use super::{c, BorrowedFd, Statx};

    weak_or_syscall! {
        pub(super) fn statx(
            dirfd_: BorrowedFd<'_>,
            path: *const c::c_char,
            flags: c::c_int,
            mask: c::c_uint,
            buf: *mut Statx
        ) via SYS_statx -> c::c_int
    }
}

#[cfg(linux_kernel)]
#[allow(non_upper_case_globals)]
pub(crate) fn statx(
    dirfd: BorrowedFd<'_>,
    path: &CStr,
    flags: AtFlags,
    mask: StatxFlags,
) -> io::Result<Statx> {
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
    #[cfg(not(any(target_os = "android", target_env = "musl")))]
    const STATX__RESERVED: u32 = c::STATX__RESERVED as u32;
    #[cfg(any(target_os = "android", target_env = "musl"))]
    const STATX__RESERVED: u32 = linux_raw_sys::general::STATX__RESERVED;
    if (mask.bits() & STATX__RESERVED) == STATX__RESERVED {
        return Err(io::Errno::INVAL);
    }
    let mask = mask & StatxFlags::all();

    let mut statx_buf = MaybeUninit::<Statx>::uninit();
    unsafe {
        ret(sys::statx(
            dirfd,
            c_str(path),
            flags.bits(),
            mask.bits(),
            statx_buf.as_mut_ptr(),
        ))?;
        Ok(statx_buf.assume_init())
    }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) fn is_statx_available() -> bool {
    unsafe {
        // Call `statx` with null pointers so that if it fails for any reason
        // other than `EFAULT`, we know it's not supported.
        matches!(
            ret(sys::statx(cwd(), null(), 0, 0, null_mut())),
            Err(io::Errno::FAULT)
        )
    }
}

#[cfg(apple)]
pub(crate) unsafe fn fcopyfile(
    from: BorrowedFd<'_>,
    to: BorrowedFd<'_>,
    state: copyfile_state_t,
    flags: CopyfileFlags,
) -> io::Result<()> {
    extern "C" {
        fn fcopyfile(
            from: c::c_int,
            to: c::c_int,
            state: copyfile_state_t,
            flags: c::c_uint,
        ) -> c::c_int;
    }

    nonnegative_ret(fcopyfile(
        borrowed_fd(from),
        borrowed_fd(to),
        state,
        flags.bits(),
    ))
}

#[cfg(apple)]
pub(crate) fn copyfile_state_alloc() -> io::Result<copyfile_state_t> {
    extern "C" {
        fn copyfile_state_alloc() -> copyfile_state_t;
    }

    let result = unsafe { copyfile_state_alloc() };
    if result.0.is_null() {
        Err(io::Errno::last_os_error())
    } else {
        Ok(result)
    }
}

#[cfg(apple)]
pub(crate) unsafe fn copyfile_state_free(state: copyfile_state_t) -> io::Result<()> {
    extern "C" {
        fn copyfile_state_free(state: copyfile_state_t) -> c::c_int;
    }

    nonnegative_ret(copyfile_state_free(state))
}

#[cfg(apple)]
const COPYFILE_STATE_COPIED: u32 = 8;

#[cfg(apple)]
pub(crate) unsafe fn copyfile_state_get_copied(state: copyfile_state_t) -> io::Result<u64> {
    let mut copied = MaybeUninit::<u64>::uninit();
    copyfile_state_get(state, COPYFILE_STATE_COPIED, copied.as_mut_ptr().cast())?;
    Ok(copied.assume_init())
}

#[cfg(apple)]
pub(crate) unsafe fn copyfile_state_get(
    state: copyfile_state_t,
    flag: u32,
    dst: *mut c::c_void,
) -> io::Result<()> {
    extern "C" {
        fn copyfile_state_get(state: copyfile_state_t, flag: u32, dst: *mut c::c_void) -> c::c_int;
    }

    nonnegative_ret(copyfile_state_get(state, flag, dst))
}

#[cfg(apple)]
pub(crate) fn getpath(fd: BorrowedFd<'_>) -> io::Result<CString> {
    // The use of `PATH_MAX` is generally not encouraged, but it
    // is inevitable in this case because macOS defines `fcntl` with
    // `F_GETPATH` in terms of `MAXPATHLEN`, and there are no
    // alternatives. If a better method is invented, it should be used
    // instead.
    let mut buf = vec![0; c::PATH_MAX as usize];

    // From the [macOS `fcntl` manual page]:
    // `F_GETPATH` - Get the path of the file descriptor `Fildes`. The argument
    //               must be a buffer of size `MAXPATHLEN` or greater.
    //
    // [macOS `fcntl` manual page]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
    unsafe {
        ret(c::fcntl(borrowed_fd(fd), c::F_GETPATH, buf.as_mut_ptr()))?;
    }

    let l = buf.iter().position(|&c| c == 0).unwrap();
    buf.truncate(l);

    // TODO: On Rust 1.56, we can use `shrink_to` here.
    //buf.shrink_to(l + 1);
    buf.shrink_to_fit();

    Ok(CString::new(buf).unwrap())
}

#[cfg(apple)]
pub(crate) fn fcntl_rdadvise(fd: BorrowedFd<'_>, offset: u64, len: u64) -> io::Result<()> {
    // From the [macOS `fcntl` manual page]:
    // `F_RDADVISE` - Issue an advisory read async with no copy to user.
    //
    // The `F_RDADVISE` command operates on the following structure which holds
    // information passed from the user to the system:
    //
    // ```c
    // struct radvisory {
    //      off_t   ra_offset;  /* offset into the file */
    //      int     ra_count;   /* size of the read     */
    // };
    // ```
    //
    // [macOS `fcntl` manual page]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
    let ra_offset = match offset.try_into() {
        Ok(len) => len,
        // If this conversion fails, the user is providing an offset outside
        // any possible file extent, so just ignore it.
        Err(_) => return Ok(()),
    };
    let ra_count = match len.try_into() {
        Ok(len) => len,
        // If this conversion fails, the user is providing a dubiously large
        // hint which is unlikely to improve performance.
        Err(_) => return Ok(()),
    };
    unsafe {
        let radvisory = c::radvisory {
            ra_offset,
            ra_count,
        };
        ret(c::fcntl(borrowed_fd(fd), c::F_RDADVISE, &radvisory))
    }
}

#[cfg(apple)]
pub(crate) fn fcntl_fullfsync(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::fcntl(borrowed_fd(fd), c::F_FULLFSYNC)) }
}

/// Convert `times` from a `futimens`/`utimensat` argument into `setattrlist`
/// arguments.
#[cfg(apple)]
fn times_to_attrlist(times: &Timestamps) -> (c::size_t, [c::timespec; 2], Attrlist) {
    // ABI details.
    const ATTR_CMN_MODTIME: u32 = 0x0000_0400;
    const ATTR_CMN_ACCTIME: u32 = 0x0000_1000;
    const ATTR_BIT_MAP_COUNT: u16 = 5;

    let mut times = times.clone();

    // If we have any `UTIME_NOW` elements, replace them with the current time.
    if times.last_access.tv_nsec == c::UTIME_NOW || times.last_modification.tv_nsec == c::UTIME_NOW
    {
        let now = {
            let mut tv = c::timeval {
                tv_sec: 0,
                tv_usec: 0,
            };
            unsafe {
                let r = c::gettimeofday(&mut tv, null_mut());
                assert_eq!(r, 0);
            }
            c::timespec {
                tv_sec: tv.tv_sec,
                tv_nsec: (tv.tv_usec * 1000) as _,
            }
        };
        if times.last_access.tv_nsec == c::UTIME_NOW {
            times.last_access = now;
        }
        if times.last_modification.tv_nsec == c::UTIME_NOW {
            times.last_modification = now;
        }
    }

    // Pack the return values following the rules for [`getattrlist`].
    //
    // [`getattrlist`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getattrlist.2.html
    let mut times_size = 0;
    let mut attrs = Attrlist {
        bitmapcount: ATTR_BIT_MAP_COUNT,
        reserved: 0,
        commonattr: 0,
        volattr: 0,
        dirattr: 0,
        fileattr: 0,
        forkattr: 0,
    };
    let mut return_times = [c::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    }; 2];
    let mut times_index = 0;
    if times.last_modification.tv_nsec != c::UTIME_OMIT {
        attrs.commonattr |= ATTR_CMN_MODTIME;
        return_times[times_index] = times.last_modification;
        times_index += 1;
        times_size += size_of::<c::timespec>();
    }
    if times.last_access.tv_nsec != c::UTIME_OMIT {
        attrs.commonattr |= ATTR_CMN_ACCTIME;
        return_times[times_index] = times.last_access;
        times_size += size_of::<c::timespec>();
    }

    (times_size, return_times, attrs)
}

/// Support type for `Attrlist`.
#[cfg(apple)]
type Attrgroup = u32;

/// Attribute list for use with `setattrlist`.
#[cfg(apple)]
#[repr(C)]
struct Attrlist {
    bitmapcount: u16,
    reserved: u16,
    commonattr: Attrgroup,
    volattr: Attrgroup,
    dirattr: Attrgroup,
    fileattr: Attrgroup,
    forkattr: Attrgroup,
}

#[cfg(linux_kernel)]
pub(crate) fn mount(
    source: Option<&CStr>,
    target: &CStr,
    file_system_type: Option<&CStr>,
    flags: super::types::MountFlagsArg,
    data: Option<&CStr>,
) -> io::Result<()> {
    unsafe {
        ret(c::mount(
            source.map_or_else(null, CStr::as_ptr),
            target.as_ptr(),
            file_system_type.map_or_else(null, CStr::as_ptr),
            flags.0,
            data.map_or_else(null, CStr::as_ptr).cast(),
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn unmount(target: &CStr, flags: super::types::UnmountFlags) -> io::Result<()> {
    unsafe { ret(c::umount2(target.as_ptr(), flags.bits())) }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn getxattr(path: &CStr, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let value_ptr = value.as_mut_ptr();

    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::getxattr(
            path.as_ptr(),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::getxattr(
            path.as_ptr(),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
            0,
            0,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn lgetxattr(path: &CStr, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let value_ptr = value.as_mut_ptr();

    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::lgetxattr(
            path.as_ptr(),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::getxattr(
            path.as_ptr(),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
            0,
            c::XATTR_NOFOLLOW,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn fgetxattr(fd: BorrowedFd<'_>, name: &CStr, value: &mut [u8]) -> io::Result<usize> {
    let value_ptr = value.as_mut_ptr();

    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::fgetxattr(
            borrowed_fd(fd),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::fgetxattr(
            borrowed_fd(fd),
            name.as_ptr(),
            value_ptr.cast::<c::c_void>(),
            value.len(),
            0,
            0,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn setxattr(
    path: &CStr,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    #[cfg(not(apple))]
    unsafe {
        ret(c::setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            flags.bits() as i32,
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            0,
            flags.bits() as i32,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn lsetxattr(
    path: &CStr,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    #[cfg(not(apple))]
    unsafe {
        ret(c::lsetxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            flags.bits() as i32,
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::setxattr(
            path.as_ptr(),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            0,
            flags.bits() as i32 | c::XATTR_NOFOLLOW,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn fsetxattr(
    fd: BorrowedFd<'_>,
    name: &CStr,
    value: &[u8],
    flags: XattrFlags,
) -> io::Result<()> {
    #[cfg(not(apple))]
    unsafe {
        ret(c::fsetxattr(
            borrowed_fd(fd),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            flags.bits() as i32,
        ))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::fsetxattr(
            borrowed_fd(fd),
            name.as_ptr(),
            value.as_ptr().cast::<c::c_void>(),
            value.len(),
            0,
            flags.bits() as i32,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn listxattr(path: &CStr, list: &mut [c::c_char]) -> io::Result<usize> {
    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::listxattr(path.as_ptr(), list.as_mut_ptr(), list.len()))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::listxattr(
            path.as_ptr(),
            list.as_mut_ptr(),
            list.len(),
            0,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn llistxattr(path: &CStr, list: &mut [c::c_char]) -> io::Result<usize> {
    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::llistxattr(path.as_ptr(), list.as_mut_ptr(), list.len()))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::listxattr(
            path.as_ptr(),
            list.as_mut_ptr(),
            list.len(),
            c::XATTR_NOFOLLOW,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn flistxattr(fd: BorrowedFd<'_>, list: &mut [c::c_char]) -> io::Result<usize> {
    let fd = borrowed_fd(fd);

    #[cfg(not(apple))]
    unsafe {
        ret_usize(c::flistxattr(fd, list.as_mut_ptr(), list.len()))
    }

    #[cfg(apple)]
    unsafe {
        ret_usize(c::flistxattr(fd, list.as_mut_ptr(), list.len(), 0))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn removexattr(path: &CStr, name: &CStr) -> io::Result<()> {
    #[cfg(not(apple))]
    unsafe {
        ret(c::removexattr(path.as_ptr(), name.as_ptr()))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::removexattr(path.as_ptr(), name.as_ptr(), 0))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn lremovexattr(path: &CStr, name: &CStr) -> io::Result<()> {
    #[cfg(not(apple))]
    unsafe {
        ret(c::lremovexattr(path.as_ptr(), name.as_ptr()))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::removexattr(
            path.as_ptr(),
            name.as_ptr(),
            c::XATTR_NOFOLLOW,
        ))
    }
}

#[cfg(any(apple, linux_kernel))]
pub(crate) fn fremovexattr(fd: BorrowedFd<'_>, name: &CStr) -> io::Result<()> {
    let fd = borrowed_fd(fd);

    #[cfg(not(apple))]
    unsafe {
        ret(c::fremovexattr(fd, name.as_ptr()))
    }

    #[cfg(apple)]
    unsafe {
        ret(c::fremovexattr(fd, name.as_ptr(), 0))
    }
}
