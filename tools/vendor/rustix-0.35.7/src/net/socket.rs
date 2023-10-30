use crate::backend;
use crate::io::{self, OwnedFd};
use crate::net::{SocketAddr, SocketAddrAny, SocketAddrV4, SocketAddrV6};
use backend::fd::{AsFd, BorrowedFd};

#[cfg(unix)]
pub use backend::net::addr::SocketAddrUnix;
pub use backend::net::types::{
    AcceptFlags, AddressFamily, Protocol, Shutdown, SocketFlags, SocketType,
};

impl Default for Protocol {
    #[inline]
    fn default() -> Self {
        Self::IP
    }
}

/// `socket(domain, type_, protocol)`—Creates a socket.
///
/// POSIX guarantees that `socket` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors
/// may be unexpectedly allocated on other threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/socket.html
/// [Linux]: https://man7.org/linux/man-pages/man2/socket.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/socket.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-socket
#[inline]
pub fn socket(domain: AddressFamily, type_: SocketType, protocol: Protocol) -> io::Result<OwnedFd> {
    backend::net::syscalls::socket(domain, type_, protocol)
}

/// `socket_with(domain, type_ | flags, protocol)`—Creates a socket, with
/// flags.
///
/// POSIX guarantees that `socket` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors
/// may be unexpectedly allocated on other threads or in libraries.
///
/// `socket_with` is the same as [`socket`] but adds an additional flags
/// operand.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/socket.html
/// [Linux]: https://man7.org/linux/man-pages/man2/socket.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/socket.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-socket
#[inline]
pub fn socket_with(
    domain: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<OwnedFd> {
    backend::net::syscalls::socket_with(domain, type_, flags, protocol)
}

/// `bind(sockfd, addr)`—Binds a socket to an IP address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html
/// [Linux]: https://man7.org/linux/man-pages/man2/bind.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
pub fn bind<Fd: AsFd>(sockfd: Fd, addr: &SocketAddr) -> io::Result<()> {
    _bind(sockfd.as_fd(), addr)
}

fn _bind(sockfd: BorrowedFd<'_>, addr: &SocketAddr) -> io::Result<()> {
    match addr {
        SocketAddr::V4(v4) => backend::net::syscalls::bind_v4(sockfd, v4),
        SocketAddr::V6(v6) => backend::net::syscalls::bind_v6(sockfd, v6),
    }
}

/// `bind(sockfd, addr)`—Binds a socket to an address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html
/// [Linux]: https://man7.org/linux/man-pages/man2/bind.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
#[doc(alias = "bind")]
pub fn bind_any<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrAny) -> io::Result<()> {
    _bind_any(sockfd.as_fd(), addr)
}

fn _bind_any(sockfd: BorrowedFd<'_>, addr: &SocketAddrAny) -> io::Result<()> {
    match addr {
        SocketAddrAny::V4(v4) => backend::net::syscalls::bind_v4(sockfd, v4),
        SocketAddrAny::V6(v6) => backend::net::syscalls::bind_v6(sockfd, v6),
        #[cfg(unix)]
        SocketAddrAny::Unix(unix) => backend::net::syscalls::bind_unix(sockfd, unix),
    }
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_in))`—Binds a socket to an
/// IPv4 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html
/// [Linux]: https://man7.org/linux/man-pages/man2/bind.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
#[inline]
#[doc(alias = "bind")]
pub fn bind_v4<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrV4) -> io::Result<()> {
    backend::net::syscalls::bind_v4(sockfd.as_fd(), addr)
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_in6))`—Binds a socket to an
/// IPv6 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html
/// [Linux]: https://man7.org/linux/man-pages/man2/bind.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
#[inline]
#[doc(alias = "bind")]
pub fn bind_v6<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrV6) -> io::Result<()> {
    backend::net::syscalls::bind_v6(sockfd.as_fd(), addr)
}

/// `bind(sockfd, addr, sizeof(struct sockaddr_un))`—Binds a socket to a
/// Unix-domain address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html
/// [Linux]: https://man7.org/linux/man-pages/man2/bind.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/bind.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-bind
#[cfg(unix)]
#[inline]
#[doc(alias = "bind")]
pub fn bind_unix<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrUnix) -> io::Result<()> {
    backend::net::syscalls::bind_unix(sockfd.as_fd(), addr)
}

/// `connect(sockfd, addr)`—Initiates a connection to an IP address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html
/// [Linux]: https://man7.org/linux/man-pages/man2/connect.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/connect.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
pub fn connect<Fd: AsFd>(sockfd: Fd, addr: &SocketAddr) -> io::Result<()> {
    _connect(sockfd.as_fd(), addr)
}

fn _connect(sockfd: BorrowedFd<'_>, addr: &SocketAddr) -> io::Result<()> {
    match addr {
        SocketAddr::V4(v4) => backend::net::syscalls::connect_v4(sockfd, v4),
        SocketAddr::V6(v6) => backend::net::syscalls::connect_v6(sockfd, v6),
    }
}

/// `connect(sockfd, addr)`—Initiates a connection.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html
/// [Linux]: https://man7.org/linux/man-pages/man2/connect.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/connect.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
#[doc(alias = "connect")]
pub fn connect_any<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrAny) -> io::Result<()> {
    _connect_any(sockfd.as_fd(), addr)
}

fn _connect_any(sockfd: BorrowedFd<'_>, addr: &SocketAddrAny) -> io::Result<()> {
    match addr {
        SocketAddrAny::V4(v4) => backend::net::syscalls::connect_v4(sockfd, v4),
        SocketAddrAny::V6(v6) => backend::net::syscalls::connect_v6(sockfd, v6),
        #[cfg(unix)]
        SocketAddrAny::Unix(unix) => backend::net::syscalls::connect_unix(sockfd, unix),
    }
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_in))`—Initiates a
/// connection to an IPv4 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html
/// [Linux]: https://man7.org/linux/man-pages/man2/connect.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/connect.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
#[inline]
#[doc(alias = "connect")]
pub fn connect_v4<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrV4) -> io::Result<()> {
    backend::net::syscalls::connect_v4(sockfd.as_fd(), addr)
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_in6))`—Initiates a
/// connection to an IPv6 address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html
/// [Linux]: https://man7.org/linux/man-pages/man2/connect.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/connect.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
#[inline]
#[doc(alias = "connect")]
pub fn connect_v6<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrV6) -> io::Result<()> {
    backend::net::syscalls::connect_v6(sockfd.as_fd(), addr)
}

/// `connect(sockfd, addr, sizeof(struct sockaddr_un))`—Initiates a
/// connection to a Unix-domain address.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html
/// [Linux]: https://man7.org/linux/man-pages/man2/connect.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/connect.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect
#[cfg(unix)]
#[inline]
#[doc(alias = "connect")]
pub fn connect_unix<Fd: AsFd>(sockfd: Fd, addr: &SocketAddrUnix) -> io::Result<()> {
    backend::net::syscalls::connect_unix(sockfd.as_fd(), addr)
}

/// `listen(fd, backlog)`—Enables listening for incoming connections.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/listen.html
/// [Linux]: https://man7.org/linux/man-pages/man2/listen.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/listen.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-listen
#[inline]
pub fn listen<Fd: AsFd>(sockfd: Fd, backlog: i32) -> io::Result<()> {
    backend::net::syscalls::listen(sockfd.as_fd(), backlog)
}

/// `accept(fd, NULL, NULL)`—Accepts an incoming connection.
///
/// Use [`acceptfrom`] to retrieve the peer address.
///
/// POSIX guarantees that `accept` will use the lowest unused file descriptor,
/// however it is not safe in general to rely on this, as file descriptors may
/// be unexpectedly allocated on other threads or in libraries.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/accept.html
/// [Linux]: https://man7.org/linux/man-pages/man2/accept.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/accept.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept
#[inline]
#[doc(alias = "accept4")]
pub fn accept<Fd: AsFd>(sockfd: Fd) -> io::Result<OwnedFd> {
    backend::net::syscalls::accept(sockfd.as_fd())
}

/// `accept4(fd, NULL, NULL, flags)`—Accepts an incoming connection, with
/// flags.
///
/// Use [`acceptfrom_with`] to retrieve the peer address.
///
/// Even though POSIX guarantees that this will use the lowest unused file
/// descriptor, it is not safe in general to rely on this, as file descriptors
/// may be unexpectedly allocated on other threads or in libraries.
///
/// `accept_with` is the same as [`accept`] but adds an additional flags
/// operand.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/accept4.2.html
#[inline]
#[doc(alias = "accept4")]
pub fn accept_with<Fd: AsFd>(sockfd: Fd, flags: AcceptFlags) -> io::Result<OwnedFd> {
    backend::net::syscalls::accept_with(sockfd.as_fd(), flags)
}

/// `accept(fd, &addr, &len)`—Accepts an incoming connection and returns the
/// peer address.
///
/// Use [`accept`] if the peer address isn't needed.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/accept.html
/// [Linux]: https://man7.org/linux/man-pages/man2/accept.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/accept.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-accept
#[inline]
#[doc(alias = "accept4")]
pub fn acceptfrom<Fd: AsFd>(sockfd: Fd) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    backend::net::syscalls::acceptfrom(sockfd.as_fd())
}

/// `accept4(fd, &addr, &len, flags)`—Accepts an incoming connection and
/// returns the peer address, with flags.
///
/// Use [`accept_with`] if the peer address isn't needed.
///
/// `acceptfrom_with` is the same as [`acceptfrom`] but adds an additional
/// flags operand.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/accept4.2.html
#[inline]
#[doc(alias = "accept4")]
pub fn acceptfrom_with<Fd: AsFd>(
    sockfd: Fd,
    flags: AcceptFlags,
) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    backend::net::syscalls::acceptfrom_with(sockfd.as_fd(), flags)
}

/// `shutdown(fd, how)`—Closes the read and/or write sides of a stream.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/shutdown.html
/// [Linux]: https://man7.org/linux/man-pages/man2/shutdown.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/shutdown.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-shutdown
#[inline]
pub fn shutdown<Fd: AsFd>(sockfd: Fd, how: Shutdown) -> io::Result<()> {
    backend::net::syscalls::shutdown(sockfd.as_fd(), how)
}

/// `getsockname(fd, addr, len)`—Returns the address a socket is bound to.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockname.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getsockname.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockname.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockname
#[inline]
pub fn getsockname<Fd: AsFd>(sockfd: Fd) -> io::Result<SocketAddrAny> {
    backend::net::syscalls::getsockname(sockfd.as_fd())
}

/// `getpeername(fd, addr, len)`—Returns the address a socket is connected
/// to.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpeername.html
/// [Linux]: https://man7.org/linux/man-pages/man2/getpeername.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getpeername.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getpeername
#[inline]
pub fn getpeername<Fd: AsFd>(sockfd: Fd) -> io::Result<Option<SocketAddrAny>> {
    backend::net::syscalls::getpeername(sockfd.as_fd())
}
