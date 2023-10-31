// Copyright 2015 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::cmp::min;
#[cfg(not(target_os = "redox"))]
use std::io::IoSlice;
use std::marker::PhantomData;
use std::mem::{self, size_of, MaybeUninit};
use std::net::Shutdown;
use std::net::{Ipv4Addr, Ipv6Addr};
#[cfg(all(feature = "all", target_vendor = "apple"))]
use std::num::NonZeroU32;
#[cfg(all(
    feature = "all",
    any(
        target_os = "android",
        target_os = "freebsd",
        target_os = "linux",
        target_vendor = "apple",
    )
))]
use std::num::NonZeroUsize;
#[cfg(feature = "all")]
use std::os::unix::ffi::OsStrExt;
#[cfg(all(
    feature = "all",
    any(
        target_os = "android",
        target_os = "freebsd",
        target_os = "linux",
        target_vendor = "apple",
    )
))]
use std::os::unix::io::RawFd;
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(feature = "all")]
use std::os::unix::net::{UnixDatagram, UnixListener, UnixStream};
#[cfg(feature = "all")]
use std::path::Path;
#[cfg(not(all(target_os = "redox", not(feature = "all"))))]
use std::ptr;
use std::time::{Duration, Instant};
use std::{io, slice};

#[cfg(not(target_vendor = "apple"))]
use libc::ssize_t;
use libc::{c_void, in6_addr, in_addr};

#[cfg(not(target_os = "redox"))]
use crate::RecvFlags;
use crate::{Domain, Protocol, SockAddr, TcpKeepalive, Type};

pub(crate) use libc::c_int;

// Used in `Domain`.
pub(crate) use libc::{AF_INET, AF_INET6};
// Used in `Type`.
#[cfg(all(feature = "all", not(target_os = "redox")))]
pub(crate) use libc::SOCK_RAW;
#[cfg(feature = "all")]
pub(crate) use libc::SOCK_SEQPACKET;
pub(crate) use libc::{SOCK_DGRAM, SOCK_STREAM};
// Used in `Protocol`.
pub(crate) use libc::{IPPROTO_ICMP, IPPROTO_ICMPV6, IPPROTO_TCP, IPPROTO_UDP};
// Used in `SockAddr`.
pub(crate) use libc::{
    sa_family_t, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_storage, socklen_t,
};
// Used in `RecvFlags`.
#[cfg(not(target_os = "redox"))]
pub(crate) use libc::{MSG_TRUNC, SO_OOBINLINE};
// Used in `Socket`.
#[cfg(all(feature = "all", not(target_os = "redox")))]
pub(crate) use libc::IP_HDRINCL;
#[cfg(not(any(
    target_os = "fuschia",
    target_os = "redox",
    target_os = "solaris",
    target_os = "illumos",
)))]
pub(crate) use libc::IP_TOS;
#[cfg(not(target_vendor = "apple"))]
pub(crate) use libc::SO_LINGER;
#[cfg(target_vendor = "apple")]
pub(crate) use libc::SO_LINGER_SEC as SO_LINGER;
pub(crate) use libc::{
    ip_mreq as IpMreq, ipv6_mreq as Ipv6Mreq, linger, IPPROTO_IP, IPPROTO_IPV6,
    IPV6_MULTICAST_HOPS, IPV6_MULTICAST_IF, IPV6_MULTICAST_LOOP, IPV6_UNICAST_HOPS, IPV6_V6ONLY,
    IP_ADD_MEMBERSHIP, IP_DROP_MEMBERSHIP, IP_MULTICAST_IF, IP_MULTICAST_LOOP, IP_MULTICAST_TTL,
    IP_TTL, MSG_OOB, MSG_PEEK, SOL_SOCKET, SO_BROADCAST, SO_ERROR, SO_KEEPALIVE, SO_RCVBUF,
    SO_RCVTIMEO, SO_REUSEADDR, SO_SNDBUF, SO_SNDTIMEO, SO_TYPE, TCP_NODELAY,
};
#[cfg(not(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_vendor = "apple"
)))]
pub(crate) use libc::{IPV6_ADD_MEMBERSHIP, IPV6_DROP_MEMBERSHIP};
#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_vendor = "apple",
))]
pub(crate) use libc::{
    IPV6_JOIN_GROUP as IPV6_ADD_MEMBERSHIP, IPV6_LEAVE_GROUP as IPV6_DROP_MEMBERSHIP,
};
#[cfg(all(
    feature = "all",
    any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_vendor = "apple",
    )
))]
pub(crate) use libc::{TCP_KEEPCNT, TCP_KEEPINTVL};

// See this type in the Windows file.
pub(crate) type Bool = c_int;

#[cfg(target_vendor = "apple")]
use libc::TCP_KEEPALIVE as KEEPALIVE_TIME;
#[cfg(not(any(target_vendor = "apple", target_os = "haiku", target_os = "openbsd")))]
use libc::TCP_KEEPIDLE as KEEPALIVE_TIME;

/// Helper macro to execute a system call that returns an `io::Result`.
macro_rules! syscall {
    ($fn: ident ( $($arg: expr),* $(,)* ) ) => {{
        #[allow(unused_unsafe)]
        let res = unsafe { libc::$fn($($arg, )*) };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

/// Maximum size of a buffer passed to system call like `recv` and `send`.
#[cfg(not(target_vendor = "apple"))]
const MAX_BUF_LEN: usize = <ssize_t>::max_value() as usize;

// The maximum read limit on most posix-like systems is `SSIZE_MAX`, with the
// man page quoting that if the count of bytes to read is greater than
// `SSIZE_MAX` the result is "unspecified".
//
// On macOS, however, apparently the 64-bit libc is either buggy or
// intentionally showing odd behavior by rejecting any read with a size larger
// than or equal to INT_MAX. To handle both of these the read size is capped on
// both platforms.
#[cfg(target_vendor = "apple")]
const MAX_BUF_LEN: usize = <c_int>::max_value() as usize - 1;

#[cfg(any(
    all(
        target_os = "linux",
        any(
            target_env = "gnu",
            all(target_env = "uclibc", target_pointer_width = "64")
        )
    ),
    target_os = "android",
))]
type IovLen = usize;

#[cfg(any(
    all(
        target_os = "linux",
        any(
            target_env = "musl",
            all(target_env = "uclibc", target_pointer_width = "32")
        )
    ),
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "fuchsia",
    target_os = "haiku",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "openbsd",
    target_os = "solaris",
    target_vendor = "apple",
))]
type IovLen = c_int;

/// Unix only API.
impl Domain {
    /// Domain for Unix socket communication, corresponding to `AF_UNIX`.
    #[cfg_attr(docsrs, doc(cfg(unix)))]
    pub const UNIX: Domain = Domain(libc::AF_UNIX);

    /// Domain for low-level packet interface, corresponding to `AF_PACKET`.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub const PACKET: Domain = Domain(libc::AF_PACKET);

    /// Domain for low-level VSOCK interface, corresponding to `AF_VSOCK`.
    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "all", any(target_os = "android", target_os = "linux"))))
    )]
    pub const VSOCK: Domain = Domain(libc::AF_VSOCK);
}

impl_debug!(
    Domain,
    libc::AF_INET,
    libc::AF_INET6,
    libc::AF_UNIX,
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    #[cfg_attr(
        docsrs,
        doc(cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux")))
    )]
    libc::AF_PACKET,
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(any(target_os = "android", target_os = "linux"))))]
    libc::AF_VSOCK,
    libc::AF_UNSPEC, // = 0.
);

/// Unix only API.
impl Type {
    /// Set `SOCK_NONBLOCK` on the `Type`.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        )))
    )]
    pub const fn nonblocking(self) -> Type {
        Type(self.0 | libc::SOCK_NONBLOCK)
    }

    /// Set `SOCK_CLOEXEC` on the `Type`.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        )))
    )]
    pub const fn cloexec(self) -> Type {
        self._cloexec()
    }

    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub(crate) const fn _cloexec(self) -> Type {
        Type(self.0 | libc::SOCK_CLOEXEC)
    }
}

impl_debug!(
    Type,
    libc::SOCK_STREAM,
    libc::SOCK_DGRAM,
    #[cfg(not(target_os = "redox"))]
    libc::SOCK_RAW,
    #[cfg(not(any(target_os = "redox", target_os = "haiku")))]
    libc::SOCK_RDM,
    libc::SOCK_SEQPACKET,
    /* TODO: add these optional bit OR-ed flags:
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    libc::SOCK_NONBLOCK,
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    libc::SOCK_CLOEXEC,
    */
);

impl_debug!(
    Protocol,
    libc::IPPROTO_ICMP,
    libc::IPPROTO_ICMPV6,
    libc::IPPROTO_TCP,
    libc::IPPROTO_UDP,
);

/// Unix-only API.
#[cfg(not(target_os = "redox"))]
impl RecvFlags {
    /// Check if the message terminates a record.
    ///
    /// Not all socket types support the notion of records.
    /// For socket types that do support it (such as [`SEQPACKET`][Type::SEQPACKET]),
    /// a record is terminated by sending a message with the end-of-record flag set.
    ///
    /// On Unix this corresponds to the MSG_EOR flag.
    pub const fn is_end_of_record(self) -> bool {
        self.0 & libc::MSG_EOR != 0
    }

    /// Check if the message contains out-of-band data.
    ///
    /// This is useful for protocols where you receive out-of-band data
    /// mixed in with the normal data stream.
    ///
    /// On Unix this corresponds to the MSG_OOB flag.
    pub const fn is_out_of_band(self) -> bool {
        self.0 & libc::MSG_OOB != 0
    }
}

#[cfg(not(target_os = "redox"))]
impl std::fmt::Debug for RecvFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RecvFlags")
            .field("is_end_of_record", &self.is_end_of_record())
            .field("is_out_of_band", &self.is_out_of_band())
            .field("is_truncated", &self.is_truncated())
            .finish()
    }
}

#[repr(transparent)]
pub struct MaybeUninitSlice<'a> {
    vec: libc::iovec,
    _lifetime: PhantomData<&'a mut [MaybeUninit<u8>]>,
}

unsafe impl<'a> Send for MaybeUninitSlice<'a> {}

unsafe impl<'a> Sync for MaybeUninitSlice<'a> {}

impl<'a> MaybeUninitSlice<'a> {
    pub(crate) fn new(buf: &'a mut [MaybeUninit<u8>]) -> MaybeUninitSlice<'a> {
        MaybeUninitSlice {
            vec: libc::iovec {
                iov_base: buf.as_mut_ptr().cast(),
                iov_len: buf.len(),
            },
            _lifetime: PhantomData,
        }
    }

    pub(crate) fn as_slice(&self) -> &[MaybeUninit<u8>] {
        unsafe { slice::from_raw_parts(self.vec.iov_base.cast(), self.vec.iov_len) }
    }

    pub(crate) fn as_mut_slice(&mut self) -> &mut [MaybeUninit<u8>] {
        unsafe { slice::from_raw_parts_mut(self.vec.iov_base.cast(), self.vec.iov_len) }
    }
}

/// Unix only API.
impl SockAddr {
    /// Constructs a `SockAddr` with the family `AF_UNIX` and the provided path.
    ///
    /// # Failure
    ///
    /// Returns an error if the path is longer than `SUN_LEN`.
    #[cfg(feature = "all")]
    #[cfg_attr(docsrs, doc(cfg(all(unix, feature = "all"))))]
    #[allow(unused_unsafe)] // TODO: replace with `unsafe_op_in_unsafe_fn` once stable.
    pub fn unix<P>(path: P) -> io::Result<SockAddr>
    where
        P: AsRef<Path>,
    {
        unsafe {
            SockAddr::init(|storage, len| {
                // Safety: `SockAddr::init` zeros the address, which is a valid
                // representation.
                let storage: &mut libc::sockaddr_un = unsafe { &mut *storage.cast() };
                let len: &mut socklen_t = unsafe { &mut *len };

                let bytes = path.as_ref().as_os_str().as_bytes();
                let too_long = match bytes.first() {
                    None => false,
                    // linux abstract namespaces aren't null-terminated
                    Some(&0) => bytes.len() > storage.sun_path.len(),
                    Some(_) => bytes.len() >= storage.sun_path.len(),
                };
                if too_long {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "path must be shorter than SUN_LEN",
                    ));
                }

                storage.sun_family = libc::AF_UNIX as sa_family_t;
                // Safety: `bytes` and `addr.sun_path` are not overlapping and
                // both point to valid memory.
                // `SockAddr::init` zeroes the memory, so the path is already
                // null terminated.
                unsafe {
                    ptr::copy_nonoverlapping(
                        bytes.as_ptr(),
                        storage.sun_path.as_mut_ptr() as *mut u8,
                        bytes.len(),
                    )
                };

                let base = storage as *const _ as usize;
                let path = &storage.sun_path as *const _ as usize;
                let sun_path_offset = path - base;
                let length = sun_path_offset
                    + bytes.len()
                    + match bytes.first() {
                        Some(&0) | None => 0,
                        Some(_) => 1,
                    };
                *len = length as socklen_t;

                Ok(())
            })
        }
        .map(|(_, addr)| addr)
    }
}

impl SockAddr {
    /// Constructs a `SockAddr` with the family `AF_VSOCK` and the provided CID/port.
    ///
    /// # Errors
    ///
    /// This function can never fail. In a future version of this library it will be made
    /// infallible.
    #[allow(unused_unsafe)] // TODO: replace with `unsafe_op_in_unsafe_fn` once stable.
    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "all", any(target_os = "android", target_os = "linux"))))
    )]
    pub fn vsock(cid: u32, port: u32) -> io::Result<SockAddr> {
        unsafe {
            SockAddr::init(|storage, len| {
                // Safety: `SockAddr::init` zeros the address, which is a valid
                // representation.
                let storage: &mut libc::sockaddr_vm = unsafe { &mut *storage.cast() };
                let len: &mut socklen_t = unsafe { &mut *len };

                storage.svm_family = libc::AF_VSOCK as sa_family_t;
                storage.svm_cid = cid;
                storage.svm_port = port;

                *len = mem::size_of::<libc::sockaddr_vm>() as socklen_t;

                Ok(())
            })
        }
        .map(|(_, addr)| addr)
    }

    /// Returns this address VSOCK CID/port if it is in the `AF_VSOCK` family,
    /// otherwise return `None`.
    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "all", any(target_os = "android", target_os = "linux"))))
    )]
    pub fn vsock_address(&self) -> Option<(u32, u32)> {
        if self.family() == libc::AF_VSOCK as sa_family_t {
            // Safety: if the ss_family field is AF_VSOCK then storage must be a sockaddr_vm.
            let addr = unsafe { &*(self.as_ptr() as *const libc::sockaddr_vm) };
            Some((addr.svm_cid, addr.svm_port))
        } else {
            None
        }
    }
}

pub(crate) type Socket = c_int;

pub(crate) unsafe fn socket_from_raw(socket: Socket) -> crate::socket::Inner {
    crate::socket::Inner::from_raw_fd(socket)
}

pub(crate) fn socket_as_raw(socket: &crate::socket::Inner) -> Socket {
    socket.as_raw_fd()
}

pub(crate) fn socket_into_raw(socket: crate::socket::Inner) -> Socket {
    socket.into_raw_fd()
}

pub(crate) fn socket(family: c_int, ty: c_int, protocol: c_int) -> io::Result<Socket> {
    syscall!(socket(family, ty, protocol))
}

#[cfg(feature = "all")]
pub(crate) fn socketpair(family: c_int, ty: c_int, protocol: c_int) -> io::Result<[Socket; 2]> {
    let mut fds = [0, 0];
    syscall!(socketpair(family, ty, protocol, fds.as_mut_ptr())).map(|_| fds)
}

pub(crate) fn bind(fd: Socket, addr: &SockAddr) -> io::Result<()> {
    syscall!(bind(fd, addr.as_ptr(), addr.len() as _)).map(|_| ())
}

pub(crate) fn connect(fd: Socket, addr: &SockAddr) -> io::Result<()> {
    syscall!(connect(fd, addr.as_ptr(), addr.len())).map(|_| ())
}

pub(crate) fn poll_connect(socket: &crate::Socket, timeout: Duration) -> io::Result<()> {
    let start = Instant::now();

    let mut pollfd = libc::pollfd {
        fd: socket.as_raw(),
        events: libc::POLLIN | libc::POLLOUT,
        revents: 0,
    };

    loop {
        let elapsed = start.elapsed();
        if elapsed >= timeout {
            return Err(io::ErrorKind::TimedOut.into());
        }

        let timeout = (timeout - elapsed).as_millis();
        let timeout = clamp(timeout, 1, c_int::max_value() as u128) as c_int;

        match syscall!(poll(&mut pollfd, 1, timeout)) {
            Ok(0) => return Err(io::ErrorKind::TimedOut.into()),
            Ok(_) => {
                // Error or hang up indicates an error (or failure to connect).
                if (pollfd.revents & libc::POLLHUP) != 0 || (pollfd.revents & libc::POLLERR) != 0 {
                    match socket.take_error() {
                        Ok(Some(err)) => return Err(err),
                        Ok(None) => {
                            return Err(io::Error::new(
                                io::ErrorKind::Other,
                                "no error set after POLLHUP",
                            ))
                        }
                        Err(err) => return Err(err),
                    }
                }
                return Ok(());
            }
            // Got interrupted, try again.
            Err(ref err) if err.kind() == io::ErrorKind::Interrupted => continue,
            Err(err) => return Err(err),
        }
    }
}

// TODO: use clamp from std lib, stable since 1.50.
fn clamp<T>(value: T, min: T, max: T) -> T
where
    T: Ord,
{
    if value <= min {
        min
    } else if value >= max {
        max
    } else {
        value
    }
}

pub(crate) fn listen(fd: Socket, backlog: c_int) -> io::Result<()> {
    syscall!(listen(fd, backlog)).map(|_| ())
}

pub(crate) fn accept(fd: Socket) -> io::Result<(Socket, SockAddr)> {
    // Safety: `accept` initialises the `SockAddr` for us.
    unsafe { SockAddr::init(|storage, len| syscall!(accept(fd, storage.cast(), len))) }
}

pub(crate) fn getsockname(fd: Socket) -> io::Result<SockAddr> {
    // Safety: `accept` initialises the `SockAddr` for us.
    unsafe { SockAddr::init(|storage, len| syscall!(getsockname(fd, storage.cast(), len))) }
        .map(|(_, addr)| addr)
}

pub(crate) fn getpeername(fd: Socket) -> io::Result<SockAddr> {
    // Safety: `accept` initialises the `SockAddr` for us.
    unsafe { SockAddr::init(|storage, len| syscall!(getpeername(fd, storage.cast(), len))) }
        .map(|(_, addr)| addr)
}

pub(crate) fn try_clone(fd: Socket) -> io::Result<Socket> {
    syscall!(fcntl(fd, libc::F_DUPFD_CLOEXEC, 0))
}

pub(crate) fn set_nonblocking(fd: Socket, nonblocking: bool) -> io::Result<()> {
    if nonblocking {
        fcntl_add(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
    } else {
        fcntl_remove(fd, libc::F_GETFL, libc::F_SETFL, libc::O_NONBLOCK)
    }
}

pub(crate) fn shutdown(fd: Socket, how: Shutdown) -> io::Result<()> {
    let how = match how {
        Shutdown::Write => libc::SHUT_WR,
        Shutdown::Read => libc::SHUT_RD,
        Shutdown::Both => libc::SHUT_RDWR,
    };
    syscall!(shutdown(fd, how)).map(|_| ())
}

pub(crate) fn recv(fd: Socket, buf: &mut [MaybeUninit<u8>], flags: c_int) -> io::Result<usize> {
    syscall!(recv(
        fd,
        buf.as_mut_ptr().cast(),
        min(buf.len(), MAX_BUF_LEN),
        flags,
    ))
    .map(|n| n as usize)
}

pub(crate) fn recv_from(
    fd: Socket,
    buf: &mut [MaybeUninit<u8>],
    flags: c_int,
) -> io::Result<(usize, SockAddr)> {
    // Safety: `recvfrom` initialises the `SockAddr` for us.
    unsafe {
        SockAddr::init(|addr, addrlen| {
            syscall!(recvfrom(
                fd,
                buf.as_mut_ptr().cast(),
                min(buf.len(), MAX_BUF_LEN),
                flags,
                addr.cast(),
                addrlen
            ))
            .map(|n| n as usize)
        })
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn recv_vectored(
    fd: Socket,
    bufs: &mut [crate::MaybeUninitSlice<'_>],
    flags: c_int,
) -> io::Result<(usize, RecvFlags)> {
    recvmsg(fd, ptr::null_mut(), bufs, flags).map(|(n, _, recv_flags)| (n, recv_flags))
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn recv_from_vectored(
    fd: Socket,
    bufs: &mut [crate::MaybeUninitSlice<'_>],
    flags: c_int,
) -> io::Result<(usize, RecvFlags, SockAddr)> {
    // Safety: `recvmsg` initialises the address storage and we set the length
    // manually.
    unsafe {
        SockAddr::init(|storage, len| {
            recvmsg(fd, storage, bufs, flags).map(|(n, addrlen, recv_flags)| {
                // Set the correct address length.
                *len = addrlen;
                (n, recv_flags)
            })
        })
    }
    .map(|((n, recv_flags), addr)| (n, recv_flags, addr))
}

/// Returns the (bytes received, sending address len, `RecvFlags`).
#[cfg(not(target_os = "redox"))]
fn recvmsg(
    fd: Socket,
    msg_name: *mut sockaddr_storage,
    bufs: &mut [crate::MaybeUninitSlice<'_>],
    flags: c_int,
) -> io::Result<(usize, libc::socklen_t, RecvFlags)> {
    let msg_namelen = if msg_name.is_null() {
        0
    } else {
        size_of::<sockaddr_storage>() as libc::socklen_t
    };
    // libc::msghdr contains unexported padding fields on Fuchsia.
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    msg.msg_name = msg_name.cast();
    msg.msg_namelen = msg_namelen;
    msg.msg_iov = bufs.as_mut_ptr().cast();
    msg.msg_iovlen = min(bufs.len(), IovLen::MAX as usize) as IovLen;
    syscall!(recvmsg(fd, &mut msg, flags))
        .map(|n| (n as usize, msg.msg_namelen, RecvFlags(msg.msg_flags)))
}

pub(crate) fn send(fd: Socket, buf: &[u8], flags: c_int) -> io::Result<usize> {
    syscall!(send(
        fd,
        buf.as_ptr().cast(),
        min(buf.len(), MAX_BUF_LEN),
        flags,
    ))
    .map(|n| n as usize)
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn send_vectored(fd: Socket, bufs: &[IoSlice<'_>], flags: c_int) -> io::Result<usize> {
    sendmsg(fd, ptr::null(), 0, bufs, flags)
}

pub(crate) fn send_to(fd: Socket, buf: &[u8], addr: &SockAddr, flags: c_int) -> io::Result<usize> {
    syscall!(sendto(
        fd,
        buf.as_ptr().cast(),
        min(buf.len(), MAX_BUF_LEN),
        flags,
        addr.as_ptr(),
        addr.len(),
    ))
    .map(|n| n as usize)
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn send_to_vectored(
    fd: Socket,
    bufs: &[IoSlice<'_>],
    addr: &SockAddr,
    flags: c_int,
) -> io::Result<usize> {
    sendmsg(fd, addr.as_storage_ptr(), addr.len(), bufs, flags)
}

/// Returns the (bytes received, sending address len, `RecvFlags`).
#[cfg(not(target_os = "redox"))]
fn sendmsg(
    fd: Socket,
    msg_name: *const sockaddr_storage,
    msg_namelen: socklen_t,
    bufs: &[IoSlice<'_>],
    flags: c_int,
) -> io::Result<usize> {
    // libc::msghdr contains unexported padding fields on Fuchsia.
    let mut msg: libc::msghdr = unsafe { mem::zeroed() };
    // Safety: we're creating a `*mut` pointer from a reference, which is UB
    // once actually used. However the OS should not write to it in the
    // `sendmsg` system call.
    msg.msg_name = (msg_name as *mut sockaddr_storage).cast();
    msg.msg_namelen = msg_namelen;
    // Safety: Same as above about `*const` -> `*mut`.
    msg.msg_iov = bufs.as_ptr() as *mut _;
    msg.msg_iovlen = min(bufs.len(), IovLen::MAX as usize) as IovLen;
    syscall!(sendmsg(fd, &msg, flags)).map(|n| n as usize)
}

/// Wrapper around `getsockopt` to deal with platform specific timeouts.
pub(crate) fn timeout_opt(fd: Socket, opt: c_int, val: c_int) -> io::Result<Option<Duration>> {
    unsafe { getsockopt(fd, opt, val).map(from_timeval) }
}

fn from_timeval(duration: libc::timeval) -> Option<Duration> {
    if duration.tv_sec == 0 && duration.tv_usec == 0 {
        None
    } else {
        let sec = duration.tv_sec as u64;
        let nsec = (duration.tv_usec as u32) * 1000;
        Some(Duration::new(sec, nsec))
    }
}

/// Wrapper around `setsockopt` to deal with platform specific timeouts.
pub(crate) fn set_timeout_opt(
    fd: Socket,
    opt: c_int,
    val: c_int,
    duration: Option<Duration>,
) -> io::Result<()> {
    let duration = into_timeval(duration);
    unsafe { setsockopt(fd, opt, val, duration) }
}

fn into_timeval(duration: Option<Duration>) -> libc::timeval {
    match duration {
        // https://github.com/rust-lang/libc/issues/1848
        #[cfg_attr(target_env = "musl", allow(deprecated))]
        Some(duration) => libc::timeval {
            tv_sec: min(duration.as_secs(), libc::time_t::max_value() as u64) as libc::time_t,
            tv_usec: duration.subsec_micros() as libc::suseconds_t,
        },
        None => libc::timeval {
            tv_sec: 0,
            tv_usec: 0,
        },
    }
}

#[cfg(feature = "all")]
#[cfg(not(any(target_os = "haiku", target_os = "openbsd")))]
pub(crate) fn keepalive_time(fd: Socket) -> io::Result<Duration> {
    unsafe {
        getsockopt::<c_int>(fd, IPPROTO_TCP, KEEPALIVE_TIME)
            .map(|secs| Duration::from_secs(secs as u64))
    }
}

#[allow(unused_variables)]
pub(crate) fn set_tcp_keepalive(fd: Socket, keepalive: &TcpKeepalive) -> io::Result<()> {
    #[cfg(not(any(target_os = "haiku", target_os = "openbsd")))]
    if let Some(time) = keepalive.time {
        let secs = into_secs(time);
        unsafe { setsockopt(fd, libc::IPPROTO_TCP, KEEPALIVE_TIME, secs)? }
    }

    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_vendor = "apple",
    ))]
    {
        if let Some(interval) = keepalive.interval {
            let secs = into_secs(interval);
            unsafe { setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPINTVL, secs)? }
        }

        if let Some(retries) = keepalive.retries {
            unsafe { setsockopt(fd, libc::IPPROTO_TCP, libc::TCP_KEEPCNT, retries as c_int)? }
        }
    }

    Ok(())
}

#[cfg(not(any(target_os = "haiku", target_os = "openbsd")))]
fn into_secs(duration: Duration) -> c_int {
    min(duration.as_secs(), c_int::max_value() as u64) as c_int
}

/// Add `flag` to the current set flags of `F_GETFD`.
fn fcntl_add(fd: Socket, get_cmd: c_int, set_cmd: c_int, flag: c_int) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous | flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

/// Remove `flag` to the current set flags of `F_GETFD`.
fn fcntl_remove(fd: Socket, get_cmd: c_int, set_cmd: c_int, flag: c_int) -> io::Result<()> {
    let previous = syscall!(fcntl(fd, get_cmd))?;
    let new = previous & !flag;
    if new != previous {
        syscall!(fcntl(fd, set_cmd, new)).map(|_| ())
    } else {
        // Flag was already set.
        Ok(())
    }
}

/// Caller must ensure `T` is the correct type for `opt` and `val`.
pub(crate) unsafe fn getsockopt<T>(fd: Socket, opt: c_int, val: c_int) -> io::Result<T> {
    let mut payload: MaybeUninit<T> = MaybeUninit::uninit();
    let mut len = size_of::<T>() as libc::socklen_t;
    syscall!(getsockopt(
        fd,
        opt,
        val,
        payload.as_mut_ptr().cast(),
        &mut len,
    ))
    .map(|_| {
        debug_assert_eq!(len as usize, size_of::<T>());
        // Safety: `getsockopt` initialised `payload` for us.
        payload.assume_init()
    })
}

/// Caller must ensure `T` is the correct type for `opt` and `val`.
pub(crate) unsafe fn setsockopt<T>(
    fd: Socket,
    opt: c_int,
    val: c_int,
    payload: T,
) -> io::Result<()> {
    let payload = &payload as *const T as *const c_void;
    syscall!(setsockopt(
        fd,
        opt,
        val,
        payload,
        mem::size_of::<T>() as libc::socklen_t,
    ))
    .map(|_| ())
}

pub(crate) fn to_in_addr(addr: &Ipv4Addr) -> in_addr {
    // `s_addr` is stored as BE on all machines, and the array is in BE order.
    // So the native endian conversion method is used so that it's never
    // swapped.
    in_addr {
        s_addr: u32::from_ne_bytes(addr.octets()),
    }
}

pub(crate) fn from_in_addr(in_addr: in_addr) -> Ipv4Addr {
    Ipv4Addr::from(in_addr.s_addr.to_ne_bytes())
}

pub(crate) fn to_in6_addr(addr: &Ipv6Addr) -> in6_addr {
    in6_addr {
        s6_addr: addr.octets(),
    }
}

pub(crate) fn from_in6_addr(addr: in6_addr) -> Ipv6Addr {
    Ipv6Addr::from(addr.s6_addr)
}

#[cfg(not(any(
    target_os = "haiku",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "solaris",
)))]
pub(crate) fn to_mreqn(
    multiaddr: &Ipv4Addr,
    interface: &crate::socket::InterfaceIndexOrAddress,
) -> libc::ip_mreqn {
    match interface {
        crate::socket::InterfaceIndexOrAddress::Index(interface) => libc::ip_mreqn {
            imr_multiaddr: to_in_addr(multiaddr),
            imr_address: to_in_addr(&Ipv4Addr::UNSPECIFIED),
            imr_ifindex: *interface as _,
        },
        crate::socket::InterfaceIndexOrAddress::Address(interface) => libc::ip_mreqn {
            imr_multiaddr: to_in_addr(multiaddr),
            imr_address: to_in_addr(interface),
            imr_ifindex: 0,
        },
    }
}

/// Unix only API.
impl crate::Socket {
    /// Accept a new incoming connection from this listener.
    ///
    /// This function directly corresponds to the `accept4(2)` function.
    ///
    /// This function will block the calling thread until a new connection is
    /// established. When established, the corresponding `Socket` and the remote
    /// peer's address will be returned.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "illumos",
            target_os = "linux",
            target_os = "netbsd",
            target_os = "openbsd"
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "illumos",
                target_os = "linux",
                target_os = "netbsd",
                target_os = "openbsd"
            )
        )))
    )]
    pub fn accept4(&self, flags: c_int) -> io::Result<(crate::Socket, SockAddr)> {
        self._accept4(flags)
    }

    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "fuchsia",
        target_os = "illumos",
        target_os = "linux",
        target_os = "netbsd",
        target_os = "openbsd"
    ))]
    pub(crate) fn _accept4(&self, flags: c_int) -> io::Result<(crate::Socket, SockAddr)> {
        // Safety: `accept4` initialises the `SockAddr` for us.
        unsafe {
            SockAddr::init(|storage, len| {
                syscall!(accept4(self.as_raw(), storage.cast(), len, flags))
                    .map(crate::Socket::from_raw)
            })
        }
    }

    /// Sets `CLOEXEC` on the socket.
    ///
    /// # Notes
    ///
    /// On supported platforms you can use [`Type::cloexec`].
    #[cfg(feature = "all")]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", unix))))]
    pub fn set_cloexec(&self, close_on_exec: bool) -> io::Result<()> {
        self._set_cloexec(close_on_exec)
    }

    pub(crate) fn _set_cloexec(&self, close_on_exec: bool) -> io::Result<()> {
        if close_on_exec {
            fcntl_add(
                self.as_raw(),
                libc::F_GETFD,
                libc::F_SETFD,
                libc::FD_CLOEXEC,
            )
        } else {
            fcntl_remove(
                self.as_raw(),
                libc::F_GETFD,
                libc::F_SETFD,
                libc::FD_CLOEXEC,
            )
        }
    }

    /// Sets `SO_NOSIGPIPE` on the socket.
    #[cfg(all(feature = "all", any(doc, target_vendor = "apple")))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", target_vendor = "apple"))))]
    pub fn set_nosigpipe(&self, nosigpipe: bool) -> io::Result<()> {
        self._set_nosigpipe(nosigpipe)
    }

    #[cfg(target_vendor = "apple")]
    pub(crate) fn _set_nosigpipe(&self, nosigpipe: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_NOSIGPIPE,
                nosigpipe as c_int,
            )
        }
    }

    /// Gets the value of the `TCP_MAXSEG` option on this socket.
    ///
    /// For more information about this option, see [`set_mss`].
    ///
    /// [`set_mss`]: crate::Socket::set_mss
    #[cfg(all(feature = "all", not(target_os = "redox")))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", unix, not(target_os = "redox")))))]
    pub fn mss(&self) -> io::Result<u32> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::IPPROTO_TCP, libc::TCP_MAXSEG)
                .map(|mss| mss as u32)
        }
    }

    /// Sets the value of the `TCP_MAXSEG` option on this socket.
    ///
    /// The `TCP_MAXSEG` option denotes the TCP Maximum Segment Size and is only
    /// available on TCP sockets.
    #[cfg(all(feature = "all", not(target_os = "redox")))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", unix, not(target_os = "redox")))))]
    pub fn set_mss(&self, mss: u32) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_MAXSEG,
                mss as c_int,
            )
        }
    }

    /// Returns `true` if `listen(2)` was called on this socket by checking the
    /// `SO_ACCEPTCONN` option on this socket.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "linux",
            )
        )))
    )]
    pub fn is_listener(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_ACCEPTCONN)
                .map(|v| v != 0)
        }
    }

    /// Returns the [`Domain`] of this socket by checking the `SO_DOMAIN` option
    /// on this socket.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            // TODO: add FreeBSD.
            // target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
        )
    ))]
    #[cfg_attr(docsrs, doc(cfg(all(
        feature = "all",
        any(
            target_os = "android",
            // TODO: add FreeBSD.
            // target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
        )
    ))))]
    pub fn domain(&self) -> io::Result<Domain> {
        unsafe { getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_DOMAIN).map(Domain) }
    }

    /// Returns the [`Protocol`] of this socket by checking the `SO_PROTOCOL`
    /// option on this socket.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "linux",
            )
        )))
    )]
    pub fn protocol(&self) -> io::Result<Option<Protocol>> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_PROTOCOL).map(|v| match v
            {
                0 => None,
                p => Some(Protocol(p)),
            })
        }
    }

    /// Gets the value for the `SO_MARK` option on this socket.
    ///
    /// This value gets the socket mark field for each packet sent through
    /// this socket.
    ///
    /// On Linux this function requires the `CAP_NET_ADMIN` capability.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn mark(&self) -> io::Result<u32> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_MARK)
                .map(|mark| mark as u32)
        }
    }

    /// Sets the value for the `SO_MARK` option on this socket.
    ///
    /// This value sets the socket mark field for each packet sent through
    /// this socket. Changing the mark can be used for mark-based routing
    /// without netfilter or for packet filtering.
    ///
    /// On Linux this function requires the `CAP_NET_ADMIN` capability.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_mark(&self, mark: u32) -> io::Result<()> {
        unsafe {
            setsockopt::<c_int>(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_MARK,
                mark as c_int,
            )
        }
    }

    /// Get the value of the `TCP_CORK` option on this socket.
    ///
    /// For more information about this option, see [`set_cork`].
    ///
    /// [`set_cork`]: Socket::set_cork
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn cork(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<Bool>(self.as_raw(), libc::IPPROTO_TCP, libc::TCP_CORK)
                .map(|cork| cork != 0)
        }
    }

    /// Set the value of the `TCP_CORK` option on this socket.
    ///
    /// If set, don't send out partial frames. All queued partial frames are
    /// sent when the option is cleared again. There is a 200 millisecond ceiling on
    /// the time for which output is corked by `TCP_CORK`. If this ceiling is reached,
    /// then queued data is automatically transmitted.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_cork(&self, cork: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_CORK,
                cork as c_int,
            )
        }
    }

    /// Get the value of the `TCP_QUICKACK` option on this socket.
    ///
    /// For more information about this option, see [`set_quickack`].
    ///
    /// [`set_quickack`]: Socket::set_quickack
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn quickack(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<Bool>(self.as_raw(), libc::IPPROTO_TCP, libc::TCP_QUICKACK)
                .map(|quickack| quickack != 0)
        }
    }

    /// Set the value of the `TCP_QUICKACK` option on this socket.
    ///
    /// If set, acks are sent immediately, rather than delayed if needed in accordance to normal
    /// TCP operation. This flag is not permanent, it only enables a switch to or from quickack mode.
    /// Subsequent operation of the TCP protocol will once again enter/leave quickack mode depending on
    /// internal protocol processing and factors such as delayed ack timeouts occurring and data transfer.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_quickack(&self, quickack: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_QUICKACK,
                quickack as c_int,
            )
        }
    }

    /// Get the value of the `TCP_THIN_LINEAR_TIMEOUTS` option on this socket.
    ///
    /// For more information about this option, see [`set_thin_linear_timeouts`].
    ///
    /// [`set_thin_linear_timeouts`]: Socket::set_thin_linear_timeouts
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn thin_linear_timeouts(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<Bool>(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_THIN_LINEAR_TIMEOUTS,
            )
            .map(|timeouts| timeouts != 0)
        }
    }

    /// Set the value of the `TCP_THIN_LINEAR_TIMEOUTS` option on this socket.
    ///    
    /// If set, the kernel will dynamically detect a thin-stream connection if there are less than four packets in flight.
    /// With less than four packets in flight the normal TCP fast retransmission will not be effective.
    /// The kernel will modify the retransmission to avoid the very high latencies that thin stream suffer because of exponential backoff.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_thin_linear_timeouts(&self, timeouts: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_THIN_LINEAR_TIMEOUTS,
                timeouts as c_int,
            )
        }
    }

    /// Gets the value for the `SO_BINDTODEVICE` option on this socket.
    ///
    /// This value gets the socket binded device's interface name.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn device(&self) -> io::Result<Option<Vec<u8>>> {
        // TODO: replace with `MaybeUninit::uninit_array` once stable.
        let mut buf: [MaybeUninit<u8>; libc::IFNAMSIZ] =
            unsafe { MaybeUninit::uninit().assume_init() };
        let mut len = buf.len() as libc::socklen_t;
        unsafe {
            syscall!(getsockopt(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_BINDTODEVICE,
                buf.as_mut_ptr().cast(),
                &mut len,
            ))?;
        }
        if len == 0 {
            Ok(None)
        } else {
            let buf = &buf[..len as usize - 1];
            // TODO: use `MaybeUninit::slice_assume_init_ref` once stable.
            Ok(Some(unsafe { &*(buf as *const [_] as *const [u8]) }.into()))
        }
    }

    /// Sets the value for the `SO_BINDTODEVICE` option on this socket.
    ///
    /// If a socket is bound to an interface, only packets received from that
    /// particular interface are processed by the socket. Note that this only
    /// works for some socket types, particularly `AF_INET` sockets.
    ///
    /// If `interface` is `None` or an empty string it removes the binding.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn bind_device(&self, interface: Option<&[u8]>) -> io::Result<()> {
        let (value, len) = if let Some(interface) = interface {
            (interface.as_ptr(), interface.len())
        } else {
            (ptr::null(), 0)
        };
        syscall!(setsockopt(
            self.as_raw(),
            libc::SOL_SOCKET,
            libc::SO_BINDTODEVICE,
            value.cast(),
            len as libc::socklen_t,
        ))
        .map(|_| ())
    }

    /// Sets the value for the `SO_SETFIB` option on this socket.
    ///
    /// Bind socket to the specified forwarding table (VRF) on a FreeBSD.
    #[cfg(all(feature = "all", any(target_os = "freebsd")))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", any(target_os = "freebsd")))))]
    pub fn set_fib(&self, fib: u32) -> io::Result<()> {
        syscall!(setsockopt(
            self.as_raw(),
            libc::SOL_SOCKET,
            libc::SO_SETFIB,
            (&fib as *const u32).cast(),
            mem::size_of::<u32>() as libc::socklen_t,
        ))
        .map(|_| ())
    }

    /// Sets the value for `IP_BOUND_IF` option on this socket.
    ///
    /// If a socket is bound to an interface, only packets received from that
    /// particular interface are processed by the socket.
    ///
    /// If `interface` is `None`, the binding is removed. If the `interface`
    /// index is not valid, an error is returned.
    ///
    /// One can use `libc::if_nametoindex` to convert an interface alias to an
    /// index.
    #[cfg(all(feature = "all", target_vendor = "apple"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", target_vendor = "apple"))))]
    pub fn bind_device_by_index(&self, interface: Option<NonZeroU32>) -> io::Result<()> {
        let index = interface.map(NonZeroU32::get).unwrap_or(0);
        unsafe { setsockopt(self.as_raw(), IPPROTO_IP, libc::IP_BOUND_IF, index) }
    }

    /// Gets the value for `IP_BOUND_IF` option on this socket, i.e. the index
    /// for the interface to which the socket is bound.
    ///
    /// Returns `None` if the socket is not bound to any interface, otherwise
    /// returns an interface index.
    #[cfg(all(feature = "all", target_vendor = "apple"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", target_vendor = "apple"))))]
    pub fn device_index(&self) -> io::Result<Option<NonZeroU32>> {
        let index =
            unsafe { getsockopt::<libc::c_uint>(self.as_raw(), IPPROTO_IP, libc::IP_BOUND_IF)? };
        Ok(NonZeroU32::new(index))
    }

    /// Get the value of the `SO_INCOMING_CPU` option on this socket.
    ///
    /// For more information about this option, see [`set_cpu_affinity`].
    ///
    /// [`set_cpu_affinity`]: crate::Socket::set_cpu_affinity
    #[cfg(all(feature = "all", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", target_os = "linux"))))]
    pub fn cpu_affinity(&self) -> io::Result<usize> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_INCOMING_CPU)
                .map(|cpu| cpu as usize)
        }
    }

    /// Set value for the `SO_INCOMING_CPU` option on this socket.
    ///
    /// Sets the CPU affinity of the socket.
    #[cfg(all(feature = "all", target_os = "linux"))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", target_os = "linux"))))]
    pub fn set_cpu_affinity(&self, cpu: usize) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_INCOMING_CPU,
                cpu as c_int,
            )
        }
    }

    /// Get the value of the `SO_REUSEPORT` option on this socket.
    ///
    /// For more information about this option, see [`set_reuse_port`].
    ///
    /// [`set_reuse_port`]: crate::Socket::set_reuse_port
    #[cfg(all(
        feature = "all",
        not(any(target_os = "solaris", target_os = "illumos"))
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            unix,
            not(any(target_os = "solaris", target_os = "illumos"))
        )))
    )]
    pub fn reuse_port(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_SOCKET, libc::SO_REUSEPORT)
                .map(|reuse| reuse != 0)
        }
    }

    /// Set value for the `SO_REUSEPORT` option on this socket.
    ///
    /// This indicates that further calls to `bind` may allow reuse of local
    /// addresses. For IPv4 sockets this means that a socket may bind even when
    /// there's a socket already listening on this port.
    #[cfg(all(
        feature = "all",
        not(any(target_os = "solaris", target_os = "illumos"))
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            unix,
            not(any(target_os = "solaris", target_os = "illumos"))
        )))
    )]
    pub fn set_reuse_port(&self, reuse: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_REUSEPORT,
                reuse as c_int,
            )
        }
    }

    /// Get the value of the `IP_FREEBIND` option on this socket.
    ///
    /// For more information about this option, see [`set_freebind`].
    ///
    /// [`set_freebind`]: crate::Socket::set_freebind
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn freebind(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_IP, libc::IP_FREEBIND)
                .map(|freebind| freebind != 0)
        }
    }

    /// Set value for the `IP_FREEBIND` option on this socket.
    ///
    /// If enabled, this boolean option allows binding to an IP address that is
    /// nonlocal or does not (yet) exist.  This permits listening on a socket,
    /// without requiring the underlying network interface or the specified
    /// dynamic IP address to be up at the time that the application is trying
    /// to bind to it.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_freebind(&self, freebind: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_IP,
                libc::IP_FREEBIND,
                freebind as c_int,
            )
        }
    }

    /// Get the value of the `IPV6_FREEBIND` option on this socket.
    ///
    /// This is an IPv6 counterpart of `IP_FREEBIND` socket option on
    /// Android/Linux. For more information about this option, see
    /// [`set_freebind`].
    ///
    /// [`set_freebind`]: crate::Socket::set_freebind
    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "all", any(target_os = "android", target_os = "linux"))))
    )]
    pub fn freebind_ipv6(&self) -> io::Result<bool> {
        unsafe {
            getsockopt::<c_int>(self.as_raw(), libc::SOL_IPV6, libc::IPV6_FREEBIND)
                .map(|freebind| freebind != 0)
        }
    }

    /// Set value for the `IPV6_FREEBIND` option on this socket.
    ///
    /// This is an IPv6 counterpart of `IP_FREEBIND` socket option on
    /// Android/Linux. For more information about this option, see
    /// [`set_freebind`].
    ///
    /// [`set_freebind`]: crate::Socket::set_freebind
    ///
    /// # Examples
    ///
    /// On Linux:
    ///
    /// ```
    /// use socket2::{Domain, Socket, Type};
    /// use std::io::{self, Error, ErrorKind};
    ///
    /// fn enable_freebind(socket: &Socket) -> io::Result<()> {
    ///     match socket.domain()? {
    ///         Domain::IPV4 => socket.set_freebind(true)?,
    ///         Domain::IPV6 => socket.set_freebind_ipv6(true)?,
    ///         _ => return Err(Error::new(ErrorKind::Other, "unsupported domain")),
    ///     };
    ///     Ok(())
    /// }
    ///
    /// # fn main() -> io::Result<()> {
    /// #     let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;
    /// #     enable_freebind(&socket)
    /// # }
    /// ```
    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(feature = "all", any(target_os = "android", target_os = "linux"))))
    )]
    pub fn set_freebind_ipv6(&self, freebind: bool) -> io::Result<()> {
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_IPV6,
                libc::IPV6_FREEBIND,
                freebind as c_int,
            )
        }
    }

    /// Copies data between a `file` and this socket using the `sendfile(2)`
    /// system call. Because this copying is done within the kernel,
    /// `sendfile()` is more efficient than the combination of `read(2)` and
    /// `write(2)`, which would require transferring data to and from user
    /// space.
    ///
    /// Different OSs support different kinds of `file`s, see the OS
    /// documentation for what kind of files are supported. Generally *regular*
    /// files are supported by all OSs.
    ///
    /// The `offset` is the absolute offset into the `file` to use as starting
    /// point.
    ///
    /// Depending on the OS this function *may* change the offset of `file`. For
    /// the best results reset the offset of the file before using it again.
    ///
    /// The `length` determines how many bytes to send, where a length of `None`
    /// means it will try to send all bytes.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "android",
            target_os = "freebsd",
            target_os = "linux",
            target_vendor = "apple",
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "android",
                target_os = "freebsd",
                target_os = "linux",
                target_vendor = "apple",
            )
        )))
    )]
    pub fn sendfile<F>(
        &self,
        file: &F,
        offset: usize,
        length: Option<NonZeroUsize>,
    ) -> io::Result<usize>
    where
        F: AsRawFd,
    {
        self._sendfile(file.as_raw_fd(), offset as _, length)
    }

    #[cfg(all(feature = "all", target_vendor = "apple"))]
    fn _sendfile(
        &self,
        file: RawFd,
        offset: libc::off_t,
        length: Option<NonZeroUsize>,
    ) -> io::Result<usize> {
        // On macOS `length` is value-result parameter. It determines the number
        // of bytes to write and returns the number of bytes written.
        let mut length = match length {
            Some(n) => n.get() as libc::off_t,
            // A value of `0` means send all bytes.
            None => 0,
        };
        syscall!(sendfile(
            file,
            self.as_raw(),
            offset,
            &mut length,
            ptr::null_mut(),
            0,
        ))
        .map(|_| length as usize)
    }

    #[cfg(all(feature = "all", any(target_os = "android", target_os = "linux")))]
    fn _sendfile(
        &self,
        file: RawFd,
        offset: libc::off_t,
        length: Option<NonZeroUsize>,
    ) -> io::Result<usize> {
        let count = match length {
            Some(n) => n.get() as libc::size_t,
            // The maximum the Linux kernel will write in a single call.
            None => 0x7ffff000, // 2,147,479,552 bytes.
        };
        let mut offset = offset;
        syscall!(sendfile(self.as_raw(), file, &mut offset, count)).map(|n| n as usize)
    }

    #[cfg(all(feature = "all", target_os = "freebsd"))]
    fn _sendfile(
        &self,
        file: RawFd,
        offset: libc::off_t,
        length: Option<NonZeroUsize>,
    ) -> io::Result<usize> {
        let nbytes = match length {
            Some(n) => n.get() as libc::size_t,
            // A value of `0` means send all bytes.
            None => 0,
        };
        let mut sbytes: libc::off_t = 0;
        syscall!(sendfile(
            file,
            self.as_raw(),
            offset,
            nbytes,
            ptr::null_mut(),
            &mut sbytes,
            0,
        ))
        .map(|_| sbytes as usize)
    }

    /// Set the value of the `TCP_USER_TIMEOUT` option on this socket.
    ///
    /// If set, this specifies the maximum amount of time that transmitted data may remain
    /// unacknowledged or buffered data may remain untransmitted before TCP will forcibly close the
    /// corresponding connection.
    ///
    /// Setting `timeout` to `None` or a zero duration causes the system default timeouts to
    /// be used. If `timeout` in milliseconds is larger than `c_uint::MAX`, the timeout is clamped
    /// to `c_uint::MAX`. For example, when `c_uint` is a 32-bit value, this limits the timeout to
    /// approximately 49.71 days.
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn set_tcp_user_timeout(&self, timeout: Option<Duration>) -> io::Result<()> {
        let timeout = timeout
            .map(|to| min(to.as_millis(), libc::c_uint::MAX as u128) as libc::c_uint)
            .unwrap_or(0);
        unsafe {
            setsockopt(
                self.as_raw(),
                libc::IPPROTO_TCP,
                libc::TCP_USER_TIMEOUT,
                timeout,
            )
        }
    }

    /// Get the value of the `TCP_USER_TIMEOUT` option on this socket.
    ///
    /// For more information about this option, see [`set_tcp_user_timeout`].
    ///
    /// [`set_tcp_user_timeout`]: Socket::set_tcp_user_timeout
    #[cfg(all(
        feature = "all",
        any(target_os = "android", target_os = "fuchsia", target_os = "linux")
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(target_os = "android", target_os = "fuchsia", target_os = "linux")
        )))
    )]
    pub fn tcp_user_timeout(&self) -> io::Result<Option<Duration>> {
        unsafe {
            getsockopt::<libc::c_uint>(self.as_raw(), libc::IPPROTO_TCP, libc::TCP_USER_TIMEOUT)
                .map(|millis| {
                    if millis == 0 {
                        None
                    } else {
                        Some(Duration::from_millis(millis as u64))
                    }
                })
        }
    }

    /// Attach Berkeley Packet Filter(BPF) on this socket.
    ///
    /// BPF allows a user-space program to attach a filter onto any socket
    /// and allow or disallow certain types of data to come through the socket.
    ///
    /// For more information about this option, see [filter](https://www.kernel.org/doc/html/v5.12/networking/filter.html)
    #[cfg(all(feature = "all", any(target_os = "linux", target_os = "android")))]
    pub fn attach_filter(&self, filters: &[libc::sock_filter]) -> io::Result<()> {
        let prog = libc::sock_fprog {
            len: filters.len() as u16,
            filter: filters.as_ptr() as *mut _,
        };

        unsafe {
            setsockopt(
                self.as_raw(),
                libc::SOL_SOCKET,
                libc::SO_ATTACH_FILTER,
                prog,
            )
        }
    }

    /// Detach Berkeley Packet Filter(BPF) from this socket.
    ///
    /// For more information about this option, see [`attach_filter`]
    #[cfg(all(feature = "all", any(target_os = "linux", target_os = "android")))]
    pub fn detach_filter(&self) -> io::Result<()> {
        unsafe { setsockopt(self.as_raw(), libc::SOL_SOCKET, libc::SO_DETACH_FILTER, 0) }
    }
}

#[cfg_attr(docsrs, doc(cfg(unix)))]
impl AsRawFd for crate::Socket {
    fn as_raw_fd(&self) -> c_int {
        self.as_raw()
    }
}

#[cfg_attr(docsrs, doc(cfg(unix)))]
impl IntoRawFd for crate::Socket {
    fn into_raw_fd(self) -> c_int {
        self.into_raw()
    }
}

#[cfg_attr(docsrs, doc(cfg(unix)))]
impl FromRawFd for crate::Socket {
    unsafe fn from_raw_fd(fd: c_int) -> crate::Socket {
        crate::Socket::from_raw(fd)
    }
}

#[cfg(feature = "all")]
from!(UnixStream, crate::Socket);
#[cfg(feature = "all")]
from!(UnixListener, crate::Socket);
#[cfg(feature = "all")]
from!(UnixDatagram, crate::Socket);
#[cfg(feature = "all")]
from!(crate::Socket, UnixStream);
#[cfg(feature = "all")]
from!(crate::Socket, UnixListener);
#[cfg(feature = "all")]
from!(crate::Socket, UnixDatagram);

#[test]
fn in_addr_convertion() {
    let ip = Ipv4Addr::new(127, 0, 0, 1);
    let raw = to_in_addr(&ip);
    // NOTE: `in_addr` is packed on NetBSD and it's unsafe to borrow.
    let a = raw.s_addr;
    assert_eq!(a, u32::from_ne_bytes([127, 0, 0, 1]));
    assert_eq!(from_in_addr(raw), ip);

    let ip = Ipv4Addr::new(127, 34, 4, 12);
    let raw = to_in_addr(&ip);
    let a = raw.s_addr;
    assert_eq!(a, u32::from_ne_bytes([127, 34, 4, 12]));
    assert_eq!(from_in_addr(raw), ip);
}

#[test]
fn in6_addr_convertion() {
    let ip = Ipv6Addr::new(0x2000, 1, 2, 3, 4, 5, 6, 7);
    let raw = to_in6_addr(&ip);
    let want = [32, 0, 0, 1, 0, 2, 0, 3, 0, 4, 0, 5, 0, 6, 0, 7];
    assert_eq!(raw.s6_addr, want);
    assert_eq!(from_in6_addr(raw), ip);
}
