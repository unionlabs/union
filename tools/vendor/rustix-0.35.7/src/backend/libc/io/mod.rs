pub(crate) mod errno;
#[cfg(not(windows))]
#[cfg(not(feature = "std"))]
pub(crate) mod io_slice;
pub(crate) mod poll_fd;
#[cfg(not(windows))]
pub(crate) mod types;

#[cfg_attr(windows, path = "windows_syscalls.rs")]
pub(crate) mod syscalls;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub mod epoll;
