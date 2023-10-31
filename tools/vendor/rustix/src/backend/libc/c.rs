#![allow(unused_imports)]

// Import everything from libc, but we'll add some stuff and override some
// things below.
pub(crate) use libc::*;

/// `PROC_SUPER_MAGIC`—The magic number for the procfs filesystem.
#[cfg(all(linux_kernel, target_env = "musl"))]
pub(crate) const PROC_SUPER_MAGIC: u32 = 0x0000_9fa0;

/// `NFS_SUPER_MAGIC`—The magic number for the NFS filesystem.
#[cfg(all(linux_kernel, target_env = "musl"))]
pub(crate) const NFS_SUPER_MAGIC: u32 = 0x0000_6969;

// Submitted upstream in <https://github.com/rust-lang/libc/pull/3272>.
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const IPPROTO_ETHERNET: c_int = linux_raw_sys::net::IPPROTO_ETHERNET as _;

// TODO: Upstream these.
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_TSN: c_int = linux_raw_sys::if_ether::ETH_P_TSN as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_ERSPAN2: c_int = linux_raw_sys::if_ether::ETH_P_ERSPAN2 as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_ERSPAN: c_int = linux_raw_sys::if_ether::ETH_P_ERSPAN as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_PROFINET: c_int = linux_raw_sys::if_ether::ETH_P_PROFINET as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_REALTEK: c_int = linux_raw_sys::if_ether::ETH_P_REALTEK as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_ETHERCAT: c_int = linux_raw_sys::if_ether::ETH_P_ETHERCAT as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_PREAUTH: c_int = linux_raw_sys::if_ether::ETH_P_PREAUTH as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_LLDP: c_int = linux_raw_sys::if_ether::ETH_P_LLDP as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_MRP: c_int = linux_raw_sys::if_ether::ETH_P_MRP as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_NCSI: c_int = linux_raw_sys::if_ether::ETH_P_NCSI as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_CFM: c_int = linux_raw_sys::if_ether::ETH_P_CFM as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_IBOE: c_int = linux_raw_sys::if_ether::ETH_P_IBOE as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_HSR: c_int = linux_raw_sys::if_ether::ETH_P_HSR as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_NSH: c_int = linux_raw_sys::if_ether::ETH_P_NSH as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_DSA_8021Q: c_int = linux_raw_sys::if_ether::ETH_P_DSA_8021Q as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_DSA_A5PSW: c_int = linux_raw_sys::if_ether::ETH_P_DSA_A5PSW as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_IFE: c_int = linux_raw_sys::if_ether::ETH_P_IFE as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_CAN: c_int = linux_raw_sys::if_ether::ETH_P_CAN as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_CANXL: c_int = linux_raw_sys::if_ether::ETH_P_CANXL as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_XDSA: c_int = linux_raw_sys::if_ether::ETH_P_XDSA as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_MAP: c_int = linux_raw_sys::if_ether::ETH_P_MAP as _;
#[cfg(all(linux_kernel, feature = "net"))]
pub(crate) const ETH_P_MCTP: c_int = linux_raw_sys::if_ether::ETH_P_MCTP as _;

#[cfg(all(
    linux_kernel,
    any(
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "sparc",
        target_arch = "sparc64"
    )
))]
pub(crate) const SIGEMT: c_int = linux_raw_sys::general::SIGEMT as _;

// TODO: Upstream these.
#[cfg(all(linux_kernel, feature = "termios"))]
pub(crate) const IUCLC: tcflag_t = linux_raw_sys::general::IUCLC as _;
#[cfg(all(linux_kernel, feature = "termios"))]
pub(crate) const XCASE: tcflag_t = linux_raw_sys::general::XCASE as _;

// On PowerPC, the regular `termios` has the `termios2` fields and there is no
// `termios2`. linux-raw-sys has aliases `termios2` to `termios` to cover this
// difference, but we still need to manually import it since `libc` doesn't
// have this.
#[cfg(all(
    linux_kernel,
    feature = "termios",
    any(target_arch = "powerpc", target_arch = "powerpc64")
))]
pub(crate) use {
    linux_raw_sys::general::{termios2, CIBAUD},
    linux_raw_sys::ioctl::{TCGETS2, TCSETS2, TCSETSF2, TCSETSW2},
};

// Automatically enable “large file” support (LFS) features.

#[cfg(target_os = "vxworks")]
pub(super) use libc::_Vx_ticks64_t as _Vx_ticks_t;
#[cfg(target_os = "aix")]
pub(super) use libc::blksize64_t as blksize_t;
#[cfg(linux_kernel)]
pub(super) use libc::fallocate64 as fallocate;
#[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
#[cfg(any(linux_like, target_os = "aix"))]
pub(super) use libc::open64 as open;
#[cfg(any(linux_kernel, target_os = "aix", target_os = "l4re"))]
pub(super) use libc::posix_fallocate64 as posix_fallocate;
#[cfg(any(all(linux_like, not(target_os = "android")), target_os = "aix"))]
pub(super) use libc::{blkcnt64_t as blkcnt_t, rlim64_t as rlim_t};
#[cfg(any(linux_like, target_os = "aix"))]
pub(super) use libc::{
    fstat64 as fstat, fstatat64 as fstatat, fstatfs64 as fstatfs, fstatvfs64 as fstatvfs,
    ftruncate64 as ftruncate, getrlimit64 as getrlimit, ino64_t as ino_t, lseek64 as lseek,
    mmap64 as mmap, off64_t as off_t, openat64 as openat, posix_fadvise64 as posix_fadvise,
    rlimit64 as rlimit, setrlimit64 as setrlimit, statfs64 as statfs, statvfs64 as statvfs,
    RLIM64_INFINITY as RLIM_INFINITY,
};
#[cfg(apple)]
pub(super) use libc::{
    host_info64_t as host_info_t, host_statistics64 as host_statistics,
    vm_statistics64_t as vm_statistics_t,
};
#[cfg(not(all(linux_kernel, any(target_pointer_width = "32", target_arch = "mips64"))))]
#[cfg(any(linux_like, target_os = "aix"))]
pub(super) use libc::{lstat64 as lstat, stat64 as stat};
#[cfg(any(linux_kernel, target_os = "aix", target_os = "emscripten"))]
pub(super) use libc::{pread64 as pread, pwrite64 as pwrite};
#[cfg(any(target_os = "aix", target_os = "linux", target_os = "emscripten"))]
pub(super) use libc::{preadv64 as preadv, pwritev64 as pwritev};

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) unsafe fn prlimit(
    pid: libc::pid_t,
    resource: libc::__rlimit_resource_t,
    new_limit: *const libc::rlimit64,
    old_limit: *mut libc::rlimit64,
) -> libc::c_int {
    // `prlimit64` wasn't supported in glibc until 2.13.
    weak_or_syscall! {
        fn prlimit64(
            pid: libc::pid_t,
            resource: libc::__rlimit_resource_t,
            new_limit: *const libc::rlimit64,
            old_limit: *mut libc::rlimit64
        ) via SYS_prlimit64 -> libc::c_int
    }

    prlimit64(pid, resource, new_limit, old_limit)
}

#[cfg(all(target_os = "linux", target_env = "musl"))]
pub(super) unsafe fn prlimit(
    pid: libc::pid_t,
    resource: libc::c_int,
    new_limit: *const libc::rlimit64,
    old_limit: *mut libc::rlimit64,
) -> libc::c_int {
    weak_or_syscall! {
        fn prlimit64(
            pid: libc::pid_t,
            resource: libc::c_int,
            new_limit: *const libc::rlimit64,
            old_limit: *mut libc::rlimit64
        ) via SYS_prlimit64 -> libc::c_int
    }

    prlimit64(pid, resource, new_limit, old_limit)
}

#[cfg(target_os = "android")]
pub(super) unsafe fn prlimit(
    pid: libc::pid_t,
    resource: libc::c_int,
    new_limit: *const libc::rlimit64,
    old_limit: *mut libc::rlimit64,
) -> libc::c_int {
    weak_or_syscall! {
        fn prlimit64(
            pid: libc::pid_t,
            resource: libc::c_int,
            new_limit: *const libc::rlimit64,
            old_limit: *mut libc::rlimit64
        ) via SYS_prlimit64 -> libc::c_int
    }

    prlimit64(pid, resource, new_limit, old_limit)
}

// 64-bit offsets on 32-bit platforms are passed in endianness-specific
// lo/hi pairs. See src/backend/linux_raw/conv.rs for details.
#[cfg(all(linux_kernel, target_endian = "little", target_pointer_width = "32"))]
fn lo(x: i64) -> usize {
    (x >> 32) as usize
}
#[cfg(all(linux_kernel, target_endian = "little", target_pointer_width = "32"))]
fn hi(x: i64) -> usize {
    x as usize
}
#[cfg(all(linux_kernel, target_endian = "big", target_pointer_width = "32"))]
fn lo(x: i64) -> usize {
    x as usize
}
#[cfg(all(linux_kernel, target_endian = "big", target_pointer_width = "32"))]
fn hi(x: i64) -> usize {
    (x >> 32) as usize
}

#[cfg(target_os = "android")]
mod readwrite_pv64 {
    use super::*;

    pub(in super::super) unsafe fn preadv64(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
    ) -> libc::ssize_t {
        // Older Android libc lacks `preadv64`, so use the `weak!` mechanism to
        // test for it, and call back to `libc::syscall`. We don't use
        // `weak_or_syscall` here because we need to pass the 64-bit offset
        // specially.
        weak! {
            fn preadv64(libc::c_int, *const libc::iovec, libc::c_int, libc::off64_t) -> libc::ssize_t
        }
        if let Some(fun) = preadv64.get() {
            fun(fd, iov, iovcnt, offset)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                syscall! {
                    fn preadv(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset_hi: usize,
                        offset_lo: usize
                    ) via SYS_preadv -> libc::ssize_t
                }
                preadv(fd, iov, iovcnt, hi(offset), lo(offset))
            }
            #[cfg(target_pointer_width = "64")]
            {
                syscall! {
                    fn preadv(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset: libc::off_t
                    ) via SYS_preadv -> libc::ssize_t
                }
                preadv(fd, iov, iovcnt, offset)
            }
        }
    }
    pub(in super::super) unsafe fn pwritev64(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
    ) -> libc::ssize_t {
        // See the comments in `preadv64`.
        weak! {
            fn pwritev64(libc::c_int, *const libc::iovec, libc::c_int, libc::off64_t) -> libc::ssize_t
        }
        if let Some(fun) = pwritev64.get() {
            fun(fd, iov, iovcnt, offset)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                syscall! {
                    fn pwritev(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset_hi: usize,
                        offset_lo: usize
                    ) via SYS_pwritev -> libc::ssize_t
                }
                pwritev(fd, iov, iovcnt, hi(offset), lo(offset))
            }
            #[cfg(target_pointer_width = "64")]
            {
                syscall! {
                    fn pwritev(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset: libc::off_t
                    ) via SYS_pwritev -> libc::ssize_t
                }
                pwritev(fd, iov, iovcnt, offset)
            }
        }
    }
}
#[cfg(target_os = "android")]
pub(super) use readwrite_pv64::{preadv64 as preadv, pwritev64 as pwritev};

// macOS added preadv and pwritev in version 11.0
#[cfg(apple)]
mod readwrite_pv {
    weakcall! {
        pub(in super::super) fn preadv(
            fd: libc::c_int,
            iov: *const libc::iovec,
            iovcnt: libc::c_int,
            offset: libc::off_t
        ) -> libc::ssize_t
    }
    weakcall! {
        pub(in super::super) fn pwritev(
            fd: libc::c_int,
            iov: *const libc::iovec,
            iovcnt: libc::c_int, offset: libc::off_t
        ) -> libc::ssize_t
    }
}
#[cfg(apple)]
pub(super) use readwrite_pv::{preadv, pwritev};

// glibc added `preadv64v2` and `pwritev64v2` in version 2.26.
#[cfg(all(target_os = "linux", target_env = "gnu"))]
mod readwrite_pv64v2 {
    use super::*;

    pub(in super::super) unsafe fn preadv64v2(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
        flags: libc::c_int,
    ) -> libc::ssize_t {
        // Older glibc lacks `preadv64v2`, so use the `weak!` mechanism to
        // test for it, and call back to `libc::syscall`. We don't use
        // `weak_or_syscall` here because we need to pass the 64-bit offset
        // specially.
        weak! {
            fn preadv64v2(libc::c_int, *const libc::iovec, libc::c_int, libc::off64_t, libc::c_int) -> libc::ssize_t
        }
        if let Some(fun) = preadv64v2.get() {
            fun(fd, iov, iovcnt, offset, flags)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                syscall! {
                    fn preadv2(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset_hi: usize,
                        offset_lo: usize,
                        flags: libc::c_int
                    ) via SYS_preadv2 -> libc::ssize_t
                }
                preadv2(fd, iov, iovcnt, hi(offset), lo(offset), flags)
            }
            #[cfg(target_pointer_width = "64")]
            {
                syscall! {
                    fn preadv2(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovcnt: libc::c_int,
                        offset: libc::off_t,
                        flags: libc::c_int
                    ) via SYS_preadv2 -> libc::ssize_t
                }
                preadv2(fd, iov, iovcnt, offset, flags)
            }
        }
    }
    pub(in super::super) unsafe fn pwritev64v2(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
        flags: libc::c_int,
    ) -> libc::ssize_t {
        // See the comments in `preadv64v2`.
        weak! {
            fn pwritev64v2(libc::c_int, *const libc::iovec, libc::c_int, libc::off64_t, libc::c_int) -> libc::ssize_t
        }
        if let Some(fun) = pwritev64v2.get() {
            fun(fd, iov, iovcnt, offset, flags)
        } else {
            #[cfg(target_pointer_width = "32")]
            {
                syscall! {
                    fn pwritev2(
                        fd: libc::c_int,
                        iov: *const libc::iovec,
                        iovec: libc::c_int,
                        offset_hi: usize,
                        offset_lo: usize,
                        flags: libc::c_int
                    ) via SYS_pwritev2 -> libc::ssize_t
                }
                pwritev2(fd, iov, iovcnt, hi(offset), lo(offset), flags)
            }
            #[cfg(target_pointer_width = "64")]
            {
                syscall! {
                    fn pwritev2(
                        fd: libc::c_int,
                        iov:*const libc::iovec,
                        iovcnt: libc::c_int,
                        offset: libc::off_t,
                        flags: libc::c_int
                    ) via SYS_pwritev2 -> libc::ssize_t
                }
                pwritev2(fd, iov, iovcnt, offset, flags)
            }
        }
    }
}
#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(super) use readwrite_pv64v2::{preadv64v2 as preadv2, pwritev64v2 as pwritev2};

// On non-glibc, assume we don't have `pwritev2`/`preadv2` in libc and use
// `c::syscall` instead.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
mod readwrite_pv64v2 {
    use super::*;

    pub(in super::super) unsafe fn preadv64v2(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
        flags: libc::c_int,
    ) -> libc::ssize_t {
        #[cfg(target_pointer_width = "32")]
        {
            syscall! {
                fn preadv2(
                    fd: libc::c_int,
                    iov: *const libc::iovec,
                    iovcnt: libc::c_int,
                    offset_hi: usize,
                    offset_lo: usize,
                    flags: libc::c_int
                ) via SYS_preadv2 -> libc::ssize_t
            }
            preadv2(fd, iov, iovcnt, hi(offset), lo(offset), flags)
        }
        #[cfg(target_pointer_width = "64")]
        {
            syscall! {
                fn preadv2(
                    fd: libc::c_int,
                    iov: *const libc::iovec,
                    iovcnt: libc::c_int,
                    offset: libc::off_t,
                    flags: libc::c_int
                ) via SYS_preadv2 -> libc::ssize_t
            }
            preadv2(fd, iov, iovcnt, offset, flags)
        }
    }
    pub(in super::super) unsafe fn pwritev64v2(
        fd: libc::c_int,
        iov: *const libc::iovec,
        iovcnt: libc::c_int,
        offset: libc::off64_t,
        flags: libc::c_int,
    ) -> libc::ssize_t {
        #[cfg(target_pointer_width = "32")]
        {
            syscall! {
                fn pwritev2(
                    fd: libc::c_int,
                    iov: *const libc::iovec,
                    iovcnt: libc::c_int,
                    offset_hi: usize,
                    offset_lo: usize,
                    flags: libc::c_int
                ) via SYS_pwritev2 -> libc::ssize_t
            }
            pwritev2(fd, iov, iovcnt, hi(offset), lo(offset), flags)
        }
        #[cfg(target_pointer_width = "64")]
        {
            syscall! {
                fn pwritev2(
                    fd: libc::c_int,
                    iov:*const libc::iovec,
                    iovcnt: libc::c_int,
                    offset: libc::off_t,
                    flags: libc::c_int
                ) via SYS_pwritev2 -> libc::ssize_t
            }
            pwritev2(fd, iov, iovcnt, offset, flags)
        }
    }
}
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
pub(super) use readwrite_pv64v2::{preadv64v2 as preadv2, pwritev64v2 as pwritev2};
