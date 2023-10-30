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
#[cfg(target_arch = "x86_64")]
#[path = "x86_64/errno.rs"]
pub mod errno;
#[cfg(feature = "general")]
#[cfg(target_arch = "x86_64")]
#[path = "x86_64/general.rs"]
pub mod general;
#[cfg(feature = "ioctl")]
#[cfg(target_arch = "x86_64")]
#[path = "x86_64/ioctl.rs"]
pub mod ioctl;
#[cfg(feature = "netlink")]
#[cfg(target_arch = "x86_64")]
#[path = "x86_64/netlink.rs"]
pub mod netlink;
