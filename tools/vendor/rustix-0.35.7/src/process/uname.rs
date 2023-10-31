//! Uname support.
//!
//! # Safety
//!
//! This function converts from `struct utsname` fields provided from the
//! kernel into `&str` references, which assumes that they're NUL-terminated.
#![allow(unsafe_code)]

use crate::backend;
use crate::ffi::CStr;
use core::fmt;

/// `uname()`—Returns high-level information about the runtime OS and
/// hardware.
#[inline]
pub fn uname() -> Uname {
    Uname(backend::process::syscalls::uname())
}

/// `struct utsname`—Return type for [`uname`].
#[doc(alias = "utsname")]
pub struct Uname(backend::process::types::RawUname);

impl Uname {
    /// `sysname`—Operating system release name
    #[inline]
    pub fn sysname(&self) -> &CStr {
        Self::to_cstr(self.0.sysname.as_ptr().cast())
    }

    /// `nodename`—Name with vague meaning
    ///
    /// This is intended to be a network name, however it's unable to convey
    /// information about hosts that have multiple names, or any information
    /// about where the names are visible.
    #[inline]
    pub fn nodename(&self) -> &CStr {
        Self::to_cstr(self.0.nodename.as_ptr().cast())
    }

    /// `release`—Operating system release version string
    #[inline]
    pub fn release(&self) -> &CStr {
        Self::to_cstr(self.0.release.as_ptr().cast())
    }

    /// `version`—Operating system build identifiers
    #[inline]
    pub fn version(&self) -> &CStr {
        Self::to_cstr(self.0.version.as_ptr().cast())
    }

    /// `machine`—Hardware architecture identifier
    #[inline]
    pub fn machine(&self) -> &CStr {
        Self::to_cstr(self.0.machine.as_ptr().cast())
    }

    /// `domainname`—NIS or YP domain identifier
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[inline]
    pub fn domainname(&self) -> &CStr {
        Self::to_cstr(self.0.domainname.as_ptr().cast())
    }

    #[inline]
    fn to_cstr<'a>(ptr: *const u8) -> &'a CStr {
        // Safety: Strings returned from the kernel are always NUL-terminated.
        unsafe { CStr::from_ptr(ptr.cast()) }
    }
}

impl fmt::Debug for Uname {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(any(target_os = "android", target_os = "linux")))]
        {
            write!(
                fmt,
                "{} {} {} {} {}",
                self.sysname().to_string_lossy(),
                self.nodename().to_string_lossy(),
                self.release().to_string_lossy(),
                self.version().to_string_lossy(),
                self.machine().to_string_lossy(),
            )
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        {
            write!(
                fmt,
                "{} {} {} {} {} {}",
                self.sysname().to_string_lossy(),
                self.nodename().to_string_lossy(),
                self.release().to_string_lossy(),
                self.version().to_string_lossy(),
                self.machine().to_string_lossy(),
                self.domainname().to_string_lossy(),
            )
        }
    }
}
