//! Uname and other system-level functions.
//!
//! # Safety
//!
//! This function converts from `struct utsname` fields provided from the
//! kernel into `&str` references, which assumes that they're NUL-terminated.
#![allow(unsafe_code)]

use crate::backend;
use crate::ffi::CStr;
#[cfg(not(target_os = "emscripten"))]
use crate::io;
use core::fmt;

#[cfg(linux_kernel)]
pub use backend::process::types::Sysinfo;

/// `uname()`—Returns high-level information about the runtime OS and
/// hardware.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/uname.html
/// [Linux]: https://man7.org/linux/man-pages/man2/uname.2.html
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
    #[cfg(linux_kernel)]
    #[inline]
    pub fn domainname(&self) -> &CStr {
        Self::to_cstr(self.0.domainname.as_ptr().cast())
    }

    #[inline]
    fn to_cstr<'a>(ptr: *const u8) -> &'a CStr {
        // SAFETY: Strings returned from the kernel are always NUL-terminated.
        unsafe { CStr::from_ptr(ptr.cast()) }
    }
}

impl fmt::Debug for Uname {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(not(linux_kernel))]
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
        #[cfg(linux_kernel)]
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

/// `sysinfo()`—Returns status information about the runtime OS.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/uname.2.html
#[cfg(linux_kernel)]
#[inline]
pub fn sysinfo() -> Sysinfo {
    backend::process::syscalls::sysinfo()
}

/// `sethostname(name)`—Sets the system host name.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/sethostname.2.html
#[cfg(not(any(target_os = "emscripten", target_os = "redox", target_os = "wasi")))]
#[inline]
pub fn sethostname(name: &[u8]) -> io::Result<()> {
    backend::process::syscalls::sethostname(name)
}
