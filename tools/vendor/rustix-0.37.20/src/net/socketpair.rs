use crate::fd::OwnedFd;
use crate::net::{AddressFamily, Protocol, SocketFlags, SocketType};
use crate::{backend, io};

/// `socketpair(domain, type_ | accept_flags, protocol)`
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
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/socketpair.html
/// [Linux]: https://man7.org/linux/man-pages/man2/socketpair.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/socketpair.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=socketpair&sektion=2
/// [NetBSD]: https://man.netbsd.org/socketpair.2
/// [OpenBSD]: https://man.openbsd.org/socketpair.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=socketpair&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/socketpair
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Socket-Pairs.html
#[inline]
pub fn socketpair(
    domain: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<(OwnedFd, OwnedFd)> {
    backend::net::syscalls::socketpair(domain, type_, flags, protocol)
}
