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

#[cfg(linux_kernel)]
pub use backend::io::types::ReadWriteFlags;

/// `read(fd, buf)`—Reads from a stream.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/read.html
/// [Linux]: https://man7.org/linux/man-pages/man2/read.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/read.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=read&sektion=2
/// [NetBSD]: https://man.netbsd.org/read.2
/// [OpenBSD]: https://man.openbsd.org/read.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=read&section=2
/// [illumos]: https://illumos.org/man/2/read
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/I_002fO-Primitives.html#index-reading-from-a-file-descriptor
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
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/write.html
/// [Linux]: https://man7.org/linux/man-pages/man2/write.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/write.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=write&sektion=2
/// [NetBSD]: https://man.netbsd.org/write.2
/// [OpenBSD]: https://man.openbsd.org/write.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=write&section=2
/// [illumos]: https://illumos.org/man/2/write
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/I_002fO-Primitives.html#index-writing-to-a-file-descriptor
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
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pread.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pread.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pread.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=pread&sektion=2
/// [NetBSD]: https://man.netbsd.org/pread.2
/// [OpenBSD]: https://man.openbsd.org/pread.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=pread&section=2
/// [illumos]: https://illumos.org/man/2/pread
#[inline]
pub fn pread<Fd: AsFd>(fd: Fd, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::pread(fd.as_fd(), buf, offset)
}

/// `pwrite(fd, bufs)`—Writes to a file at a given position.
///
/// Contrary to POSIX, on many popular platforms including Linux and FreeBSD,
/// if the file is opened in append mode, this ignores the offset appends the
/// data to the end of the file.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/pwrite.html
/// [Linux]: https://man7.org/linux/man-pages/man2/pwrite.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/pwrite.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=pwrite&sektion=2
/// [NetBSD]: https://man.netbsd.org/pwrite.2
/// [OpenBSD]: https://man.openbsd.org/pwrite.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=pwrite&section=2
/// [illumos]: https://illumos.org/man/2/pwrite
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
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/readv.html
/// [Linux]: https://man7.org/linux/man-pages/man2/readv.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/readv.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=readv&sektion=2
/// [NetBSD]: https://man.netbsd.org/readv.2
/// [OpenBSD]: https://man.openbsd.org/readv.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=readv&section=2
/// [illumos]: https://illumos.org/man/2/readv
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
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/writev.html
/// [Linux]: https://man7.org/linux/man-pages/man2/writev.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/writev.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=writev&sektion=2
/// [NetBSD]: https://man.netbsd.org/writev.2
/// [OpenBSD]: https://man.openbsd.org/writev.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=writev&section=2
/// [illumos]: https://illumos.org/man/2/writev
#[inline]
pub fn writev<Fd: AsFd>(fd: Fd, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    backend::io::syscalls::writev(fd.as_fd(), bufs)
}

/// `preadv(fd, bufs, offset)`—Reads from a file at a given position into
/// multiple buffers.
///
/// # References
///  - [Linux]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/preadv.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=preadv&sektion=2
/// [NetBSD]: https://man.netbsd.org/preadv.2
/// [OpenBSD]: https://man.openbsd.org/preadv.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=preadv&section=2
/// [illumos]: https://illumos.org/man/2/preadv
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
#[inline]
pub fn preadv<Fd: AsFd>(fd: Fd, bufs: &mut [IoSliceMut<'_>], offset: u64) -> io::Result<usize> {
    backend::io::syscalls::preadv(fd.as_fd(), bufs, offset)
}

/// `pwritev(fd, bufs, offset)`—Writes to a file at a given position from
/// multiple buffers.
///
/// Contrary to POSIX, on many popular platforms including Linux and FreeBSD,
/// if the file is opened in append mode, this ignores the offset appends the
/// data to the end of the file.
///
/// # References
///  - [Linux]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/pwritev.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=pwritev&sektion=2
/// [NetBSD]: https://man.netbsd.org/pwritev.2
/// [OpenBSD]: https://man.openbsd.org/pwritev.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=pwritev&section=2
/// [illumos]: https://illumos.org/man/2/pwritev
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
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
#[cfg(linux_kernel)]
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
#[cfg(linux_kernel)]
#[inline]
pub fn pwritev2<Fd: AsFd>(
    fd: Fd,
    bufs: &[IoSlice<'_>],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    backend::io::syscalls::pwritev2(fd.as_fd(), bufs, offset, flags)
}
