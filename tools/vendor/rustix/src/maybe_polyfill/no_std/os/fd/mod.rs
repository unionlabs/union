//! The following is derived from Rust's
//! library/std/src/os/fd/mod.rs at revision
//! fa68e73e9947be8ffc5b3b46d899e4953a44e7e9.
//!
//! All code in this file is licensed MIT or Apache 2.0 at your option.
//!
//! Owned and borrowed Unix-like file descriptors.

#![cfg_attr(staged_api, unstable(feature = "io_safety", issue = "87074"))]
#![deny(unsafe_op_in_unsafe_fn)]

// `RawFd`, `AsRawFd`, etc.
mod raw;

// `OwnedFd`, `AsFd`, etc.
mod owned;

pub use owned::*;
pub use raw::*;
