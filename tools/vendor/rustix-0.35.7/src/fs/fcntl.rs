//! The Unix `fcntl` function is effectively lots of different functions
//! hidden behind a single dynamic dispatch interface. In order to provide
//! a type-safe API, rustix makes them all separate functions so that they
//! can have dedicated static type signatures.

use crate::backend;
use crate::io::{self, OwnedFd};
use backend::fd::{AsFd, RawFd};
use backend::fs::types::{FdFlags, OFlags};

/// `fcntl(fd, F_GETFD)`—Returns a file descriptor's flags.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fcntl.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[inline]
#[doc(alias = "F_GETFD")]
pub fn fcntl_getfd<Fd: AsFd>(fd: Fd) -> io::Result<FdFlags> {
    backend::fs::syscalls::fcntl_getfd(fd.as_fd())
}

/// `fcntl(fd, F_SETFD, flags)`—Sets a file descriptor's flags.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fcntl.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[inline]
#[doc(alias = "F_SETFD")]
pub fn fcntl_setfd<Fd: AsFd>(fd: Fd, flags: FdFlags) -> io::Result<()> {
    backend::fs::syscalls::fcntl_setfd(fd.as_fd(), flags)
}

/// `fcntl(fd, F_GETFL)`—Returns a file descriptor's access mode and status.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fcntl.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[inline]
#[doc(alias = "F_GETFL")]
pub fn fcntl_getfl<Fd: AsFd>(fd: Fd) -> io::Result<OFlags> {
    backend::fs::syscalls::fcntl_getfl(fd.as_fd())
}

/// `fcntl(fd, F_SETFL, flags)`—Sets a file descriptor's status.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fcntl.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[inline]
#[doc(alias = "F_SETFL")]
pub fn fcntl_setfl<Fd: AsFd>(fd: Fd, flags: OFlags) -> io::Result<()> {
    backend::fs::syscalls::fcntl_setfl(fd.as_fd(), flags)
}

/// `fcntl(fd, F_GET_SEALS)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "linux",
))]
#[inline]
#[doc(alias = "F_GET_SEALS")]
pub fn fcntl_get_seals<Fd: AsFd>(fd: Fd) -> io::Result<SealFlags> {
    backend::fs::syscalls::fcntl_get_seals(fd.as_fd())
}

#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "linux",
))]
pub use backend::fs::types::SealFlags;

/// `fcntl(fd, F_ADD_SEALS)`
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "linux",
))]
#[inline]
#[doc(alias = "F_ADD_SEALS")]
pub fn fcntl_add_seals<Fd: AsFd>(fd: Fd, seals: SealFlags) -> io::Result<()> {
    backend::fs::syscalls::fcntl_add_seals(fd.as_fd(), seals)
}

/// `fcntl(fd, F_DUPFD_CLOEXEC)`—Creates a new `OwnedFd` instance, with value
/// at least `min`, that has `O_CLOEXEC` set and that shares the same
/// underlying [file description] as `fd`.
///
/// POSIX guarantees that `F_DUPFD_CLOEXEC` will use the lowest unused file
/// descriptor which is at least `min`, however it is not safe in general to
/// rely on this, as file descriptors may be unexpectedly allocated on other
/// threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fcntl.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fcntl.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
#[doc(alias = "F_DUPFD_CLOEXEC")]
pub fn fcntl_dupfd_cloexec<Fd: AsFd>(fd: Fd, min: RawFd) -> io::Result<OwnedFd> {
    backend::fs::syscalls::fcntl_dupfd_cloexec(fd.as_fd(), min)
}
