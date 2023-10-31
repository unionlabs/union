//! `recv` and `send`, and variants.

#[cfg(unix)]
use crate::net::SocketAddrUnix;
use crate::net::{SocketAddr, SocketAddrAny, SocketAddrV4, SocketAddrV6};
use crate::{backend, io};
use backend::fd::{AsFd, BorrowedFd};

pub use backend::net::send_recv::{RecvFlags, SendFlags};

/// `recv(fd, buf, flags)`—Reads data from a socket.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recv.html
/// [Linux]: https://man7.org/linux/man-pages/man2/recv.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/recv.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-recv
#[inline]
pub fn recv<Fd: AsFd>(fd: Fd, buf: &mut [u8], flags: RecvFlags) -> io::Result<usize> {
    backend::net::syscalls::recv(fd.as_fd(), buf, flags)
}

/// `send(fd, buf, flags)`—Writes data to a socket.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/send.html
/// [Linux]: https://man7.org/linux/man-pages/man2/send.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/send.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-send
#[inline]
pub fn send<Fd: AsFd>(fd: Fd, buf: &[u8], flags: SendFlags) -> io::Result<usize> {
    backend::net::syscalls::send(fd.as_fd(), buf, flags)
}

/// `recvfrom(fd, buf, flags, addr, len)`—Reads data from a socket and
/// returns the sender address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvfrom.html
/// [Linux]: https://man7.org/linux/man-pages/man2/recvfrom.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/recvfrom.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-recvfrom
#[inline]
pub fn recvfrom<Fd: AsFd>(
    fd: Fd,
    buf: &mut [u8],
    flags: RecvFlags,
) -> io::Result<(usize, Option<SocketAddrAny>)> {
    backend::net::syscalls::recvfrom(fd.as_fd(), buf, flags)
}

/// `sendto(fd, buf, flags, addr)`—Writes data to a socket to a specific IP
/// address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
pub fn sendto<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddr,
) -> io::Result<usize> {
    _sendto(fd.as_fd(), buf, flags, addr)
}

fn _sendto(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddr,
) -> io::Result<usize> {
    match addr {
        SocketAddr::V4(v4) => backend::net::syscalls::sendto_v4(fd, buf, flags, v4),
        SocketAddr::V6(v6) => backend::net::syscalls::sendto_v6(fd, buf, flags, v6),
    }
}

/// `sendto(fd, buf, flags, addr)`—Writes data to a socket to a specific
/// address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
pub fn sendto_any<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrAny,
) -> io::Result<usize> {
    _sendto_any(fd.as_fd(), buf, flags, addr)
}

fn _sendto_any(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrAny,
) -> io::Result<usize> {
    match addr {
        SocketAddrAny::V4(v4) => backend::net::syscalls::sendto_v4(fd, buf, flags, v4),
        SocketAddrAny::V6(v6) => backend::net::syscalls::sendto_v6(fd, buf, flags, v6),
        #[cfg(unix)]
        SocketAddrAny::Unix(unix) => backend::net::syscalls::sendto_unix(fd, buf, flags, unix),
    }
}

/// `sendto(fd, buf, flags, addr, sizeof(struct sockaddr_in))`—Writes data to
/// a socket to a specific IPv4 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
#[inline]
#[doc(alias = "sendto")]
pub fn sendto_v4<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV4,
) -> io::Result<usize> {
    backend::net::syscalls::sendto_v4(fd.as_fd(), buf, flags, addr)
}

/// `sendto(fd, buf, flags, addr, sizeof(struct sockaddr_in6))`—Writes data
/// to a socket to a specific IPv6 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
#[inline]
#[doc(alias = "sendto")]
pub fn sendto_v6<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV6,
) -> io::Result<usize> {
    backend::net::syscalls::sendto_v6(fd.as_fd(), buf, flags, addr)
}

/// `sendto(fd, buf, flags, addr, sizeof(struct sockaddr_un))`—Writes data to
/// a socket to a specific Unix-domain socket address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
#[cfg(unix)]
#[inline]
#[doc(alias = "sendto")]
pub fn sendto_unix<Fd: AsFd>(
    fd: Fd,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrUnix,
) -> io::Result<usize> {
    backend::net::syscalls::sendto_unix(fd.as_fd(), buf, flags, addr)
}

// TODO: `recvmsg`, `sendmsg`
