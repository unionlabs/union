//! Terminal-related `ioctl` functions.

use crate::fd::AsFd;
use crate::{backend, io};

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
    backend::termios::syscalls::ioctl_tiocexcl(fd.as_fd())
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
    backend::termios::syscalls::ioctl_tiocnxcl(fd.as_fd())
}
