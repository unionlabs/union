//! Automatically enable “large file” support features.

#[cfg(not(windows))]
use super::c;

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
)))]
pub(super) use c::{
    fstat as libc_fstat, fstatat as libc_fstatat, ftruncate as libc_ftruncate, lseek as libc_lseek,
    off_t as libc_off_t,
};

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub(super) use c::{
    fstat64 as libc_fstat, fstatat64 as libc_fstatat, ftruncate64 as libc_ftruncate,
    lseek64 as libc_lseek, off64_t as libc_off_t, rlimit64 as libc_rlimit,
};

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
    target_os = "wasi",
)))]
#[cfg(any(feature = "mm", feature = "time", target_arch = "x86"))] // vdso.rs uses `madvise`
pub(super) use c::mmap as libc_mmap;

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "fuchsia",
    target_os = "l4re",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(super) use c::{rlimit as libc_rlimit, RLIM_INFINITY as LIBC_RLIM_INFINITY};

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "fuchsia",
    target_os = "emscripten",
    target_os = "l4re",
    target_os = "linux",
    target_os = "wasi",
)))]
pub(super) use c::{getrlimit as libc_getrlimit, setrlimit as libc_setrlimit};

// TODO: Add `RLIM64_INFINITY` to upstream libc.
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub(super) const LIBC_RLIM_INFINITY: u64 = !0_u64;

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub(super) use c::{getrlimit64 as libc_getrlimit, setrlimit64 as libc_setrlimit};

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
#[cfg(any(feature = "mm", feature = "time", target_arch = "x86"))] // vdso.rs uses `madvise`
pub(super) use c::mmap64 as libc_mmap;

// `prlimit64` wasn't supported in glibc until 2.13.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
weak_or_syscall! {
    fn prlimit64(
        pid: c::pid_t,
        resource: c::__rlimit_resource_t,
        new_limit: *const c::rlimit64,
        old_limit: *mut c::rlimit64
    ) via SYS_prlimit64 -> c::c_int
}
#[cfg(all(target_os = "linux", target_env = "musl"))]
weak_or_syscall! {
    fn prlimit64(
        pid: c::pid_t,
        resource: c::c_int,
        new_limit: *const c::rlimit64,
        old_limit: *mut c::rlimit64
    ) via SYS_prlimit64 -> c::c_int
}
#[cfg(target_os = "android")]
weak_or_syscall! {
    fn prlimit64(
        pid: c::pid_t,
        resource: c::c_int,
        new_limit: *const c::rlimit64,
        old_limit: *mut c::rlimit64
    ) via SYS_prlimit64 -> c::c_int
}
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) unsafe fn libc_prlimit(
    pid: c::pid_t,
    resource: c::__rlimit_resource_t,
    new_limit: *const c::rlimit64,
    old_limit: *mut c::rlimit64,
) -> c::c_int {
    prlimit64(pid, resource, new_limit, old_limit)
}
#[cfg(all(target_os = "linux", target_env = "musl"))]
pub(super) unsafe fn libc_prlimit(
    pid: c::pid_t,
    resource: c::c_int,
    new_limit: *const c::rlimit64,
    old_limit: *mut c::rlimit64,
) -> c::c_int {
    prlimit64(pid, resource, new_limit, old_limit)
}
#[cfg(target_os = "android")]
pub(super) unsafe fn libc_prlimit(
    pid: c::pid_t,
    resource: c::c_int,
    new_limit: *const c::rlimit64,
    old_limit: *mut c::rlimit64,
) -> c::c_int {
    prlimit64(pid, resource, new_limit, old_limit)
}

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
    target_os = "redox",
)))]
pub(super) use c::openat as libc_openat;
#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub(super) use c::openat64 as libc_openat;

#[cfg(target_os = "fuchsia")]
pub(super) use c::fallocate as libc_fallocate;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub(super) use c::fallocate64 as libc_fallocate;
#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "ios",
    target_os = "linux",
    target_os = "l4re",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub(super) use c::posix_fadvise as libc_posix_fadvise;
#[cfg(any(
    target_os = "android",
    target_os = "emscripten",
    target_os = "linux",
    target_os = "l4re",
))]
pub(super) use c::posix_fadvise64 as libc_posix_fadvise;

#[cfg(all(not(any(
    windows,
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
))))]
pub(super) use c::{pread as libc_pread, pwrite as libc_pwrite};
#[cfg(any(target_os = "android", target_os = "linux", target_os = "emscripten"))]
pub(super) use c::{pread64 as libc_pread, pwrite64 as libc_pwrite};
#[cfg(any(target_os = "linux", target_os = "emscripten"))]
pub(super) use c::{preadv64 as libc_preadv, pwritev64 as libc_pwritev};
#[cfg(target_os = "android")]
mod readwrite_pv64 {
    use super::c;

    // 64-bit offsets on 32-bit platforms are passed in endianness-specific
    // lo/hi pairs. See src/backend/linux_raw/conv.rs for details.
    #[cfg(all(target_endian = "little", target_pointer_width = "32"))]
    fn lo(x: u64) -> usize {
        (x >> 32) as usize
    }
    #[cfg(all(target_endian = "little", target_pointer_width = "32"))]
    fn hi(x: u64) -> usize {
        (x & 0xffff_ffff) as usize
    }
    #[cfg(all(target_endian = "big", target_pointer_width = "32"))]
    fn lo(x: u64) -> usize {
        (x & 0xffff_ffff) as usize
    }
    #[cfg(all(target_endian = "big", target_pointer_width = "32"))]
    fn hi(x: u64) -> usize {
        (x >> 32) as usize
    }

    pub(in super::super) unsafe fn preadv64(
        fd: c::c_int,
        iov: *const c::iovec,
        iovcnt: c::c_int,
        offset: c::off64_t,
    ) -> c::ssize_t {
        // Older Android libc lacks `preadv64`, so use the `weak!` mechanism to
        // test for it, and call back to `c::syscall`. We don't use
        // `weak_or_syscall` here because we need to pass the 64-bit offset
        // specially.
        weak! {
            fn preadv64(c::c_int, *const c::iovec, c::c_int, c::off64_t) -> c::ssize_t
        }
        if let Some(fun) = preadv64.get() {
            fun(fd, iov, iovcnt, offset)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                c::syscall(
                    c::SYS_preadv,
                    fd,
                    iov,
                    iovcnt,
                    hi(offset as u64),
                    lo(offset as u64),
                ) as c::ssize_t
            }
            #[cfg(target_pointer_width = "64")]
            {
                c::syscall(c::SYS_preadv, fd, iov, iovcnt, offset) as c::ssize_t
            }
        }
    }
    pub(in super::super) unsafe fn pwritev64(
        fd: c::c_int,
        iov: *const c::iovec,
        iovcnt: c::c_int,
        offset: c::off64_t,
    ) -> c::ssize_t {
        // See the comments in `preadv64`.
        weak! {
            fn pwritev64(c::c_int, *const c::iovec, c::c_int, c::off64_t) -> c::ssize_t
        }
        if let Some(fun) = pwritev64.get() {
            fun(fd, iov, iovcnt, offset)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                c::syscall(
                    c::SYS_pwritev,
                    fd,
                    iov,
                    iovcnt,
                    hi(offset as u64),
                    lo(offset as u64),
                ) as c::ssize_t
            }
            #[cfg(target_pointer_width = "64")]
            {
                c::syscall(c::SYS_pwritev, fd, iov, iovcnt, offset) as c::ssize_t
            }
        }
    }
}
#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "emscripten",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "redox",
)))]
pub(super) use c::{preadv as libc_preadv, pwritev as libc_pwritev};
#[cfg(target_os = "android")]
pub(super) use readwrite_pv64::{preadv64 as libc_preadv, pwritev64 as libc_pwritev};
// macOS added preadv and pwritev in version 11.0
#[cfg(any(target_os = "ios", target_os = "macos"))]
mod readwrite_pv {
    use super::c;

    weakcall! {
        pub(in super::super) fn preadv(
            fd: c::c_int,
            iov: *const c::iovec,
            iovcnt: c::c_int,
            offset: c::off_t
        ) -> c::ssize_t
    }
    weakcall! {
        pub(in super::super) fn pwritev(
            fd: c::c_int,
            iov: *const c::iovec,
            iovcnt: c::c_int, offset: c::off_t
        ) -> c::ssize_t
    }
}
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) use c::{preadv64v2 as libc_preadv2, pwritev64v2 as libc_pwritev2};
#[cfg(any(target_os = "ios", target_os = "macos"))]
pub(super) use readwrite_pv::{preadv as libc_preadv, pwritev as libc_pwritev};

#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "dragonfly",
    target_os = "fuchsia",
    target_os = "illumos",
    target_os = "ios",
    target_os = "linux",
    target_os = "l4re",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
)))]
pub(super) use c::posix_fallocate as libc_posix_fallocate;
#[cfg(any(target_os = "l4re"))]
pub(super) use c::posix_fallocate64 as libc_posix_fallocate;
#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "linux",
    target_os = "l4re",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(super) use {c::fstatfs as libc_fstatfs, c::statfs as libc_statfs};
#[cfg(not(any(
    windows,
    target_os = "android",
    target_os = "emscripten",
    target_os = "illumos",
    target_os = "linux",
    target_os = "l4re",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(super) use {c::fstatvfs as libc_fstatvfs, c::statvfs as libc_statvfs};

#[cfg(any(
    target_os = "android",
    target_os = "linux",
    target_os = "emscripten",
    target_os = "l4re",
))]
pub(super) use {
    c::fstatfs64 as libc_fstatfs, c::fstatvfs64 as libc_fstatvfs, c::statfs64 as libc_statfs,
    c::statvfs64 as libc_statvfs,
};
