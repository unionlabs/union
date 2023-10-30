//! Efficient decimal integer formatting.
//!
//! # Safety
//!
//! This uses `CStr::from_bytes_with_nul_unchecked` and
//! `str::from_utf8_unchecked`on the buffer that it filled itself.
#![allow(unsafe_code)]

use crate::backend::fd::{AsFd, AsRawFd};
use crate::ffi::CStr;
use core::fmt::Write;
use itoa::{Buffer, Integer};
#[cfg(all(feature = "std", unix))]
use std::os::unix::ffi::OsStrExt;
#[cfg(all(feature = "std", target_os = "wasi"))]
use std::os::wasi::ffi::OsStrExt;
#[cfg(feature = "std")]
use {core::fmt, std::ffi::OsStr, std::path::Path};

/// Format an integer into a decimal `Path` component, without constructing a
/// temporary `PathBuf` or `String`.
///
/// This is used for opening paths such as `/proc/self/fd/<fd>` on Linux.
///
/// # Example
///
/// ```
/// # #[cfg(any(feature = "fs", feature = "net"))]
/// use rustix::path::DecInt;
///
/// # #[cfg(any(feature = "fs", feature = "net"))]
/// assert_eq!(
///     format!("hello {}", DecInt::new(9876).as_ref().display()),
///     "hello 9876"
/// );
/// ```
#[derive(Clone)]
pub struct DecInt {
    // 20 `u8`s is enough to hold the decimal ASCII representation of any
    // `u64`, and we add one for a NUL terminator for `as_c_str`.
    buf: [u8; 20 + 1],
    len: usize,
}

impl DecInt {
    /// Construct a new path component from an integer.
    #[inline]
    pub fn new<Int: Integer>(i: Int) -> Self {
        let mut me = DecIntWriter(Self {
            buf: [0; 20 + 1],
            len: 0,
        });
        let mut buf = Buffer::new();
        me.write_str(buf.format(i)).unwrap();
        me.0
    }

    /// Construct a new path component from a file descriptor.
    #[inline]
    pub fn from_fd<Fd: AsFd>(fd: Fd) -> Self {
        Self::new(fd.as_fd().as_raw_fd())
    }

    /// Return the raw byte buffer as a `&str`.
    #[inline]
    pub fn as_str(&self) -> &str {
        // SAFETY: `DecInt` always holds a formatted decimal number, so it's
        // always valid UTF-8.
        unsafe { core::str::from_utf8_unchecked(self.as_bytes()) }
    }

    /// Return the raw byte buffer as a `&CStr`.
    #[inline]
    pub fn as_c_str(&self) -> &CStr {
        let bytes_with_nul = &self.buf[..=self.len];
        debug_assert!(CStr::from_bytes_with_nul(bytes_with_nul).is_ok());

        // SAFETY: `self.buf` holds a single decimal ASCII representation and
        // at least one extra NUL byte.
        unsafe { CStr::from_bytes_with_nul_unchecked(bytes_with_nul) }
    }

    /// Return the raw byte buffer.
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        &self.buf[..self.len]
    }
}

struct DecIntWriter(DecInt);

impl core::fmt::Write for DecIntWriter {
    #[inline]
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        match self.0.buf.get_mut(self.0.len..self.0.len + s.len()) {
            Some(slice) => {
                slice.copy_from_slice(s.as_bytes());
                self.0.len += s.len();
                Ok(())
            }
            None => Err(core::fmt::Error),
        }
    }
}

#[cfg(feature = "std")]
impl AsRef<Path> for DecInt {
    #[inline]
    fn as_ref(&self) -> &Path {
        let as_os_str: &OsStr = OsStrExt::from_bytes(&self.buf[..self.len]);
        Path::new(as_os_str)
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for DecInt {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.as_str().fmt(fmt)
    }
}
