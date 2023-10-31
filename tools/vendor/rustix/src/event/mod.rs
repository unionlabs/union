//! Event operations.

#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
mod eventfd;
#[cfg(bsd)]
pub mod kqueue;
mod poll;
#[cfg(solarish)]
pub mod port;

#[cfg(linux_kernel)]
pub use crate::backend::event::epoll;
#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
pub use eventfd::{eventfd, EventfdFlags};
pub use poll::{poll, PollFd, PollFlags};
