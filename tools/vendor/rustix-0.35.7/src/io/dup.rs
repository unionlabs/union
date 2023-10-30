//! Functions which duplicate file descriptors.

use crate::backend;
use crate::io::{self, OwnedFd};
use backend::fd::AsFd;

#[cfg(not(target_os = "wasi"))]
pub use backend::io::types::DupFlags;

/// `dup(fd)`—Creates a new `OwnedFd` instance that shares the same
/// underlying [file description] as `fd`.
///
/// This function does not set the `O_CLOEXEC` flag. To do a `dup` that does
/// set `O_CLOEXEC`, use [`fcntl_dupfd_cloexec`].
///
/// POSIX guarantees that `dup` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors may
/// be unexpectedly allocated on other threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [`fcntl_dupfd_cloexec`]: crate::fs::fcntl_dupfd_cloexec
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup.html
/// [Linux]: https://man7.org/linux/man-pages/man2/dup.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/dup.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn dup<Fd: AsFd>(fd: Fd) -> io::Result<OwnedFd> {
    backend::io::syscalls::dup(fd.as_fd())
}

/// `dup2(fd, new)`—Changes the [file description] of a file descriptor.
///
/// `dup2` conceptually closes `new` and then sets the file description for
/// `new` to be the same as the one for `fd`. This is a very unusual operation,
/// and should only be used on file descriptors where you know how `new` will
/// be subsequently used.
///
/// This function does not set the `O_CLOEXEC` flag. To do a `dup2` that does
/// set `O_CLOEXEC`, use [`dup3`] with [`DupFlags::CLOEXEC`] on platforms which
/// support it, or [`fcntl_dupfd_cloexec`]
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [`fcntl_dupfd_cloexec`]: crate::fs::fcntl_dupfd_cloexec
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/dup2.html
/// [Linux]: https://man7.org/linux/man-pages/man2/dup2.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/dup2.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn dup2<Fd: AsFd>(fd: Fd, new: &mut OwnedFd) -> io::Result<()> {
    backend::io::syscalls::dup2(fd.as_fd(), new)
}

/// `dup3(fd, new, flags)`—Changes the [file description] of a file
/// descriptor, with flags.
///
/// `dup3` is the same as [`dup2`] but adds an additional flags operand, and it
/// fails in the case that `fd` and `new` have the same file descriptor value.
/// This additional difference is the reason this function isn't named
/// `dup2_with`.
///
/// # References
///  - [Linux]
///
/// [file description]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_258
/// [Linux]: https://man7.org/linux/man-pages/man2/dup3.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn dup3<Fd: AsFd>(fd: Fd, new: &mut OwnedFd, flags: DupFlags) -> io::Result<()> {
    backend::io::syscalls::dup3(fd.as_fd(), new, flags)
}
