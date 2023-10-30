//! Mostly platform-specific functionality
#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd"))]
pub mod aio;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod epoll;

#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"))]
#[allow(missing_docs)]
pub mod event;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod eventfd;

#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "redox",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "illumos",
          target_os = "openbsd"))]
#[macro_use]
pub mod ioctl;

#[cfg(target_os = "linux")]
pub mod memfd;

#[cfg(not(target_os = "redox"))]
#[allow(missing_docs)]
pub mod mman;

#[cfg(target_os = "linux")]
#[allow(missing_docs)]
pub mod personality;

pub mod pthread;

#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "linux",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"))]
#[allow(missing_docs)]
pub mod ptrace;

#[cfg(target_os = "linux")]
pub mod quota;

#[cfg(any(target_os = "linux"))]
#[allow(missing_docs)]
pub mod reboot;

#[cfg(not(any(target_os = "redox", target_os = "fuchsia", target_os = "illumos")))]
pub mod resource;

#[cfg(not(target_os = "redox"))]
pub mod select;

#[cfg(any(target_os = "android",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos"))]
pub mod sendfile;

pub mod signal;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod signalfd;

#[cfg(not(target_os = "redox"))]
#[allow(missing_docs)]
pub mod socket;

#[allow(missing_docs)]
pub mod stat;

#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "ios",
          target_os = "linux",
          target_os = "macos",
          target_os = "openbsd"
))]
pub mod statfs;

pub mod statvfs;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod sysinfo;

#[allow(missing_docs)]
pub mod termios;

#[allow(missing_docs)]
pub mod time;

pub mod uio;

pub mod utsname;

pub mod wait;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod inotify;

#[cfg(any(target_os = "android", target_os = "linux"))]
#[allow(missing_docs)]
pub mod timerfd;
