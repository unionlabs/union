//! `read` and `write`, optionally positioned, optionally vectored

use crate::{backend, io};
use backend::fd::AsFd;

// Declare `IoSlice` and `IoSliceMut`.
#[cfg(not(windows))]
#[cfg(not(feature = "std"))]
pub use backend::io::io_slice::{IoSlice, IoSliceMut};
#[cfg(not(windows))]
#[cfg(feature = "std")]
pub use std::io::{IoSlice, IoSliceMut};

/// `RWF_*` constants for use with [`preadv2`] and [`pwritev2`].
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use backend::io::types::ReadWriteFlags;

/// `read(fd, buf)`—Reads from a stream.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html
/// [Linux]: https://man7.org/linux/man-pages/man2/read.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/read.2.html
#[inline]
pub fn read<Fd: AsFd>(fd: Fd, buf: &mut [u8]) -> io::Result<usize> {
    backend::io::syscalls::read(fd.as_fd(), buf)
}

/// `write(fd, buf)`—Writes to a stream.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
/// [Linux]: https://man7.org/linux/man-pages/man2/write.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/write.2.html
#[inline]
pub fn write<Fd: AsFd>(fd: Fd, buf: &[u8]) -> io::Result<usize> {
    backend::io::syscalls::write(fd.as_fd(), buf)
}

/// `pread(fd, buf, offset)`—Reads from a file at a given position.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pread.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pread.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pread.2.html
#[inline]
pub fn pread<Fd: AsFd>(fd: Fd, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::pread(fd.as_fd(), buf, offset)
}

/// `pwrite(fd, bufs)`—Writes to a file at a given position.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pwrite.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pwrite.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pwrite.2.html
#[inline]
pub fn pwrite<Fd: AsFd>(fd: Fd, buf: &[u8], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::pwrite(fd.as_fd(), buf, offset)
}

/// `readv(fd, bufs)`—Reads from a stream into multiple buffers.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/readv.html
/// [Linux]: https://man7.org/linux/man-pages/man2/readv.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/readv.2.html
#[inline]
pub fn readv<Fd: AsFd>(fd: Fd, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    backend::io::syscalls::readv(fd.as_fd(), bufs)
}

/// `writev(fd, bufs)`—Writes to a stream from multiple buffers.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/writev.html
/// [Linux]: https://man7.org/linux/man-pages/man2/writev.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/writev.2.html
#[inline]
pub fn writev<Fd: AsFd>(fd: Fd, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    backend::io::syscalls::writev(fd.as_fd(), bufs)
}

/// `preadv(fd, bufs, offset)`—Reads from a file at a given position into
/// multiple buffers.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/preadv.2.html
#[cfg(not(target_os = "redox"))]
#[inline]
pub fn preadv<Fd: AsFd>(fd: Fd, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::preadv(fd.as_fd(), bufs, offset)
}

/// `pwritev(fd, bufs, offset)`—Writes to a file at a given position from
/// multiple buffers.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pwritev.2.html
#[cfg(not(target_os = "redox"))]
#[inline]
pub fn pwritev<Fd: AsFd>(fd: Fd, bufs: &[IoSlice<'_>], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::pwritev(fd.as_fd(), bufs, offset)
}

/// `preadv2(fd, bufs, offset, flags)`—Reads data, with several options.
///
/// An `offset` of `u64::MAX` means to use and update the current file offset.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/preadv2.2.html
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn preadv2<Fd: AsFd>(
    fd: Fd,
    bufs: &mut [IoSliceMut<'_>],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    backend::io::syscalls::preadv2(fd.as_fd(), bufs, offset, flags)
}

/// `pwritev2(fd, bufs, offset, flags)`—Writes data, with several options.
///
/// An `offset` of `u64::MAX` means to use and update the current file offset.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pwritev2.2.html
#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub fn pwritev2<Fd: AsFd>(
    fd: Fd,
    bufs: &[IoSlice<'_>],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    backend::io::syscalls::pwritev2(fd.as_fd(), bufs, offset, flags)
}
