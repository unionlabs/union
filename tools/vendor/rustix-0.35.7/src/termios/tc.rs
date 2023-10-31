use crate::fd::AsFd;
use crate::process::Pid;
use crate::{backend, io};

pub use backend::termios::types::{
    Action, OptionalActions, QueueSelector, Speed, Tcflag, Termios, Winsize,
};

/// `tcgetattr(fd)`—Get terminal attributes.
///
/// Also known as the `TCGETS` operation with `ioctl`.
///
/// # References
///  - [POSIX `tcgetattr`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcgetattr`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcgetattr.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[cfg(not(any(windows, target_os = "wasi")))]
#[inline]
#[doc(alias = "TCGETS")]
pub fn tcgetattr<Fd: AsFd>(fd: Fd) -> io::Result<Termios> {
    backend::termios::syscalls::tcgetattr(fd.as_fd())
}

/// `tcgetwinsize(fd)`—Get the current terminal window size.
///
/// Also known as the `TIOCGWINSZ` operation with `ioctl`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
#[cfg(not(any(windows, target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCGWINSZ")]
pub fn tcgetwinsize<Fd: AsFd>(fd: Fd) -> io::Result<Winsize> {
    backend::termios::syscalls::tcgetwinsize(fd.as_fd())
}

/// `tcgetpgrp(fd)`—Get the terminal foreground process group.
///
/// Also known as the `TIOCGPGRP` operation with `ioctl`.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcgetpgrp.html
/// [Linux]: https://man7.org/linux/man-pages/man3/tcgetpgrp.3.html
#[cfg(not(any(windows, target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCGPGRP")]
pub fn tcgetpgrp<Fd: AsFd>(fd: Fd) -> io::Result<Pid> {
    backend::termios::syscalls::tcgetpgrp(fd.as_fd())
}

/// `tcsetpgrp(fd, pid)`—Set the terminal foreground process group.
///
/// Also known as the `TIOCSPGRP` operation with `ioctl`.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcsetpgrp.html
/// [Linux]: https://man7.org/linux/man-pages/man3/tcsetpgrp.3.html
#[cfg(not(any(windows, target_os = "wasi")))]
#[inline]
#[doc(alias = "TIOCSPGRP")]
pub fn tcsetpgrp<Fd: AsFd>(fd: Fd, pid: Pid) -> io::Result<()> {
    backend::termios::syscalls::tcsetpgrp(fd.as_fd(), pid)
}

/// `tcsetattr(fd)`—Set terminal attributes.
///
/// Also known as the `TCSETS` operation with `ioctl`.
///
/// # References
///  - [POSIX `tcsetattr`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcsetattr`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcsetattr.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[inline]
#[doc(alias = "TCSETS")]
pub fn tcsetattr<Fd: AsFd>(
    fd: Fd,
    optional_actions: OptionalActions,
    termios: &Termios,
) -> io::Result<()> {
    backend::termios::syscalls::tcsetattr(fd.as_fd(), optional_actions, termios)
}

/// `tcsendbreak(fd, 0)`—Transmit zero-valued bits.
///
/// Also known as the `TCSBRK` operation with `ioctl`, with a duration of 0.
///
/// This function always uses an effective duration parameter of zero. For the
/// equivalent of a `tcsendbreak` with a non-zero duration parameter, use
/// `tcdrain`.
///
/// # References
///  - [POSIX `tcsendbreak`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcsendbreak`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcsendbreak.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[inline]
#[doc(alias = "TCSBRK")]
pub fn tcsendbreak<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::termios::syscalls::tcsendbreak(fd.as_fd())
}

/// `tcdrain(fd, duration)`—Wait until all pending output has been written.
///
/// # References
///  - [POSIX `tcdrain`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcsetattr`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcdrain.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[inline]
pub fn tcdrain<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::termios::syscalls::tcdrain(fd.as_fd())
}

/// `tcflush(fd, queue_selector)`—Wait until all pending output has been
/// written.
///
/// # References
///  - [POSIX `tcflush`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcflush`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcflush.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[inline]
#[doc(alias = "TCFLSH")]
pub fn tcflush<Fd: AsFd>(fd: Fd, queue_selector: QueueSelector) -> io::Result<()> {
    backend::termios::syscalls::tcflush(fd.as_fd(), queue_selector)
}

/// `tcflow(fd, action)`—Suspend or resume transmission or reception.
///
/// # References
///  - [POSIX `tcflow`]
///  - [Linux `ioctl_tty`]
///  - [Linux `termios`]
///
/// [POSIX `tcflow`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcflow.html
/// [Linux `ioctl_tty`]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
/// [Linux `termios`]: https://man7.org/linux/man-pages/man3/termios.3.html
#[inline]
#[doc(alias = "TCXONC")]
pub fn tcflow<Fd: AsFd>(fd: Fd, action: Action) -> io::Result<()> {
    backend::termios::syscalls::tcflow(fd.as_fd(), action)
}

/// `tcgetsid(fd)`—Return the session ID of the current session with `fd` as
/// its controlling terminal.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/tcgetsid.html
/// [Linux]: https://man7.org/linux/man-pages/man3/tcgetsid.3.html
#[inline]
#[doc(alias = "TIOCGSID")]
pub fn tcgetsid<Fd: AsFd>(fd: Fd) -> io::Result<Pid> {
    backend::termios::syscalls::tcgetsid(fd.as_fd())
}

/// `tcsetwinsize(fd)`—Set the current terminal window size.
///
/// Also known as the `TIOCSWINSZ` operation with `ioctl`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man4/tty_ioctl.4.html
#[inline]
#[doc(alias = "TIOCSWINSZ")]
pub fn tcsetwinsize<Fd: AsFd>(fd: Fd, winsize: Winsize) -> io::Result<()> {
    backend::termios::syscalls::tcsetwinsize(fd.as_fd(), winsize)
}
