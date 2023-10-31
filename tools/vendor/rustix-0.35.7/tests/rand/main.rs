//! Tests for [`rustix::rand`].

#![cfg(feature = "rand")]
#![cfg(not(windows))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

#[cfg(any(linux_raw, all(libc, target_os = "linux")))]
mod getrandom;
