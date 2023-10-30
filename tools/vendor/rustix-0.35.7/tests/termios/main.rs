//! Tests for [`rustix::termios`].

#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg(feature = "termios")]

#[cfg(not(windows))]
mod isatty;
#[cfg(not(any(windows, target_os = "fuchsia")))]
mod ttyname;
