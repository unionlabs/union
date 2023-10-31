//! Rust friendly bindings to the various *nix system functions.
//!
//! Modules are structured according to the C header file that they would be
//! defined in.
#![crate_name = "nix"]
#![cfg(unix)]
#![allow(non_camel_case_types)]
#![cfg_attr(test, deny(warnings))]
#![recursion_limit = "500"]
#![deny(unused)]
#![deny(unstable_features)]
#![deny(missing_copy_implementations)]
#![deny(missing_debug_implementations)]
#![warn(missing_docs)]

// Re-exported external crates
pub use libc;

// Private internal modules
#[macro_use] mod macros;

// Public crates
#[cfg(not(target_os = "redox"))]
#[allow(missing_docs)]
pub mod dir;
pub mod env;
#[allow(missing_docs)]
pub mod errno;
pub mod features;
#[allow(missing_docs)]
pub mod fcntl;
#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "illumos",
          target_os = "openbsd"))]
pub mod ifaddrs;
#[cfg(any(target_os = "android",
          target_os = "linux"))]
#[allow(missing_docs)]
pub mod kmod;
#[cfg(any(target_os = "android",
          target_os = "freebsd",
          target_os = "linux"))]
pub mod mount;
#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "fushsia",
          target_os = "linux",
          target_os = "netbsd"))]
#[allow(missing_docs)]
pub mod mqueue;
#[cfg(not(target_os = "redox"))]
pub mod net;
pub mod poll;
#[cfg(not(any(target_os = "redox", target_os = "fuchsia")))]
pub mod pty;
pub mod sched;
pub mod sys;
#[allow(missing_docs)]
pub mod time;
// This can be implemented for other platforms as soon as libc
// provides bindings for them.
#[cfg(all(target_os = "linux",
          any(target_arch = "x86", target_arch = "x86_64")))]
#[allow(missing_docs)]
pub mod ucontext;
#[allow(missing_docs)]
pub mod unistd;

/*
 *
 * ===== Result / Error =====
 *
 */

use libc::{c_char, PATH_MAX};

use std::{ptr, result};
use std::ffi::{CStr, OsStr};
use std::os::unix::ffi::OsStrExt;
use std::path::{Path, PathBuf};

use errno::Errno;

/// Nix Result Type
pub type Result<T> = result::Result<T, Errno>;

/// Nix's main error type.
///
/// It's a wrapper around Errno.  As such, it's very interoperable with
/// [`std::io::Error`], but it has the advantages of:
/// * `Clone`
/// * `Copy`
/// * `Eq`
/// * Small size
/// * Represents all of the system's errnos, instead of just the most common
/// ones.
pub type Error = Errno;

/// Common trait used to represent file system paths by many Nix functions.
pub trait NixPath {
    /// Is the path empty?
    fn is_empty(&self) -> bool;

    /// Length of the path in bytes
    fn len(&self) -> usize;

    /// Execute a function with this path as a `CStr`.
    ///
    /// Mostly used internally by Nix.
    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T;
}

impl NixPath for str {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(OsStr::new(self))
    }

    fn len(&self) -> usize {
        NixPath::len(OsStr::new(self))
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T {
            OsStr::new(self).with_nix_path(f)
        }
}

impl NixPath for OsStr {
    fn is_empty(&self) -> bool {
        self.as_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.as_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
        where F: FnOnce(&CStr) -> T {
            self.as_bytes().with_nix_path(f)
        }
}

impl NixPath for CStr {
    fn is_empty(&self) -> bool {
        self.to_bytes().is_empty()
    }

    fn len(&self) -> usize {
        self.to_bytes().len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
        // Equivalence with the [u8] impl.
        if self.len() >= PATH_MAX as usize {
            return Err(Errno::ENAMETOOLONG)
        }

        Ok(f(self))
    }
}

impl NixPath for [u8] {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }

    fn len(&self) -> usize {
        self.len()
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T>
            where F: FnOnce(&CStr) -> T {
        let mut buf = [0u8; PATH_MAX as usize];

        if self.len() >= PATH_MAX as usize {
            return Err(Errno::ENAMETOOLONG)
        }

        match self.iter().position(|b| *b == 0) {
            Some(_) => Err(Errno::EINVAL),
            None => {
                unsafe {
                    // TODO: Replace with bytes::copy_memory. rust-lang/rust#24028
                    ptr::copy_nonoverlapping(self.as_ptr(), buf.as_mut_ptr(), self.len());
                    Ok(f(CStr::from_ptr(buf.as_ptr() as *const c_char)))
                }

            }
        }
    }
}

impl NixPath for Path {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().with_nix_path(f)
    }
}

impl NixPath for PathBuf {
    fn is_empty(&self) -> bool {
        NixPath::is_empty(self.as_os_str())
    }

    fn len(&self) -> usize {
        NixPath::len(self.as_os_str())
    }

    fn with_nix_path<T, F>(&self, f: F) -> Result<T> where F: FnOnce(&CStr) -> T {
        self.as_os_str().with_nix_path(f)
    }
}
