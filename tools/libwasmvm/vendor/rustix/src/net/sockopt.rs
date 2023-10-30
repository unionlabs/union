//! `getsockopt` and `setsockopt` functions.
//!
//! In the rustix API, there is a separate function for each option, so that
//! it can be given an option-specific type signature.

#![doc(alias = "getsockopt")]
#![doc(alias = "setsockopt")]

use crate::net::{Ipv4Addr, Ipv6Addr, SocketType};
use crate::{backend, io};
use backend::fd::AsFd;
use core::time::Duration;

pub use backend::net::types::Timeout;

/// `getsockopt(fd, SOL_SOCKET, SO_TYPE)`—Returns the type of a socket.
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_TYPE")]
pub fn get_socket_type<Fd: AsFd>(fd: Fd) -> io::Result<SocketType> {
    backend::net::syscalls::sockopt::get_socket_type(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, value)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_REUSEADDR")]
pub fn set_socket_reuseaddr<Fd: AsFd>(fd: Fd, value: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_reuseaddr(fd.as_fd(), value)
}

/// `setsockopt(fd, SOL_SOCKET, SO_BROADCAST, broadcast)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_BROADCAST")]
pub fn set_socket_broadcast<Fd: AsFd>(fd: Fd, broadcast: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_broadcast(fd.as_fd(), broadcast)
}

/// `getsockopt(fd, SOL_SOCKET, SO_BROADCAST)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_BROADCAST")]
pub fn get_socket_broadcast<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_socket_broadcast(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_LINGER, linger)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_LINGER")]
pub fn set_socket_linger<Fd: AsFd>(fd: Fd, linger: Option<Duration>) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_linger(fd.as_fd(), linger)
}

/// `getsockopt(fd, SOL_SOCKET, SO_LINGER)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_LINGER")]
pub fn get_socket_linger<Fd: AsFd>(fd: Fd) -> io::Result<Option<Duration>> {
    backend::net::syscalls::sockopt::get_socket_linger(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_PASSCRED, passcred)`
///
/// # References
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "SO_PASSCRED")]
pub fn set_socket_passcred<Fd: AsFd>(fd: Fd, passcred: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_passcred(fd.as_fd(), passcred)
}

/// `getsockopt(fd, SOL_SOCKET, SO_PASSCRED)`
///
/// # References
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "SO_PASSCRED")]
pub fn get_socket_passcred<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_socket_passcred(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, id, timeout)`—Set the sending or receiving
/// timeout.
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_RCVTIMEO")]
#[doc(alias = "SO_SNDTIMEO")]
pub fn set_socket_timeout<Fd: AsFd>(
    fd: Fd,
    id: Timeout,
    timeout: Option<Duration>,
) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_timeout(fd.as_fd(), id, timeout)
}

/// `getsockopt(fd, SOL_SOCKET, id)`—Get the sending or receiving timeout.
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_RCVTIMEO")]
#[doc(alias = "SO_SNDTIMEO")]
pub fn get_socket_timeout<Fd: AsFd>(fd: Fd, id: Timeout) -> io::Result<Option<Duration>> {
    backend::net::syscalls::sockopt::get_socket_timeout(fd.as_fd(), id)
}

/// `getsockopt(fd, SOL_SOCKET, SO_ERROR)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_ERROR")]
pub fn get_socket_error<Fd: AsFd>(fd: Fd) -> io::Result<Result<(), io::Errno>> {
    backend::net::syscalls::sockopt::get_socket_error(fd.as_fd())
}

/// `getsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[cfg(any(apple, target_os = "freebsd"))]
#[doc(alias = "SO_NOSIGPIPE")]
#[inline]
pub fn getsockopt_nosigpipe<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::getsockopt_nosigpipe(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_NOSIGPIPE, val)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[cfg(any(apple, target_os = "freebsd"))]
#[doc(alias = "SO_NOSIGPIPE")]
#[inline]
pub fn setsockopt_nosigpipe<Fd: AsFd>(fd: Fd, val: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::setsockopt_nosigpipe(fd.as_fd(), val)
}

/// `setsockopt(fd, SOL_SOCKET, SO_KEEPALIVE, keepalive)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_KEEPALIVE")]
pub fn set_socket_keepalive<Fd: AsFd>(fd: Fd, keepalive: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_keepalive(fd.as_fd(), keepalive)
}

/// `getsockopt(fd, SOL_SOCKET, SO_KEEPALIVE)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_KEEPALIVE")]
pub fn get_socket_keepalive<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_socket_keepalive(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_RCVBUF, size)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_RCVBUF")]
pub fn set_socket_recv_buffer_size<Fd: AsFd>(fd: Fd, size: usize) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_recv_buffer_size(fd.as_fd(), size)
}

/// `getsockopt(fd, SOL_SOCKET, SO_RCVBUF)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_RCVBUF")]
pub fn get_socket_recv_buffer_size<Fd: AsFd>(fd: Fd) -> io::Result<usize> {
    backend::net::syscalls::sockopt::get_socket_recv_buffer_size(fd.as_fd())
}

/// `setsockopt(fd, SOL_SOCKET, SO_SNDBUF, size)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `setsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `setsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/setsockopt.2
/// [OpenBSD]: https://man.openbsd.org/setsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/setsockopt
/// [glibc `setsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_SNDBUF")]
pub fn set_socket_send_buffer_size<Fd: AsFd>(fd: Fd, size: usize) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_socket_send_buffer_size(fd.as_fd(), size)
}

/// `getsockopt(fd, SOL_SOCKET, SO_SNDBUF)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `sys/socket.h`]
///  - [Linux `getsockopt`]
///  - [Linux `socket`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `SOL_SOCKET` options]
///  - [Apple]
///  - [FreeBSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [DragonFly BSD]
///  - [illumos]
///  - [glibc `getsockopt`]
///  - [glibc `SOL_SOCKET` Options]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `sys/socket.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/sys_socket.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `socket`]: https://man7.org/linux/man-pages/man7/socket.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `SOL_SOCKET` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/sol-socket-socket-options
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [NetBSD]: https://man.netbsd.org/getsockopt.2
/// [OpenBSD]: https://man.openbsd.org/getsockopt.2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [illumos]: https://illumos.org/man/3SOCKET/getsockopt
/// [glibc `getsockopt`]: https://www.gnu.org/software/libc/manual/html_node/Socket-Option-Functions.html
/// [glibc `SOL_SOCKET` options]: https://www.gnu.org/software/libc/manual/html_node/Socket_002dLevel-Options.html
#[inline]
#[doc(alias = "SO_SNDBUF")]
pub fn get_socket_send_buffer_size<Fd: AsFd>(fd: Fd) -> io::Result<usize> {
    backend::net::syscalls::sockopt::get_socket_send_buffer_size(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IP, IP_TTL, ttl)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `setsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_TTL")]
pub fn set_ip_ttl<Fd: AsFd>(fd: Fd, ttl: u32) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ip_ttl(fd.as_fd(), ttl)
}

/// `getsockopt(fd, IPPROTO_IP, IP_TTL)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `getsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_TTL")]
pub fn get_ip_ttl<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::net::syscalls::sockopt::get_ip_ttl(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_V6ONLY, only_v6)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_V6ONLY")]
pub fn set_ipv6_v6only<Fd: AsFd>(fd: Fd, only_v6: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_v6only(fd.as_fd(), only_v6)
}

/// `getsockopt(fd, IPPROTO_IPV6, IPV6_V6ONLY)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `getsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `getsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `getsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `getsockopt`]: https://man.netbsd.org/getsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `getsockopt`]: https://man.openbsd.org/getsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `getsockopt`]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `getsockopt`]: https://illumos.org/man/3SOCKET/getsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_V6ONLY")]
pub fn get_ipv6_v6only<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_ipv6_v6only(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IP, IP_MULTICAST_LOOP, multicast_loop)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `setsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_MULTICAST_LOOP")]
pub fn set_ip_multicast_loop<Fd: AsFd>(fd: Fd, multicast_loop: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ip_multicast_loop(fd.as_fd(), multicast_loop)
}

/// `getsockopt(fd, IPPROTO_IP, IP_MULTICAST_LOOP)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `getsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
#[inline]
#[doc(alias = "IP_MULTICAST_LOOP")]
pub fn get_ip_multicast_loop<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_ip_multicast_loop(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IP, IP_MULTICAST_TTL, multicast_ttl)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `setsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_MULTICAST_TTL")]
pub fn set_ip_multicast_ttl<Fd: AsFd>(fd: Fd, multicast_ttl: u32) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ip_multicast_ttl(fd.as_fd(), multicast_ttl)
}

/// `getsockopt(fd, IPPROTO_IP, IP_MULTICAST_TTL)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `getsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
#[inline]
#[doc(alias = "IP_MULTICAST_TTL")]
pub fn get_ip_multicast_ttl<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::net::syscalls::sockopt::get_ip_multicast_ttl(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_MULTICAST_LOOP, multicast_loop)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_MULTICAST_LOOP")]
pub fn set_ipv6_multicast_loop<Fd: AsFd>(fd: Fd, multicast_loop: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_multicast_loop(fd.as_fd(), multicast_loop)
}

/// `getsockopt(fd, IPPROTO_IPV6, IPV6_MULTICAST_LOOP)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `getsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `getsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `getsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `getsockopt`]: https://man.netbsd.org/getsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `getsockopt`]: https://man.openbsd.org/getsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `getsockopt`]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `getsockopt`]: https://illumos.org/man/3SOCKET/getsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_MULTICAST_LOOP")]
pub fn get_ipv6_multicast_loop<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_ipv6_multicast_loop(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_MULTICAST_HOPS, multicast_hops)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IP_MULTICAST_TTL")]
pub fn set_ipv6_multicast_hops<Fd: AsFd>(fd: Fd, multicast_hops: u32) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_multicast_hops(fd.as_fd(), multicast_hops)
}

/// `getsockopt(fd, IPPROTO_IPV6, IPV6_UNICAST_HOPS)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `getsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `getsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `getsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `getsockopt`]: https://man.netbsd.org/getsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `getsockopt`]: https://man.openbsd.org/getsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `getsockopt`]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `getsockopt`]: https://illumos.org/man/3SOCKET/getsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_UNICAST_HOPS")]
pub fn get_ipv6_unicast_hops<Fd: AsFd>(fd: Fd) -> io::Result<u8> {
    backend::net::syscalls::sockopt::get_ipv6_unicast_hops(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_UNICAST_HOPS, unicast_hops)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_UNICAST_HOPS")]
pub fn set_ipv6_unicast_hops<Fd: AsFd>(fd: Fd, unicast_hops: Option<u8>) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_unicast_hops(fd.as_fd(), unicast_hops)
}

/// `getsockopt(fd, IPPROTO_IPV6, IPV6_MULTICAST_HOPS)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `getsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `getsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `getsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `getsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `getsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `getsockopt`]: https://man.netbsd.org/getsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `getsockopt`]: https://man.openbsd.org/getsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `getsockopt`]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `getsockopt`]: https://illumos.org/man/3SOCKET/getsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IP_MULTICAST_TTL")]
pub fn get_ipv6_multicast_hops<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::net::syscalls::sockopt::get_ipv6_multicast_hops(fd.as_fd())
}

/// `setsockopt(fd, IPPROTO_IP, IP_ADD_MEMBERSHIP, multiaddr, interface)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `setsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_ADD_MEMBERSHIP")]
pub fn set_ip_add_membership<Fd: AsFd>(
    fd: Fd,
    multiaddr: &Ipv4Addr,
    interface: &Ipv4Addr,
) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ip_add_membership(fd.as_fd(), multiaddr, interface)
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_ADD_MEMBERSHIP, multiaddr, interface)`
///
/// `IPV6_ADD_MEMBERSHIP` is the same as `IPV6_JOIN_GROUP` in POSIX.
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_JOIN_GROUP")]
#[doc(alias = "IPV6_ADD_MEMBERSHIP")]
pub fn set_ipv6_add_membership<Fd: AsFd>(
    fd: Fd,
    multiaddr: &Ipv6Addr,
    interface: u32,
) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_add_membership(fd.as_fd(), multiaddr, interface)
}

/// `setsockopt(fd, IPPROTO_IP, IP_DROP_MEMBERSHIP, multiaddr, interface)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ip`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IP` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip`]
///  - [illumos `setsockopt`]
///  - [illumos `ip`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ip`]: https://man7.org/linux/man-pages/man7/ip.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ip-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip`]: https://man.freebsd.org/cgi/man.cgi?query=ip&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip`]: https://man.netbsd.org/ip.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip`]: https://man.openbsd.org/ip.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip`]: https://man.dragonflybsd.org/?command=ip&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip`]: https://illumos.org/man/4P/ip
#[inline]
#[doc(alias = "IP_DROP_MEMBERSHIP")]
pub fn set_ip_drop_membership<Fd: AsFd>(
    fd: Fd,
    multiaddr: &Ipv4Addr,
    interface: &Ipv4Addr,
) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ip_drop_membership(fd.as_fd(), multiaddr, interface)
}

/// `setsockopt(fd, IPPROTO_IPV6, IPV6_DROP_MEMBERSHIP, multiaddr, interface)`
///
/// `IPV6_DROP_MEMBERSHIP` is the same as `IPV6_LEAVE_GROUP` in POSIX.
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/in.h`]
///  - [Linux `setsockopt`]
///  - [Linux `ipv6`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_IPV6` options]
///  - [Apple `setsockopt`]
///  - [Apple `ip6`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `ip6`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `ip6`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `ip6`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `ip6`]
///  - [illumos `setsockopt`]
///  - [illumos `ip6`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/in.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_in.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `ipv6`]: https://man7.org/linux/man-pages/man7/ipv6.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_IPV6` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-ipv6-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `ip6`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/ip6.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `ip6`]: https://man.freebsd.org/cgi/man.cgi?query=ip6&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `ip6`]: https://man.netbsd.org/ip6.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `ip6`]: https://man.openbsd.org/ip6.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `ip6`]: https://man.dragonflybsd.org/?command=ip6&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `ip6`]: https://illumos.org/man/4P/ip6
#[inline]
#[doc(alias = "IPV6_LEAVE_GROUP")]
#[doc(alias = "IPV6_DROP_MEMBERSHIP")]
pub fn set_ipv6_drop_membership<Fd: AsFd>(
    fd: Fd,
    multiaddr: &Ipv6Addr,
    interface: u32,
) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_ipv6_drop_membership(fd.as_fd(), multiaddr, interface)
}

/// `setsockopt(fd, IPPROTO_TCP, TCP_NODELAY, nodelay)`
///
/// # References
///  - [POSIX `setsockopt`]
///  - [POSIX `netinet/tcp.h`]
///  - [Linux `setsockopt`]
///  - [Linux `tcp`]
///  - [Winsock2 `setsockopt`]
///  - [Winsock2 `IPPROTO_TCP` options]
///  - [Apple `setsockopt`]
///  - [Apple `tcp`]
///  - [FreeBSD `setsockopt`]
///  - [FreeBSD `tcp`]
///  - [NetBSD `setsockopt`]
///  - [NetBSD `tcp`]
///  - [OpenBSD `setsockopt`]
///  - [OpenBSD `tcp`]
///  - [DragonFly BSD `setsockopt`]
///  - [DragonFly BSD `tcp`]
///  - [illumos `setsockopt`]
///  - [illumos `tcp`]
///
/// [POSIX `setsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html
/// [POSIX `netinet/tcp.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_tcp.h.html
/// [Linux `setsockopt`]: https://man7.org/linux/man-pages/man2/setsockopt.2.html
/// [Linux `tcp`]: https://man7.org/linux/man-pages/man7/tcp.7.html
/// [Winsock2 `setsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-setsockopt
/// [Winsock2 `IPPROTO_TCP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-tcp-socket-options
/// [Apple `setsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsockopt.2.html
/// [Apple `tcp`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/tcp.4.auto.html
/// [FreeBSD `setsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=setsockopt&sektion=2
/// [FreeBSD `tcp`]: https://man.freebsd.org/cgi/man.cgi?query=tcp&sektion=4
/// [NetBSD `setsockopt`]: https://man.netbsd.org/setsockopt.2
/// [NetBSD `tcp`]: https://man.netbsd.org/tcp.4
/// [OpenBSD `setsockopt`]: https://man.openbsd.org/setsockopt.2
/// [OpenBSD `tcp`]: https://man.openbsd.org/tcp.4
/// [DragonFly BSD `setsockopt`]: https://man.dragonflybsd.org/?command=setsockopt&section=2
/// [DragonFly BSD `tcp`]: https://man.dragonflybsd.org/?command=tcp&section=4
/// [illumos `setsockopt`]: https://illumos.org/man/3SOCKET/setsockopt
/// [illumos `tcp`]: https://illumos.org/man/4P/tcp
#[inline]
#[doc(alias = "TCP_NODELAY")]
pub fn set_tcp_nodelay<Fd: AsFd>(fd: Fd, nodelay: bool) -> io::Result<()> {
    backend::net::syscalls::sockopt::set_tcp_nodelay(fd.as_fd(), nodelay)
}

/// `getsockopt(fd, IPPROTO_TCP, TCP_NODELAY)`
///
/// # References
///  - [POSIX `getsockopt`]
///  - [POSIX `netinet/tcp.h`]
///  - [Linux `getsockopt`]
///  - [Linux `tcp`]
///  - [Winsock2 `getsockopt`]
///  - [Winsock2 `IPPROTO_TCP` options]
///  - [Apple `getsockopt`]
///  - [Apple `tcp`]
///  - [FreeBSD `getsockopt`]
///  - [FreeBSD `tcp`]
///  - [NetBSD `getsockopt`]
///  - [NetBSD `tcp`]
///  - [OpenBSD `getsockopt`]
///  - [OpenBSD `tcp`]
///  - [DragonFly BSD `getsockopt`]
///  - [DragonFly BSD `tcp`]
///  - [illumos `getsockopt`]
///  - [illumos `tcp`]
///
/// [POSIX `getsockopt`]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html
/// [POSIX `netinet/tcp.h`]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/netinet_tcp.h.html
/// [Linux `getsockopt`]: https://man7.org/linux/man-pages/man2/getsockopt.2.html
/// [Linux `tcp`]: https://man7.org/linux/man-pages/man7/tcp.7.html
/// [Winsock2 `getsockopt`]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-getsockopt
/// [Winsock2 `IPPROTO_TCP` options]: https://docs.microsoft.com/en-us/windows/win32/winsock/ipproto-tcp-socket-options
/// [Apple `getsockopt`]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/getsockopt.2.html
/// [Apple `tcp`]: https://opensource.apple.com/source/xnu/xnu-7195.81.3/bsd/man/man4/tcp.4.auto.html
/// [FreeBSD `getsockopt`]: https://man.freebsd.org/cgi/man.cgi?query=getsockopt&sektion=2
/// [FreeBSD `tcp`]: https://man.freebsd.org/cgi/man.cgi?query=tcp&sektion=4
/// [NetBSD `getsockopt`]: https://man.netbsd.org/getsockopt.2
/// [NetBSD `tcp`]: https://man.netbsd.org/tcp.4
/// [OpenBSD `getsockopt`]: https://man.openbsd.org/getsockopt.2
/// [OpenBSD `tcp`]: https://man.openbsd.org/tcp.4
/// [DragonFly BSD `getsockopt`]: https://man.dragonflybsd.org/?command=getsockopt&section=2
/// [DragonFly BSD `tcp`]: https://man.dragonflybsd.org/?command=tcp&section=4
/// [illumos `getsockopt`]: https://illumos.org/man/3SOCKET/getsockopt
/// [illumos `tcp`]: https://illumos.org/man/4P/tcp
#[inline]
#[doc(alias = "TCP_NODELAY")]
pub fn get_tcp_nodelay<Fd: AsFd>(fd: Fd) -> io::Result<bool> {
    backend::net::syscalls::sockopt::get_tcp_nodelay(fd.as_fd())
}
