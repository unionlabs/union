//! Tests for [`rustix::time`].

#![cfg(feature = "time")]
#![cfg(not(windows))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

mod dynamic_clocks;
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
mod monotonic;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod timerfd;
mod timespec;
mod y2038;
