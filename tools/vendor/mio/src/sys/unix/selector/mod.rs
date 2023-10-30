#[cfg(any(
    target_os = "android",
    target_os = "illumos",
    target_os = "linux",
    target_os = "redox",
))]
mod epoll;

#[cfg(any(
    target_os = "android",
    target_os = "illumos",
    target_os = "linux",
    target_os = "redox",
))]
pub(crate) use self::epoll::{event, Event, Events, Selector};

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
mod kqueue;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd"
))]
pub(crate) use self::kqueue::{event, Event, Events, Selector};

/// Lowest file descriptor used in `Selector::try_clone`.
///
/// # Notes
///
/// Usually fds 0, 1 and 2 are standard in, out and error. Some application
/// blindly assume this to be true, which means using any one of those a select
/// could result in some interesting and unexpected errors. Avoid that by using
/// an fd that doesn't have a pre-determined usage.
const LOWEST_FD: libc::c_int = 3;
