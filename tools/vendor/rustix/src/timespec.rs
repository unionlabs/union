//! `Timespec` and related types, which are used by multiple public API
//! modules.

use crate::backend::c;

/// `struct timespec`
#[cfg(not(all(
    libc,
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
pub type Timespec = c::timespec;

/// `struct timespec`
#[cfg(all(
    libc,
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct Timespec {
    /// Seconds.
    pub tv_sec: Secs,

    /// Nanoseconds. Must be less than 1_000_000_000.
    pub tv_nsec: Nsecs,
}

/// A type for the `tv_sec` field of [`Timespec`].
#[cfg(not(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
)))]
#[allow(deprecated)]
pub type Secs = c::time_t;

/// A type for the `tv_sec` field of [`Timespec`].
#[cfg(all(
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
pub type Secs = i64;

/// A type for the `tv_nsec` field of [`Timespec`].
#[cfg(all(libc, target_arch = "x86_64", target_pointer_width = "32"))]
pub type Nsecs = i64;

/// A type for the `tv_nsec` field of [`Timespec`].
#[cfg(all(libc, not(all(target_arch = "x86_64", target_pointer_width = "32"))))]
pub type Nsecs = c::c_long;

/// A type for the `tv_nsec` field of [`Timespec`].
#[cfg(linux_raw)]
pub type Nsecs = i64;

/// On 32-bit glibc platforms, `timespec` has anonymous padding fields, which
/// Rust doesn't support yet (see `unnamed_fields`), so we define our own
/// struct with explicit padding, with bidirectional `From` impls.
#[cfg(all(
    libc,
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
#[repr(C)]
#[derive(Debug, Clone)]
pub(crate) struct LibcTimespec {
    pub(crate) tv_sec: Secs,

    #[cfg(target_endian = "big")]
    padding: core::mem::MaybeUninit<u32>,

    pub(crate) tv_nsec: Nsecs,

    #[cfg(target_endian = "little")]
    padding: core::mem::MaybeUninit<u32>,
}

#[cfg(all(
    libc,
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<LibcTimespec> for Timespec {
    #[inline]
    fn from(t: LibcTimespec) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_nsec: t.tv_nsec,
        }
    }
}

#[cfg(all(
    libc,
    any(target_arch = "arm", target_arch = "mips", target_arch = "x86"),
    target_env = "gnu",
))]
impl From<Timespec> for LibcTimespec {
    #[inline]
    fn from(t: Timespec) -> Self {
        Self {
            tv_sec: t.tv_sec,
            tv_nsec: t.tv_nsec,
            padding: core::mem::MaybeUninit::uninit(),
        }
    }
}
