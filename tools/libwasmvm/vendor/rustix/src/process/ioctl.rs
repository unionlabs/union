use crate::{backend, io};
use backend::fd::AsFd;

/// `ioctl(fd, TIOCSCTTY, 0)`—Sets the controlling terminal for the processs.
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
#[doc(alias = "TIOCSCTTY")]
pub fn ioctl_tiocsctty<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::process::syscalls::ioctl_tiocsctty(fd.as_fd())
}
