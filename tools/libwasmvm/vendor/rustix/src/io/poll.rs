use crate::{backend, io};

pub use backend::io::poll_fd::{PollFd, PollFlags};

/// `poll(self.fds, timeout)`
///
/// # References
///  - [Beej's Guide to Network Programming]
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/slightly-advanced-techniques.html#poll
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/poll.html
/// [Linux]: https://man7.org/linux/man-pages/man2/poll.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/poll.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsapoll
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=poll&sektion=2
/// [NetBSD]: https://man.netbsd.org/poll.2
/// [OpenBSD]: https://man.openbsd.org/poll.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=poll&section=2
/// [illumos]: https://illumos.org/man/2/poll
#[inline]
pub fn poll(fds: &mut [PollFd<'_>], timeout: i32) -> io::Result<usize> {
    backend::io::syscalls::poll(fds, timeout)
}
