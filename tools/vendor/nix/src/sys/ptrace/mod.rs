///! Provides helpers for making ptrace system calls 

#[cfg(any(target_os = "android", target_os = "linux"))]
mod linux;

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use self::linux::*;

#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"))]
mod bsd;

#[cfg(any(target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "macos",
          target_os = "netbsd",
          target_os = "openbsd"
          ))]
pub use self::bsd::*;
