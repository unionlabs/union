#![allow(non_camel_case_types, non_upper_case_globals, non_snake_case)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
pub use std::os::raw as ctypes;

#[cfg(all(not(feature = "std"), feature = "no_std"))]
pub mod ctypes {
    // The signedness of `char` is platform-specific, however a consequence
    // of it being platform-specific is that any code which depends on the
    // signedness of `char` is already non-portable. So we can just use `u8`
    // here and no portable code will notice.
    pub type c_char = u8;

    // The following assumes that Linux is always either ILP32 or LP64,
    // and char is always 8-bit.
    //
    // In theory, `c_long` and `c_ulong` could be `isize` and `usize`
    // respectively, however in practice Linux doesn't use them in that way
    // consistently. So stick with the convention followed by `libc` and
    // others and use the fixed-width types.
    pub type c_schar = i8;
    pub type c_uchar = u8;
    pub type c_short = i16;
    pub type c_ushort = u16;
    pub type c_int = i32;
    pub type c_uint = u32;
    #[cfg(target_pointer_width = "32")]
    pub type c_long = i32;
    #[cfg(target_pointer_width = "32")]
    pub type c_ulong = u32;
    #[cfg(target_pointer_width = "64")]
    pub type c_long = i64;
    #[cfg(target_pointer_width = "64")]
    pub type c_ulong = u64;
    pub type c_longlong = i64;
    pub type c_ulonglong = u64;
    pub type c_float = f32;
    pub type c_double = f64;

    pub use core::ffi::c_void;
}

// Confirm that our type definitions above match the actual type definitions.
#[cfg(test)]
mod assertions {
    use super::ctypes;
    static_assertions::assert_eq_size!(ctypes::c_char, libc::c_char);
    static_assertions::assert_type_eq_all!(ctypes::c_schar, libc::c_schar);
    static_assertions::assert_type_eq_all!(ctypes::c_uchar, libc::c_uchar);
    static_assertions::assert_type_eq_all!(ctypes::c_short, libc::c_short);
    static_assertions::assert_type_eq_all!(ctypes::c_ushort, libc::c_ushort);
    static_assertions::assert_type_eq_all!(ctypes::c_int, libc::c_int);
    static_assertions::assert_type_eq_all!(ctypes::c_uint, libc::c_uint);
    static_assertions::assert_type_eq_all!(ctypes::c_long, libc::c_long);
    static_assertions::assert_type_eq_all!(ctypes::c_ulong, libc::c_ulong);
    static_assertions::assert_type_eq_all!(ctypes::c_longlong, libc::c_longlong);
    static_assertions::assert_type_eq_all!(ctypes::c_ulonglong, libc::c_ulonglong);
    static_assertions::assert_type_eq_all!(ctypes::c_float, libc::c_float);
    static_assertions::assert_type_eq_all!(ctypes::c_double, libc::c_double);
}

// We don't enable `derive_eq` in bindgen because adding `PartialEq`/`Eq` to
// *all* structs noticeably increases compile times. But we can add a few
// manual impls where they're especially useful.
#[cfg(feature = "general")]
impl PartialEq for general::__kernel_timespec {
    fn eq(&self, other: &Self) -> bool {
        ({
            let Self { tv_sec, tv_nsec } = self;
            (tv_sec, tv_nsec)
        }) == ({
            let Self { tv_sec, tv_nsec } = other;
            (tv_sec, tv_nsec)
        })
    }
}
#[cfg(feature = "general")]
impl Eq for general::__kernel_timespec {}

#[cfg(feature = "general")]
pub mod cmsg_macros {
    use crate::ctypes::{c_long, c_uchar, c_uint};
    use crate::general::{cmsghdr, msghdr};
    use core::mem::size_of;
    use core::ptr;

    pub const unsafe fn CMSG_ALIGN(len: c_uint) -> c_uint {
        let c_long_size = size_of::<c_long>() as c_uint;
        (len + c_long_size - 1) & !(c_long_size - 1)
    }

    // TODO: In Rust 1.63 we can make this a `const fn`.
    pub unsafe fn CMSG_DATA(cmsg: *const cmsghdr) -> *mut c_uchar {
        (cmsg as *mut c_uchar).add(size_of::<cmsghdr>())
    }

    pub const unsafe fn CMSG_SPACE(len: c_uint) -> c_uint {
        size_of::<cmsghdr>() as c_uint + CMSG_ALIGN(len)
    }

    pub const unsafe fn CMSG_LEN(len: c_uint) -> c_uint {
        size_of::<cmsghdr>() as c_uint + len
    }

    // TODO: In Rust 1.63 we can make this a `const fn`.
    pub unsafe fn CMSG_FIRSTHDR(mhdr: *const msghdr) -> *mut cmsghdr {
        if (*mhdr).msg_controllen < size_of::<cmsghdr>() as _ {
            return ptr::null_mut();
        }

        (*mhdr).msg_control as *mut cmsghdr
    }

    pub unsafe fn CMSG_NXTHDR(mhdr: *const msghdr, cmsg: *const cmsghdr) -> *mut cmsghdr {
        // We convert from raw pointers to usize here, which may not be sound in a
        // future version of Rust. Once the provenance rules are set in stone,
        // it will be a good idea to give this function a once-over.

        let cmsg_len = (*cmsg).cmsg_len;
        let next_cmsg = (cmsg as *mut u8).add(CMSG_ALIGN(cmsg_len as _) as usize) as *mut cmsghdr;
        let max = ((*mhdr).msg_control as usize) + ((*mhdr).msg_controllen as usize);

        if cmsg_len < size_of::<cmsghdr>() as _ {
            return ptr::null_mut();
        }

        if next_cmsg.add(1) as usize > max
            || next_cmsg as usize + CMSG_ALIGN(cmsg_len as _) as usize > max
        {
            return ptr::null_mut();
        }

        next_cmsg
    }
}

#[cfg(feature = "general")]
pub mod select_macros {
    use crate::ctypes::c_int;
    use crate::general::__kernel_fd_set;
    use core::mem::size_of;

    pub unsafe fn FD_CLR(fd: c_int, set: *mut __kernel_fd_set) {
        let bytes = set as *mut u8;
        if fd >= 0 {
            *bytes.add((fd / 8) as usize) &= !(1 << (fd % 8));
        }
    }

    pub unsafe fn FD_SET(fd: c_int, set: *mut __kernel_fd_set) {
        let bytes = set as *mut u8;
        if fd >= 0 {
            *bytes.add((fd / 8) as usize) |= 1 << (fd % 8);
        }
    }

    pub unsafe fn FD_ISSET(fd: c_int, set: *const __kernel_fd_set) -> bool {
        let bytes = set as *const u8;
        if fd >= 0 {
            *bytes.add((fd / 8) as usize) & (1 << (fd % 8)) != 0
        } else {
            false
        }
    }

    pub unsafe fn FD_ZERO(set: *mut __kernel_fd_set) {
        let bytes = set as *mut u8;
        core::ptr::write_bytes(bytes, 0, size_of::<__kernel_fd_set>());
    }
}

#[cfg(feature = "general")]
pub mod signal_macros {
    pub const SIG_DFL: super::general::__kernel_sighandler_t = None;

    /// Rust doesn't currently permit us to use `transmute` to convert the
    /// `SIG_IGN` value into a function pointer in a `const` initializer, so
    /// we make it a function instead.
    ///
    // TODO: In Rust 1.56 we can make this a `const fn`.
    #[inline]
    pub fn sig_ign() -> super::general::__kernel_sighandler_t {
        // Safety: This creates an invalid pointer, but the pointer type
        // includes `unsafe`, which covers the safety of calling it.
        Some(unsafe {
            core::mem::transmute::<usize, unsafe extern "C" fn(crate::ctypes::c_int)>(1)
        })
    }
}

// The rest of this file is auto-generated!
#[cfg(feature = "errno")]
#[cfg(target_arch = "arm")]
#[path = "arm/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "arm")]
#[path = "arm/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "arm")]
#[path = "arm/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "arm")]
#[path = "arm/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "aarch64")]
#[path = "aarch64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "aarch64")]
#[path = "aarch64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "aarch64")]
#[path = "aarch64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "aarch64")]
#[path = "aarch64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "loongarch64")]
#[path = "loongarch64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "loongarch64")]
#[path = "loongarch64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "loongarch64")]
#[path = "loongarch64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "loongarch64")]
#[path = "loongarch64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "mips")]
#[path = "mips/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "mips")]
#[path = "mips/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "mips")]
#[path = "mips/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "mips")]
#[path = "mips/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "mips64")]
#[path = "mips64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "mips64")]
#[path = "mips64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "mips64")]
#[path = "mips64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "mips64")]
#[path = "mips64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "powerpc")]
#[path = "powerpc/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "powerpc")]
#[path = "powerpc/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "powerpc")]
#[path = "powerpc/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "powerpc")]
#[path = "powerpc/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "powerpc64")]
#[path = "powerpc64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "powerpc64")]
#[path = "powerpc64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "powerpc64")]
#[path = "powerpc64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "powerpc64")]
#[path = "powerpc64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "riscv32")]
#[path = "riscv32/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "riscv32")]
#[path = "riscv32/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "riscv32")]
#[path = "riscv32/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "riscv32")]
#[path = "riscv32/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "riscv64")]
#[path = "riscv64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "riscv64")]
#[path = "riscv64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "riscv64")]
#[path = "riscv64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "riscv64")]
#[path = "riscv64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "s390x")]
#[path = "s390x/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "s390x")]
#[path = "s390x/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "s390x")]
#[path = "s390x/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "s390x")]
#[path = "s390x/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "sparc")]
#[path = "sparc/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "sparc")]
#[path = "sparc/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "sparc")]
#[path = "sparc/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "sparc")]
#[path = "sparc/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "sparc64")]
#[path = "sparc64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "sparc64")]
#[path = "sparc64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "sparc64")]
#[path = "sparc64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "sparc64")]
#[path = "sparc64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(target_arch = "x86")]
#[path = "x86/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "x86")]
#[path = "x86/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "x86")]
#[path = "x86/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "x86")]
#[path = "x86/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[path = "x86_64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[path = "x86_64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[path = "x86_64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "64"))]
#[path = "x86_64/netlink.rs"]
pub mod netlink;
#[cfg(feature = "errno")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
#[path = "x32/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
#[path = "x32/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
#[path = "x32/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(all(target_arch = "x86_64", target_pointer_width = "32"))]
#[path = "x32/netlink.rs"]
pub mod netlink;
