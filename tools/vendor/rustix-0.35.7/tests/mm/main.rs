//! Tests for [`rustix::mm`].

#![cfg(feature = "mm")]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
mod mlock;
#[cfg(not(windows))]
mod mmap;
#[cfg(not(windows))]
mod prot;
