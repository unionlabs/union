//! `recv` and `send`, and variants.

#[cfg(unix)]
use crate::net::SocketAddrUnix;
use crate::net::{SocketAddr, SocketAddrAny, SocketAddrV4, SocketAddrV6};
use crate::{backend, io};
use backend::fd::{AsFd, BorrowedFd};

pub use backend::net::send_recv::{RecvFlags, SendFlags};

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
mod msg;

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub use msg::*;

/// `recv(fd, buf, flags)`—Reads data from a socket.
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendrecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recv.html
/// [Linux]: https://man7.org/linux/man-pages/man2/recv.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/recv.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-recv
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=recv&sektion=2
/// [NetBSD]: https://man.netbsd.org/recv.2
/// [OpenBSD]: https://man.openbsd.org/recv.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=recv&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/recv
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Receiving-Data.html
#[inline]
pub fn recv<Fd: AsFd>(fd: Fd, buf: &mut [u8], flags: RecvFlags) -> io::Result<usize> {
    backend::net::syscalls::recv(fd.as_fd(), buf, flags)
}

/// `send(fd, buf, flags)`—Writes data to a socket.
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendrecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/send.html
/// [Linux]: https://man7.org/linux/man-pages/man2/send.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/send.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-send
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=send&sektion=2
/// [NetBSD]: https://man.netbsd.org/send.2
/// [OpenBSD]: https://man.openbsd.org/send.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=send&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/send
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Data.html
#[inline]
pub fn send<Fd: AsFd>(fd: Fd, buf: &[u8], flags: SendFlags) -> io::Result<usize> {
    backend::net::syscalls::send(fd.as_fd(), buf, flags)
}

/// `recvfrom(fd, buf, flags, addr, len)`—Reads data from a socket and
/// returns the sender address.
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvfrom.html
/// [Linux]: https://man7.org/linux/man-pages/man2/recvfrom.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/recvfrom.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-recvfrom
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=recvfrom&sektion=2
/// [NetBSD]: https://man.netbsd.org/recvfrom.2
/// [OpenBSD]: https://man.openbsd.org/recvfrom.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=recvfrom&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/recvfrom
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Receiving-Datagrams.html
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendto&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendto.2
/// [OpenBSD]: https://man.openbsd.org/sendto.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendto&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendto
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Datagrams.html
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendto&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendto.2
/// [OpenBSD]: https://man.openbsd.org/sendto.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendto&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendto
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Datagrams.html
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendto&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendto.2
/// [OpenBSD]: https://man.openbsd.org/sendto.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendto&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendto
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Datagrams.html
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendto&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendto.2
/// [OpenBSD]: https://man.openbsd.org/sendto.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendto&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendto
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Datagrams.html
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
///  - [glibc]
///
/// [Beej's Guide to Network Programming]: https://beej.us/guide/bgnet/html/split/system-calls-or-bust.html#sendtorecv
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html
/// [Linux]: https://man7.org/linux/man-pages/man2/sendto.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/sendto.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-sendto
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=sendto&sektion=2
/// [NetBSD]: https://man.netbsd.org/sendto.2
/// [OpenBSD]: https://man.openbsd.org/sendto.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=sendto&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/sendto
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Sending-Datagrams.html
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
