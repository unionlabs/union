//! Adapt the Winsock2 API to resemble a POSIX-style libc API.

#![allow(unused_imports)]
#![allow(non_camel_case_types)]

use windows_sys::Win32::Networking::WinSock;

pub(crate) use libc::{
    c_char, c_int, c_long, c_longlong, c_schar, c_short, c_uchar, c_uint, c_ulong, c_ulonglong,
    c_ushort, c_void, ssize_t,
};
pub(crate) type socklen_t = i32;

// windows-sys declares these constants as unsigned. For better compatibility
// with Unix-family APIs, redeclare them as signed. Filed upstream:
// <https://github.com/microsoft/windows-rs/issues/1718>
pub(crate) const AF_INET: i32 = WinSock::AF_INET as _;
pub(crate) const AF_INET6: i32 = WinSock::AF_INET6 as _;
pub(crate) const AF_UNSPEC: i32 = WinSock::AF_UNSPEC as _;
pub(crate) const SO_TYPE: i32 = WinSock::SO_TYPE as _;
pub(crate) const SO_REUSEADDR: i32 = WinSock::SO_REUSEADDR as _;
pub(crate) const SO_BROADCAST: i32 = WinSock::SO_BROADCAST as _;
pub(crate) const SO_LINGER: i32 = WinSock::SO_LINGER as _;
pub(crate) const SOL_SOCKET: i32 = WinSock::SOL_SOCKET as _;
pub(crate) const SO_RCVTIMEO: i32 = WinSock::SO_RCVTIMEO as _;
pub(crate) const SO_SNDTIMEO: i32 = WinSock::SO_SNDTIMEO as _;
pub(crate) const IP_TTL: i32 = WinSock::IP_TTL as _;
pub(crate) const TCP_NODELAY: i32 = WinSock::TCP_NODELAY as _;
pub(crate) const IP_ADD_MEMBERSHIP: i32 = WinSock::IP_ADD_MEMBERSHIP as _;
pub(crate) const IP_DROP_MEMBERSHIP: i32 = WinSock::IP_DROP_MEMBERSHIP as _;
pub(crate) const IP_MULTICAST_TTL: i32 = WinSock::IP_MULTICAST_TTL as _;
pub(crate) const IP_MULTICAST_LOOP: i32 = WinSock::IP_MULTICAST_LOOP as _;
pub(crate) const IPV6_ADD_MEMBERSHIP: i32 = WinSock::IPV6_ADD_MEMBERSHIP as _;
pub(crate) const IPV6_DROP_MEMBERSHIP: i32 = WinSock::IPV6_DROP_MEMBERSHIP as _;
pub(crate) const IPV6_MULTICAST_LOOP: i32 = WinSock::IPV6_MULTICAST_LOOP as _;
pub(crate) const IPV6_V6ONLY: i32 = WinSock::IPV6_V6ONLY as _;
pub(crate) const POLLERR: i16 = WinSock::POLLERR as _;
pub(crate) const POLLIN: i16 = WinSock::POLLIN as _;
pub(crate) const POLLNVAL: i16 = WinSock::POLLNVAL as _;
pub(crate) const POLLHUP: i16 = WinSock::POLLHUP as _;
pub(crate) const POLLPRI: i16 = WinSock::POLLPRI as _;
pub(crate) const POLLOUT: i16 = WinSock::POLLOUT as _;
pub(crate) const POLLRDNORM: i16 = WinSock::POLLRDNORM as _;
pub(crate) const POLLWRNORM: i16 = WinSock::POLLWRNORM as _;
pub(crate) const POLLRDBAND: i16 = WinSock::POLLRDBAND as _;
pub(crate) const POLLWRBAND: i16 = WinSock::POLLWRBAND as _;

// As above, cast the types for better compatibility, and also rename these to
// their Unix names.
pub(crate) const SHUT_RDWR: i32 = WinSock::SD_BOTH as _;
pub(crate) const SHUT_RD: i32 = WinSock::SD_RECEIVE as _;
pub(crate) const SHUT_WR: i32 = WinSock::SD_SEND as _;

// Include the contents of `WinSock`, renaming as needed to match POSIX.
//
// Use `WSA_E_CANCELLED` for `ECANCELED` instead of `WSAECANCELLED`, because
// `WSAECANCELLED` will be removed in the future.
// <https://docs.microsoft.com/en-us/windows/win32/api/ws2spi/nc-ws2spi-lpnsplookupserviceend#remarks>
pub(crate) use WinSock::{
    closesocket as close, ioctlsocket as ioctl, WSAPoll as poll, ADDRESS_FAMILY as sa_family_t,
    ADDRINFOA as addrinfo, IN6_ADDR as in6_addr, IN_ADDR as in_addr, IPV6_MREQ as ipv6_mreq,
    IP_MREQ as ip_mreq, SOCKADDR as sockaddr, SOCKADDR_IN as sockaddr_in,
    SOCKADDR_IN6 as sockaddr_in6, SOCKADDR_STORAGE as sockaddr_storage, WSAEACCES as EACCES,
    WSAEADDRINUSE as EADDRINUSE, WSAEADDRNOTAVAIL as EADDRNOTAVAIL,
    WSAEAFNOSUPPORT as EAFNOSUPPORT, WSAEALREADY as EALREADY, WSAEBADF as EBADF,
    WSAECONNABORTED as ECONNABORTED, WSAECONNREFUSED as ECONNREFUSED, WSAECONNRESET as ECONNRESET,
    WSAEDESTADDRREQ as EDESTADDRREQ, WSAEDISCON as EDISCON, WSAEDQUOT as EDQUOT,
    WSAEFAULT as EFAULT, WSAEHOSTDOWN as EHOSTDOWN, WSAEHOSTUNREACH as EHOSTUNREACH,
    WSAEINPROGRESS as EINPROGRESS, WSAEINTR as EINTR, WSAEINVAL as EINVAL,
    WSAEINVALIDPROCTABLE as EINVALIDPROCTABLE, WSAEINVALIDPROVIDER as EINVALIDPROVIDER,
    WSAEISCONN as EISCONN, WSAELOOP as ELOOP, WSAEMFILE as EMFILE, WSAEMSGSIZE as EMSGSIZE,
    WSAENAMETOOLONG as ENAMETOOLONG, WSAENETDOWN as ENETDOWN, WSAENETRESET as ENETRESET,
    WSAENETUNREACH as ENETUNREACH, WSAENOBUFS as ENOBUFS, WSAENOMORE as ENOMORE,
    WSAENOPROTOOPT as ENOPROTOOPT, WSAENOTCONN as ENOTCONN, WSAENOTEMPTY as ENOTEMPTY,
    WSAENOTSOCK as ENOTSOCK, WSAEOPNOTSUPP as EOPNOTSUPP, WSAEPFNOSUPPORT as EPFNOSUPPORT,
    WSAEPROCLIM as EPROCLIM, WSAEPROTONOSUPPORT as EPROTONOSUPPORT, WSAEPROTOTYPE as EPROTOTYPE,
    WSAEPROVIDERFAILEDINIT as EPROVIDERFAILEDINIT, WSAEREFUSED as EREFUSED, WSAEREMOTE as EREMOTE,
    WSAESHUTDOWN as ESHUTDOWN, WSAESOCKTNOSUPPORT as ESOCKTNOSUPPORT, WSAESTALE as ESTALE,
    WSAETIMEDOUT as ETIMEDOUT, WSAETOOMANYREFS as ETOOMANYREFS, WSAEUSERS as EUSERS,
    WSAEWOULDBLOCK as EWOULDBLOCK, WSAEWOULDBLOCK as EAGAIN, WSAPOLLFD as pollfd,
    WSA_E_CANCELLED as ECANCELED, *,
};
