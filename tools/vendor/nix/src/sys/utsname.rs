//! Get system identification
use std::mem;
use libc::{self, c_char};
use std::ffi::CStr;
use std::str::from_utf8_unchecked;

/// Describes the running system.  Return type of [`uname`].
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[repr(transparent)]
pub struct UtsName(libc::utsname);

impl UtsName {
    /// Name of the operating system implementation
    pub fn sysname(&self) -> &str {
        to_str(&(&self.0.sysname as *const c_char ) as *const *const c_char)
    }

    /// Network name of this machine.
    pub fn nodename(&self) -> &str {
        to_str(&(&self.0.nodename as *const c_char ) as *const *const c_char)
    }

    /// Release level of the operating system.
    pub fn release(&self) -> &str {
        to_str(&(&self.0.release as *const c_char ) as *const *const c_char)
    }

    /// Version level of the operating system.
    pub fn version(&self) -> &str {
        to_str(&(&self.0.version as *const c_char ) as *const *const c_char)
    }

    /// Machine hardware platform.
    pub fn machine(&self) -> &str {
        to_str(&(&self.0.machine as *const c_char ) as *const *const c_char)
    }
}

/// Get system identification
pub fn uname() -> UtsName {
    unsafe {
        let mut ret = mem::MaybeUninit::uninit();
        libc::uname(ret.as_mut_ptr());
        UtsName(ret.assume_init())
    }
}

#[inline]
fn to_str<'a>(s: *const *const c_char) -> &'a str {
    unsafe {
        let res = CStr::from_ptr(*s).to_bytes();
        from_utf8_unchecked(res)
    }
}

#[cfg(test)]
mod test {
    #[cfg(target_os = "linux")]
    #[test]
    pub fn test_uname_linux() {
        assert_eq!(super::uname().sysname(), "Linux");
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    #[test]
    pub fn test_uname_darwin() {
        assert_eq!(super::uname().sysname(), "Darwin");
    }

    #[cfg(target_os = "freebsd")]
    #[test]
    pub fn test_uname_freebsd() {
        assert_eq!(super::uname().sysname(), "FreeBSD");
    }
}
