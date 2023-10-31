use crate::backend;
use crate::io::{self, OwnedFd};

#[cfg(not(any(target_os = "ios", target_os = "macos")))]
pub use backend::io::types::PipeFlags;

/// `PIPE_BUF`—The maximum length at which writes to a pipe are atomic.
///
/// # References
///  - [Linux]
///  - [POSIX]
///
/// [Linux]: https://man7.org/linux/man-pages/man7/pipe.7.html
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
#[cfg(not(any(
    windows,
    target_os = "illumos",
    target_os = "redox",
    target_os = "wasi",
)))]
pub const PIPE_BUF: usize = backend::io::types::PIPE_BUF;

/// `pipe()`—Creates a pipe.
///
/// This function creates a pipe and returns two file descriptors, for the
/// reading and writing ends of the pipe, respectively.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pipe.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pipe.2.html
#[inline]
pub fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    backend::io::syscalls::pipe()
}

/// `pipe2(flags)`—Creates a pipe, with flags.
///
/// This function creates a pipe and returns two file descriptors, for the
/// reading and writing ends of the pipe, respectively.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pipe2.2.html
#[cfg(not(any(target_os = "ios", target_os = "macos")))]
#[inline]
#[doc(alias = "pipe2")]
pub fn pipe_with(flags: PipeFlags) -> io::Result<(OwnedFd, OwnedFd)> {
    backend::io::syscalls::pipe_with(flags)
}
