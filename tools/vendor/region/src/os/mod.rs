#[cfg(windows)]
mod windows;

#[cfg(windows)]
pub use self::windows::*;

#[cfg(unix)]
mod unix;

#[cfg(unix)]
pub use self::unix::*;

#[cfg(any(target_os = "macos", target_os = "ios"))]
mod macos;

#[cfg(any(target_os = "macos", target_os = "ios"))]
pub use self::macos::*;

#[cfg(any(target_os = "linux", target_os = "android"))]
mod linux;

#[cfg(any(target_os = "linux", target_os = "android"))]
pub use self::linux::*;

#[cfg(target_os = "freebsd")]
mod freebsd;

#[cfg(target_os = "freebsd")]
pub use self::freebsd::*;

#[cfg(target_os = "illumos")]
mod illumos;

#[cfg(target_os = "illumos")]
pub use self::illumos::*;

#[cfg(target_os = "openbsd")]
mod openbsd;

#[cfg(target_os = "openbsd")]
pub use self::openbsd::*;
