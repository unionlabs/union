//! Tests for [`rustix::fs`].

#![cfg(feature = "fs")]
#![cfg(not(windows))]
#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
#![cfg_attr(core_c_str, feature(core_c_str))]

mod cwd;
mod dir;
mod fcntl;
mod file;
#[cfg(not(target_os = "wasi"))]
mod flock;
mod futimens;
mod invalid_offset;
mod long_paths;
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "illumos",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "redox",
    target_os = "wasi",
)))]
mod makedev;
mod mkdirat;
mod mknodat;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod openat;
#[cfg(any(target_os = "android", target_os = "linux"))]
mod openat2;
mod readdir;
mod renameat;
#[cfg(not(any(target_os = "illumos", target_os = "redox", target_os = "wasi")))]
mod statfs;
mod utimensat;
mod y2038;
