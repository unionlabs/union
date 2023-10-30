use crate::backend;
use crate::io::{self, OwnedFd};
use crate::net::{AddressFamily, Protocol, SocketFlags, SocketType};

/// `socketpair(domain, type_ | accept_flags, protocol)`
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/socketpair.html
/// [Linux]: https://man7.org/linux/man-pages/man2/socketpair.2.html
#[inline]
pub fn socketpair(
    domain: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<(OwnedFd, OwnedFd)> {
    backend::net::syscalls::socketpair(domain, type_, flags, protocol)
}
