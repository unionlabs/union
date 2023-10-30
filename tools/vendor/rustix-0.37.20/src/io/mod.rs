//! I/O operations.

mod close;
#[cfg(not(windows))]
mod dup;
mod errno;
#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
mod eventfd;
#[cfg(not(windows))]
mod fcntl;
#[cfg(not(feature = "std"))]
pub(crate) mod fd;
mod ioctl;
#[cfg(not(any(windows, target_os = "redox")))]
#[cfg(all(feature = "fs", feature = "net"))]
mod is_read_write;
#[cfg(bsd)]
pub mod kqueue;
#[cfg(not(any(windows, target_os = "wasi")))]
mod pipe;
mod poll;
#[cfg(solarish)]
pub mod port;
#[cfg(all(feature = "procfs", linux_kernel))]
mod procfs;
#[cfg(not(windows))]
mod read_write;
mod seek_from;
#[cfg(not(windows))]
mod stdio;

#[cfg(linux_kernel)]
pub use crate::backend::io::epoll;
pub use close::close;
#[cfg(not(windows))]
pub use dup::*;
pub use errno::{retry_on_intr, Errno, Result};
#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
pub use eventfd::{eventfd, EventfdFlags};
#[cfg(not(windows))]
pub use fcntl::*;
pub use ioctl::*;
#[cfg(not(any(windows, target_os = "redox")))]
#[cfg(all(feature = "fs", feature = "net"))]
pub use is_read_write::*;
#[cfg(not(any(windows, target_os = "wasi")))]
pub use pipe::*;
pub use poll::{poll, PollFd, PollFlags};
#[cfg(all(feature = "procfs", linux_kernel))]
pub use procfs::*;
#[cfg(not(windows))]
pub use read_write::*;
pub use seek_from::SeekFrom;
#[cfg(not(windows))]
pub use stdio::*;
