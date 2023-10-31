//! Tests for [`rustix::io`].

#![cfg_attr(target_os = "wasi", feature(wasi_ext))]
#![cfg_attr(io_lifetimes_use_std, feature(io_safety))]

#[cfg(not(feature = "rustc-dep-of-std"))]
#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
mod dup2_to_replace_stdio;
#[cfg(not(feature = "rustc-dep-of-std"))] // TODO
#[cfg(not(windows))]
#[cfg(feature = "net")]
#[cfg(not(target_os = "wasi"))]
mod epoll;
mod error;
#[cfg(not(windows))]
#[cfg(not(target_os = "wasi"))]
mod eventfd;
#[cfg(not(windows))]
mod from_into;
#[cfg(not(target_os = "redox"))]
mod ioctl;
mod poll;
#[cfg(all(feature = "procfs", any(target_os = "android", target_os = "linux")))]
mod procfs;
#[cfg(not(windows))]
#[cfg(not(target_os = "redox"))] // redox doesn't have cwd/openat
#[cfg(not(target_os = "wasi"))] // wasi support for S_IRUSR etc. submitted to libc in #2264
mod read_write;
#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "android"))]
mod seals;
