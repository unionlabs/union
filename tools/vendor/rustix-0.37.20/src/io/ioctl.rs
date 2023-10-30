//! The Unix `ioctl` function is effectively lots of different functions
//! hidden behind a single dynamic dispatch interface. In order to provide
//! a type-safe API, rustix makes them all separate functions so that they
//! can have dedicated static type signatures.

use crate::{backend, io};
use backend::fd::AsFd;

/// `ioctl(fd, TIOCEXCL)`—Enables exclusive mode on a terminal.
///
/// # References
///  - [Linux]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=tty&sektion=4
/// [NetBSD]: https://man.netbsd.org/tty.4
/// [OpenBSD]: https://man.openbsd.org/tty.4
#[cfg(not(any(windows, target_os = "haiku", target_os = "redox", target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCEXCL")]
pub fn ioctl_tiocexcl<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::io::syscalls::ioctl_tiocexcl(fd.as_fd())
}

/// `ioctl(fd, TIOCNXCL)`—Disables exclusive mode on a terminal.
///
/// # References
///  - [Linux]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=tty&sektion=4
/// [NetBSD]: https://man.netbsd.org/tty.4
/// [OpenBSD]: https://man.openbsd.org/tty.4
#[cfg(not(any(windows, target_os = "haiku", target_os = "redox", target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCNXCL")]
pub fn ioctl_tiocnxcl<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::io::syscalls::ioctl_tiocnxcl(fd.as_fd())
}

/// `ioctl(fd, FIOCLEX, NULL)`—Set the close-on-exec flag.
///
/// Also known as `fcntl(fd, F_SETFD, FD_CLOEXEC)`.
///
/// # References
///  - [Winsock2]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-ioctlsocket
/// [NetBSD]: https://man.netbsd.org/ioctl.2#GENERIC%20IOCTLS
/// [OpenBSD]: https://man.openbsd.org/ioctl.2#GENERIC_IOCTLS
#[cfg(apple)]
#[inline]
#[doc(alias = "FIOCLEX")]
#[doc(alias = "FD_CLOEXEC")]
pub fn ioctl_fioclex<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::io::syscalls::ioctl_fioclex(fd.as_fd())
}

/// `ioctl(fd, FIONBIO, &value)`—Enables or disables non-blocking mode.
///
/// # References
///  - [Winsock2]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/winsock/winsock-ioctls#unix-ioctl-codes
/// [NetBSD]: https://man.netbsd.org/ioctl.2#GENERIC%20IOCTLS
/// [OpenBSD]: https://man.openbsd.org/ioctl.2#GENERIC_IOCTLS
#[inline]
#[doc(alias = "FIONBIO")]
pub fn ioctl_fionbio<Fd: AsFd>(fd: Fd, value: bool) -> io::Result<()> {
    backend::io::syscalls::ioctl_fionbio(fd.as_fd(), value)
}

/// `ioctl(fd, FIONREAD)`—Returns the number of bytes ready to be read.
///
/// The result of this function gets silently coerced into a C `int`
/// by the OS, so it may contain a wrapped value.
///
/// # References
///  - [Linux]
///  - [Winsock2]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/ioctl_tty.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/winsock/winsock-ioctls#unix-ioctl-codes
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=ioctl&sektion=2#GENERIC%09IOCTLS
/// [NetBSD]: https://man.netbsd.org/ioctl.2#GENERIC%20IOCTLS
/// [OpenBSD]: https://man.openbsd.org/ioctl.2#GENERIC_IOCTLS
#[cfg(not(target_os = "redox"))]
#[inline]
#[doc(alias = "FIONREAD")]
pub fn ioctl_fionread<Fd: AsFd>(fd: Fd) -> io::Result<u64> {
    backend::io::syscalls::ioctl_fionread(fd.as_fd())
}

/// `ioctl(fd, BLKSSZGET)`—Returns the logical block size of a block device.
///
/// This is mentioned in the [Linux `openat` manual page].
///
/// [Linux `openat` manual page]: https://man7.org/linux/man-pages/man2/openat.2.html
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "BLKSSZGET")]
pub fn ioctl_blksszget<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::io::syscalls::ioctl_blksszget(fd.as_fd())
}

/// `ioctl(fd, BLKPBSZGET)`—Returns the physical block size of a block device.
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "BLKPBSZGET")]
pub fn ioctl_blkpbszget<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::io::syscalls::ioctl_blkpbszget(fd.as_fd())
}

/// `ioctl(fd, FICLONE, src_fd)`—Share data between open files.
///
/// This ioctl is not available on Sparc platforms
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/ioctl_ficlone.2.html
#[cfg(all(linux_kernel, not(any(target_arch = "sparc", target_arch = "sparc64"))))]
#[inline]
#[doc(alias = "FICLONE")]
pub fn ioctl_ficlone<Fd: AsFd, SrcFd: AsFd>(fd: Fd, src_fd: SrcFd) -> io::Result<()> {
    backend::io::syscalls::ioctl_ficlone(fd.as_fd(), src_fd.as_fd())
}

/// `ioctl(fd, EXT4_IOC_RESIZE_FS, blocks)`—Resize ext4 filesystem on fd.
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "EXT4_IOC_RESIZE_FS")]
pub fn ext4_ioc_resize_fs<Fd: AsFd>(fd: Fd, blocks: u64) -> io::Result<()> {
    backend::io::syscalls::ext4_ioc_resize_fs(fd.as_fd(), blocks)
}
