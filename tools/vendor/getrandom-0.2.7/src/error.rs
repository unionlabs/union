// Copyright 2018 Developers of the Rand project.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// https://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or https://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.
use core::{fmt, num::NonZeroU32};

/// A small and `no_std` compatible error type
///
/// The [`Error::raw_os_error()`] will indicate if the error is from the OS, and
/// if so, which error code the OS gave the application. If such an error is
/// encountered, please consult with your system documentation.
///
/// Internally this type is a NonZeroU32, with certain values reserved for
/// certain purposes, see [`Error::INTERNAL_START`] and [`Error::CUSTOM_START`].
///
/// *If this crate's `"std"` Cargo feature is enabled*, then:
/// - [`getrandom::Error`][Error] implements
///   [`std::error::Error`](https://doc.rust-lang.org/std/error/trait.Error.html)
/// - [`std::io::Error`](https://doc.rust-lang.org/std/io/struct.Error.html) implements
///   [`From<getrandom::Error>`](https://doc.rust-lang.org/std/convert/trait.From.html).
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Error(NonZeroU32);

const fn internal_error(n: u16) -> Error {
    // SAFETY: code > 0 as INTERNAL_START > 0 and adding n won't overflow a u32.
    let code = Error::INTERNAL_START + (n as u32);
    Error(unsafe { NonZeroU32::new_unchecked(code) })
}

impl Error {
    /// This target/platform is not supported by `getrandom`.
    pub const UNSUPPORTED: Error = internal_error(0);
    /// The platform-specific `errno` returned a non-positive value.
    pub const ERRNO_NOT_POSITIVE: Error = internal_error(1);
    /// Call to iOS [`SecRandomCopyBytes`](https://developer.apple.com/documentation/security/1399291-secrandomcopybytes) failed.
    pub const IOS_SEC_RANDOM: Error = internal_error(3);
    /// Call to Windows [`RtlGenRandom`](https://docs.microsoft.com/en-us/windows/win32/api/ntsecapi/nf-ntsecapi-rtlgenrandom) failed.
    pub const WINDOWS_RTL_GEN_RANDOM: Error = internal_error(4);
    /// RDRAND instruction failed due to a hardware issue.
    pub const FAILED_RDRAND: Error = internal_error(5);
    /// RDRAND instruction unsupported on this target.
    pub const NO_RDRAND: Error = internal_error(6);
    /// The browser does not have support for `self.crypto`.
    pub const WEB_CRYPTO: Error = internal_error(7);
    /// The browser does not have support for `crypto.getRandomValues`.
    pub const WEB_GET_RANDOM_VALUES: Error = internal_error(8);
    /// On VxWorks, call to `randSecure` failed (random number generator is not yet initialized).
    pub const VXWORKS_RAND_SECURE: Error = internal_error(11);
    /// NodeJS does not have support for the `crypto` module.
    pub const NODE_CRYPTO: Error = internal_error(12);
    /// NodeJS does not have support for `crypto.randomFillSync`.
    pub const NODE_RANDOM_FILL_SYNC: Error = internal_error(13);

    /// Codes below this point represent OS Errors (i.e. positive i32 values).
    /// Codes at or above this point, but below [`Error::CUSTOM_START`] are
    /// reserved for use by the `rand` and `getrandom` crates.
    pub const INTERNAL_START: u32 = 1 << 31;

    /// Codes at or above this point can be used by users to define their own
    /// custom errors.
    pub const CUSTOM_START: u32 = (1 << 31) + (1 << 30);

    /// Extract the raw OS error code (if this error came from the OS)
    ///
    /// This method is identical to [`std::io::Error::raw_os_error()`][1], except
    /// that it works in `no_std` contexts. If this method returns `None`, the
    /// error value can still be formatted via the `Display` implementation.
    ///
    /// [1]: https://doc.rust-lang.org/std/io/struct.Error.html#method.raw_os_error
    #[inline]
    pub fn raw_os_error(self) -> Option<i32> {
        if self.0.get() < Self::INTERNAL_START {
            match () {
                #[cfg(target_os = "solid_asp3")]
                // On SOLID, negate the error code again to obtain the original
                // error code.
                () => Some(-(self.0.get() as i32)),
                #[cfg(not(target_os = "solid_asp3"))]
                () => Some(self.0.get() as i32),
            }
        } else {
            None
        }
    }

    /// Extract the bare error code.
    ///
    /// This code can either come from the underlying OS, or be a custom error.
    /// Use [`Error::raw_os_error()`] to disambiguate.
    #[inline]
    pub const fn code(self) -> NonZeroU32 {
        self.0
    }
}

cfg_if! {
    if #[cfg(unix)] {
        fn os_err(errno: i32, buf: &mut [u8]) -> Option<&str> {
            let buf_ptr = buf.as_mut_ptr() as *mut libc::c_char;
            if unsafe { libc::strerror_r(errno, buf_ptr, buf.len()) } != 0 {
                return None;
            }

            // Take up to trailing null byte
            let n = buf.len();
            let idx = buf.iter().position(|&b| b == 0).unwrap_or(n);
            core::str::from_utf8(&buf[..idx]).ok()
        }
    } else {
        fn os_err(_errno: i32, _buf: &mut [u8]) -> Option<&str> {
            None
        }
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut dbg = f.debug_struct("Error");
        if let Some(errno) = self.raw_os_error() {
            dbg.field("os_error", &errno);
            let mut buf = [0u8; 128];
            if let Some(err) = os_err(errno, &mut buf) {
                dbg.field("description", &err);
            }
        } else if let Some(desc) = internal_desc(*self) {
            dbg.field("internal_code", &self.0.get());
            dbg.field("description", &desc);
        } else {
            dbg.field("unknown_code", &self.0.get());
        }
        dbg.finish()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(errno) = self.raw_os_error() {
            let mut buf = [0u8; 128];
            match os_err(errno, &mut buf) {
                Some(err) => err.fmt(f),
                None => write!(f, "OS Error: {}", errno),
            }
        } else if let Some(desc) = internal_desc(*self) {
            f.write_str(desc)
        } else {
            write!(f, "Unknown Error: {}", self.0.get())
        }
    }
}

impl From<NonZeroU32> for Error {
    fn from(code: NonZeroU32) -> Self {
        Self(code)
    }
}

fn internal_desc(error: Error) -> Option<&'static str> {
    match error {
        Error::UNSUPPORTED => Some("getrandom: this target is not supported"),
        Error::ERRNO_NOT_POSITIVE => Some("errno: did not return a positive value"),
        Error::IOS_SEC_RANDOM => Some("SecRandomCopyBytes: iOS Security framework failure"),
        Error::WINDOWS_RTL_GEN_RANDOM => Some("RtlGenRandom: Windows system function failure"),
        Error::FAILED_RDRAND => Some("RDRAND: failed multiple times: CPU issue likely"),
        Error::NO_RDRAND => Some("RDRAND: instruction not supported"),
        Error::WEB_CRYPTO => Some("Web Crypto API is unavailable"),
        Error::WEB_GET_RANDOM_VALUES => Some("Web API crypto.getRandomValues is unavailable"),
        Error::VXWORKS_RAND_SECURE => Some("randSecure: VxWorks RNG module is not initialized"),
        Error::NODE_CRYPTO => Some("Node.js crypto module is unavailable"),
        Error::NODE_RANDOM_FILL_SYNC => Some("Node.js API crypto.randomFillSync is unavailable"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::Error;
    use core::mem::size_of;

    #[test]
    fn test_size() {
        assert_eq!(size_of::<Error>(), 4);
        assert_eq!(size_of::<Result<(), Error>>(), 4);
    }
}
