//! libc syscalls supporting `rustix::net`.

use super::super::c;
use super::super::conv::{borrowed_fd, ret, ret_owned_fd, ret_send_recv, send_recv_len};
#[cfg(unix)]
use super::addr::SocketAddrUnix;
use super::ext::{in6_addr_new, in_addr_new};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io;
use crate::net::{SocketAddrAny, SocketAddrV4, SocketAddrV6};
use crate::utils::as_ptr;
use core::convert::TryInto;
use core::mem::{size_of, MaybeUninit};
#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
use {
    super::msghdr::{with_noaddr_msghdr, with_recv_msghdr, with_v4_msghdr, with_v6_msghdr},
    crate::io::{IoSlice, IoSliceMut},
    crate::net::{RecvAncillaryBuffer, RecvMsgReturn, SendAncillaryBuffer},
};
#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
use {
    super::read_sockaddr::{initialize_family_to_unspec, maybe_read_sockaddr_os, read_sockaddr_os},
    super::send_recv::{RecvFlags, SendFlags},
    super::types::{AddressFamily, Protocol, Shutdown, SocketFlags, SocketType},
    super::write_sockaddr::{encode_sockaddr_v4, encode_sockaddr_v6},
    core::ptr::null_mut,
};

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn recv(fd: BorrowedFd<'_>, buf: &mut [u8], flags: RecvFlags) -> io::Result<usize> {
    unsafe {
        ret_send_recv(c::recv(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn send(fd: BorrowedFd<'_>, buf: &[u8], flags: SendFlags) -> io::Result<usize> {
    unsafe {
        ret_send_recv(c::send(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn recvfrom(
    fd: BorrowedFd<'_>,
    buf: &mut [u8],
    flags: RecvFlags,
) -> io::Result<(usize, Option<SocketAddrAny>)> {
    unsafe {
        let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();
        let mut len = size_of::<c::sockaddr_storage>() as c::socklen_t;

        // `recvfrom` does not write to the storage if the socket is
        // connection-oriented sockets, so we initialize the family field to
        // `AF_UNSPEC` so that we can detect this case.
        initialize_family_to_unspec(storage.as_mut_ptr());

        ret_send_recv(c::recvfrom(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
            storage.as_mut_ptr().cast(),
            &mut len,
        ))
        .map(|nread| {
            (
                nread,
                maybe_read_sockaddr_os(storage.as_ptr(), len.try_into().unwrap()),
            )
        })
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendto_v4(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV4,
) -> io::Result<usize> {
    unsafe {
        ret_send_recv(c::sendto(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
            as_ptr(&encode_sockaddr_v4(addr)).cast::<c::sockaddr>(),
            size_of::<SocketAddrV4>() as _,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendto_v6(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV6,
) -> io::Result<usize> {
    unsafe {
        ret_send_recv(c::sendto(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
            as_ptr(&encode_sockaddr_v6(addr)).cast::<c::sockaddr>(),
            size_of::<SocketAddrV6>() as _,
        ))
    }
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendto_unix(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrUnix,
) -> io::Result<usize> {
    unsafe {
        ret_send_recv(c::sendto(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            send_recv_len(buf.len()),
            flags.bits(),
            as_ptr(&addr.unix).cast(),
            addr.addr_len(),
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn socket(
    domain: AddressFamily,
    type_: SocketType,
    protocol: Protocol,
) -> io::Result<OwnedFd> {
    unsafe {
        ret_owned_fd(c::socket(
            domain.0 as c::c_int,
            type_.0 as c::c_int,
            protocol.0,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn socket_with(
    domain: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<OwnedFd> {
    unsafe {
        ret_owned_fd(c::socket(
            domain.0 as c::c_int,
            type_.0 as c::c_int | flags.bits(),
            protocol.0,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn bind_v4(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    unsafe {
        ret(c::bind(
            borrowed_fd(sockfd),
            as_ptr(&encode_sockaddr_v4(addr)).cast(),
            size_of::<c::sockaddr_in>() as c::socklen_t,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn bind_v6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    unsafe {
        ret(c::bind(
            borrowed_fd(sockfd),
            as_ptr(&encode_sockaddr_v6(addr)).cast(),
            size_of::<c::sockaddr_in6>() as c::socklen_t,
        ))
    }
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn bind_unix(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    unsafe {
        ret(c::bind(
            borrowed_fd(sockfd),
            as_ptr(&addr.unix).cast(),
            addr.addr_len(),
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn connect_v4(sockfd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    unsafe {
        ret(c::connect(
            borrowed_fd(sockfd),
            as_ptr(&encode_sockaddr_v4(addr)).cast(),
            size_of::<c::sockaddr_in>() as c::socklen_t,
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn connect_v6(sockfd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    unsafe {
        ret(c::connect(
            borrowed_fd(sockfd),
            as_ptr(&encode_sockaddr_v6(addr)).cast(),
            size_of::<c::sockaddr_in6>() as c::socklen_t,
        ))
    }
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn connect_unix(sockfd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    unsafe {
        ret(c::connect(
            borrowed_fd(sockfd),
            as_ptr(&addr.unix).cast(),
            addr.addr_len(),
        ))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn listen(sockfd: BorrowedFd<'_>, backlog: c::c_int) -> io::Result<()> {
    unsafe { ret(c::listen(borrowed_fd(sockfd), backlog)) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn accept(sockfd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    unsafe {
        let owned_fd = ret_owned_fd(c::accept(borrowed_fd(sockfd), null_mut(), null_mut()))?;
        Ok(owned_fd)
    }
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn recvmsg(
    sockfd: BorrowedFd<'_>,
    iov: &mut [IoSliceMut<'_>],
    control: &mut RecvAncillaryBuffer<'_>,
    msg_flags: RecvFlags,
) -> io::Result<RecvMsgReturn> {
    let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();

    with_recv_msghdr(&mut storage, iov, control, |msghdr| {
        let result =
            unsafe { ret_send_recv(c::recvmsg(borrowed_fd(sockfd), msghdr, msg_flags.bits())) };

        result.map(|bytes| {
            // Get the address of the sender, if any.
            let addr =
                unsafe { maybe_read_sockaddr_os(msghdr.msg_name as _, msghdr.msg_namelen as _) };

            RecvMsgReturn {
                bytes,
                address: addr,
                flags: RecvFlags::from_bits_truncate(msghdr.msg_flags),
            }
        })
    })
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendmsg_noaddr(
    sockfd: BorrowedFd<'_>,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_noaddr_msghdr(iov, control, |msghdr| unsafe {
        ret_send_recv(c::sendmsg(borrowed_fd(sockfd), &msghdr, msg_flags.bits()))
    })
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendmsg_v4(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrV4,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_v4_msghdr(addr, iov, control, |msghdr| unsafe {
        ret_send_recv(c::sendmsg(borrowed_fd(sockfd), &msghdr, msg_flags.bits()))
    })
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn sendmsg_v6(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrV6,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_v6_msghdr(addr, iov, control, |msghdr| unsafe {
        ret_send_recv(c::sendmsg(borrowed_fd(sockfd), &msghdr, msg_flags.bits()))
    })
}

#[cfg(all(unix, not(target_os = "redox")))]
pub(crate) fn sendmsg_unix(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrUnix,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    super::msghdr::with_unix_msghdr(addr, iov, control, |msghdr| unsafe {
        ret_send_recv(c::sendmsg(borrowed_fd(sockfd), &msghdr, msg_flags.bits()))
    })
}

#[cfg(not(any(
    apple,
    windows,
    target_os = "haiku",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(crate) fn accept_with(sockfd: BorrowedFd<'_>, flags: SocketFlags) -> io::Result<OwnedFd> {
    unsafe {
        let owned_fd = ret_owned_fd(c::accept4(
            borrowed_fd(sockfd),
            null_mut(),
            null_mut(),
            flags.bits(),
        ))?;
        Ok(owned_fd)
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn acceptfrom(sockfd: BorrowedFd<'_>) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    unsafe {
        let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();
        let mut len = size_of::<c::sockaddr_storage>() as c::socklen_t;
        let owned_fd = ret_owned_fd(c::accept(
            borrowed_fd(sockfd),
            storage.as_mut_ptr().cast(),
            &mut len,
        ))?;
        Ok((
            owned_fd,
            maybe_read_sockaddr_os(storage.as_ptr(), len.try_into().unwrap()),
        ))
    }
}

#[cfg(not(any(
    apple,
    windows,
    target_os = "haiku",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(crate) fn acceptfrom_with(
    sockfd: BorrowedFd<'_>,
    flags: SocketFlags,
) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    unsafe {
        let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();
        let mut len = size_of::<c::sockaddr_storage>() as c::socklen_t;
        let owned_fd = ret_owned_fd(c::accept4(
            borrowed_fd(sockfd),
            storage.as_mut_ptr().cast(),
            &mut len,
            flags.bits(),
        ))?;
        Ok((
            owned_fd,
            maybe_read_sockaddr_os(storage.as_ptr(), len.try_into().unwrap()),
        ))
    }
}

/// Darwin lacks `accept4`, but does have `accept`. We define
/// `SocketFlags` to have no flags, so we can discard it here.
#[cfg(any(apple, windows, target_os = "haiku"))]
pub(crate) fn accept_with(sockfd: BorrowedFd<'_>, _flags: SocketFlags) -> io::Result<OwnedFd> {
    accept(sockfd)
}

/// Darwin lacks `accept4`, but does have `accept`. We define
/// `SocketFlags` to have no flags, so we can discard it here.
#[cfg(any(apple, windows, target_os = "haiku"))]
pub(crate) fn acceptfrom_with(
    sockfd: BorrowedFd<'_>,
    _flags: SocketFlags,
) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    acceptfrom(sockfd)
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn shutdown(sockfd: BorrowedFd<'_>, how: Shutdown) -> io::Result<()> {
    unsafe { ret(c::shutdown(borrowed_fd(sockfd), how as c::c_int)) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn getsockname(sockfd: BorrowedFd<'_>) -> io::Result<SocketAddrAny> {
    unsafe {
        let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();
        let mut len = size_of::<c::sockaddr_storage>() as c::socklen_t;
        ret(c::getsockname(
            borrowed_fd(sockfd),
            storage.as_mut_ptr().cast(),
            &mut len,
        ))?;
        Ok(read_sockaddr_os(storage.as_ptr(), len.try_into().unwrap()))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn getpeername(sockfd: BorrowedFd<'_>) -> io::Result<Option<SocketAddrAny>> {
    unsafe {
        let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();
        let mut len = size_of::<c::sockaddr_storage>() as c::socklen_t;
        ret(c::getpeername(
            borrowed_fd(sockfd),
            storage.as_mut_ptr().cast(),
            &mut len,
        ))?;
        Ok(maybe_read_sockaddr_os(
            storage.as_ptr(),
            len.try_into().unwrap(),
        ))
    }
}

#[cfg(not(any(windows, target_os = "redox", target_os = "wasi")))]
pub(crate) fn socketpair(
    domain: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<(OwnedFd, OwnedFd)> {
    unsafe {
        let mut fds = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(c::socketpair(
            c::c_int::from(domain.0),
            type_.0 as c::c_int | flags.bits(),
            protocol.0,
            fds.as_mut_ptr().cast::<c::c_int>(),
        ))?;

        let [fd0, fd1] = fds.assume_init();
        Ok((fd0, fd1))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) mod sockopt {
    use super::{c, in6_addr_new, in_addr_new, BorrowedFd};
    use crate::io;
    use crate::net::sockopt::Timeout;
    use crate::net::{Ipv4Addr, Ipv6Addr, SocketType};
    use crate::utils::as_mut_ptr;
    use core::convert::TryInto;
    use core::time::Duration;
    #[cfg(windows)]
    use windows_sys::Win32::Foundation::BOOL;

    // TODO: With Rust 1.53 we can use `Duration::ZERO` instead.
    const DURATION_ZERO: Duration = Duration::from_secs(0);

    #[inline]
    fn getsockopt<T: Copy>(fd: BorrowedFd<'_>, level: i32, optname: i32) -> io::Result<T> {
        use super::*;

        let mut optlen = core::mem::size_of::<T>().try_into().unwrap();
        debug_assert!(
            optlen as usize >= core::mem::size_of::<c::c_int>(),
            "Socket APIs don't ever use `bool` directly"
        );

        unsafe {
            let mut value = core::mem::zeroed::<T>();
            ret(c::getsockopt(
                borrowed_fd(fd),
                level,
                optname,
                as_mut_ptr(&mut value).cast(),
                &mut optlen,
            ))?;
            // On Windows at least, `getsockopt` has been observed writing 1
            // byte on at least (`IPPROTO_TCP`, `TCP_NODELAY`), even though
            // Windows' documentation says that should write a 4-byte `BOOL`.
            // So, we initialize the memory to zeros above, and just assert
            // that `getsockopt` doesn't write too many bytes here.
            assert!(
                optlen as usize <= size_of::<T>(),
                "unexpected getsockopt size"
            );
            Ok(value)
        }
    }

    #[inline]
    fn setsockopt<T: Copy>(
        fd: BorrowedFd<'_>,
        level: i32,
        optname: i32,
        value: T,
    ) -> io::Result<()> {
        use super::*;

        let optlen = core::mem::size_of::<T>().try_into().unwrap();
        debug_assert!(
            optlen as usize >= core::mem::size_of::<c::c_int>(),
            "Socket APIs don't ever use `bool` directly"
        );

        unsafe {
            ret(c::setsockopt(
                borrowed_fd(fd),
                level,
                optname,
                as_ptr(&value).cast(),
                optlen,
            ))
        }
    }

    #[inline]
    pub(crate) fn get_socket_type(fd: BorrowedFd<'_>) -> io::Result<SocketType> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_TYPE)
    }

    #[inline]
    pub(crate) fn set_socket_reuseaddr(fd: BorrowedFd<'_>, reuseaddr: bool) -> io::Result<()> {
        setsockopt(
            fd,
            c::SOL_SOCKET as _,
            c::SO_REUSEADDR,
            from_bool(reuseaddr),
        )
    }

    #[inline]
    pub(crate) fn set_socket_broadcast(fd: BorrowedFd<'_>, broadcast: bool) -> io::Result<()> {
        setsockopt(
            fd,
            c::SOL_SOCKET as _,
            c::SO_BROADCAST,
            from_bool(broadcast),
        )
    }

    #[inline]
    pub(crate) fn get_socket_broadcast(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_BROADCAST).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_socket_linger(
        fd: BorrowedFd<'_>,
        linger: Option<Duration>,
    ) -> io::Result<()> {
        // Convert `linger` to seconds, rounding up.
        let l_linger = if let Some(linger) = linger {
            let mut l_linger = linger.as_secs();
            if linger.subsec_nanos() != 0 {
                l_linger = l_linger.checked_add(1).ok_or(io::Errno::INVAL)?;
            }
            l_linger.try_into().map_err(|_e| io::Errno::INVAL)?
        } else {
            0
        };
        let linger = c::linger {
            l_onoff: linger.is_some() as _,
            l_linger,
        };
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_LINGER, linger)
    }

    #[inline]
    pub(crate) fn get_socket_linger(fd: BorrowedFd<'_>) -> io::Result<Option<Duration>> {
        let linger: c::linger = getsockopt(fd, c::SOL_SOCKET as _, c::SO_LINGER)?;
        // TODO: With Rust 1.50, this could use `.then`.
        Ok(if linger.l_onoff != 0 {
            Some(Duration::from_secs(linger.l_linger as u64))
        } else {
            None
        })
    }

    #[cfg(linux_kernel)]
    #[inline]
    pub(crate) fn set_socket_passcred(fd: BorrowedFd<'_>, passcred: bool) -> io::Result<()> {
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_PASSCRED, from_bool(passcred))
    }

    #[cfg(linux_kernel)]
    #[inline]
    pub(crate) fn get_socket_passcred(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_PASSCRED).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_socket_timeout(
        fd: BorrowedFd<'_>,
        id: Timeout,
        timeout: Option<Duration>,
    ) -> io::Result<()> {
        let optname = match id {
            Timeout::Recv => c::SO_RCVTIMEO,
            Timeout::Send => c::SO_SNDTIMEO,
        };

        #[cfg(not(windows))]
        let timeout = match timeout {
            Some(timeout) => {
                if timeout == DURATION_ZERO {
                    return Err(io::Errno::INVAL);
                }

                // Rust's musl libc bindings deprecated `time_t` while they
                // transition to 64-bit `time_t`. What we want here is just
                // “whatever type `timeval`'s `tv_sec` is”, so we're ok using
                // the deprecated type.
                #[allow(deprecated)]
                let tv_sec = timeout.as_secs().try_into().unwrap_or(c::time_t::MAX);

                // `subsec_micros` rounds down, so we use `subsec_nanos` and
                // manually round up.
                let mut timeout = c::timeval {
                    tv_sec,
                    tv_usec: ((timeout.subsec_nanos() + 999) / 1000) as _,
                };
                if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                    timeout.tv_usec = 1;
                }
                timeout
            }
            None => c::timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        };

        #[cfg(windows)]
        let timeout: u32 = match timeout {
            Some(timeout) => {
                if timeout == DURATION_ZERO {
                    return Err(io::Errno::INVAL);
                }

                // `as_millis` rounds down, so we use `as_nanos` and
                // manually round up.
                let mut timeout: u32 = ((timeout.as_nanos() + 999_999) / 1_000_000)
                    .try_into()
                    .map_err(|_convert_err| io::Errno::INVAL)?;
                if timeout == 0 {
                    timeout = 1;
                }
                timeout
            }
            None => 0,
        };

        setsockopt(fd, c::SOL_SOCKET, optname, timeout)
    }

    #[inline]
    pub(crate) fn get_socket_timeout(
        fd: BorrowedFd<'_>,
        id: Timeout,
    ) -> io::Result<Option<Duration>> {
        let optname = match id {
            Timeout::Recv => c::SO_RCVTIMEO,
            Timeout::Send => c::SO_SNDTIMEO,
        };

        #[cfg(not(windows))]
        {
            let timeout: c::timeval = getsockopt(fd, c::SOL_SOCKET, optname)?;
            if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                Ok(None)
            } else {
                Ok(Some(
                    Duration::from_secs(timeout.tv_sec as u64)
                        + Duration::from_micros(timeout.tv_usec as u64),
                ))
            }
        }

        #[cfg(windows)]
        {
            let timeout: u32 = getsockopt(fd, c::SOL_SOCKET, optname)?;
            if timeout == 0 {
                Ok(None)
            } else {
                Ok(Some(Duration::from_millis(timeout as u64)))
            }
        }
    }

    #[cfg(any(apple, target_os = "freebsd"))]
    #[inline]
    pub(crate) fn getsockopt_nosigpipe(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::SOL_SOCKET, c::SO_NOSIGPIPE).map(to_bool)
    }

    #[cfg(any(apple, target_os = "freebsd"))]
    #[inline]
    pub(crate) fn setsockopt_nosigpipe(fd: BorrowedFd<'_>, val: bool) -> io::Result<()> {
        setsockopt(fd, c::SOL_SOCKET, c::SO_NOSIGPIPE, from_bool(val))
    }

    #[inline]
    pub(crate) fn get_socket_error(fd: BorrowedFd<'_>) -> io::Result<Result<(), io::Errno>> {
        let err: c::c_int = getsockopt(fd, c::SOL_SOCKET as _, c::SO_ERROR)?;
        Ok(if err == 0 {
            Ok(())
        } else {
            Err(io::Errno::from_raw_os_error(err))
        })
    }

    #[inline]
    pub(crate) fn set_socket_keepalive(fd: BorrowedFd<'_>, keepalive: bool) -> io::Result<()> {
        setsockopt(
            fd,
            c::SOL_SOCKET as _,
            c::SO_KEEPALIVE,
            from_bool(keepalive),
        )
    }

    #[inline]
    pub(crate) fn get_socket_keepalive(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_KEEPALIVE).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_socket_recv_buffer_size(fd: BorrowedFd<'_>, size: usize) -> io::Result<()> {
        let size: c::c_int = size.try_into().map_err(|_| io::Errno::INVAL)?;
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_RCVBUF, size)
    }

    #[inline]
    pub(crate) fn get_socket_recv_buffer_size(fd: BorrowedFd<'_>) -> io::Result<usize> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_RCVBUF).map(|size: u32| size as usize)
    }

    #[inline]
    pub(crate) fn set_socket_send_buffer_size(fd: BorrowedFd<'_>, size: usize) -> io::Result<()> {
        let size: c::c_int = size.try_into().map_err(|_| io::Errno::INVAL)?;
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_SNDBUF, size)
    }

    #[inline]
    pub(crate) fn get_socket_send_buffer_size(fd: BorrowedFd<'_>) -> io::Result<usize> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_SNDBUF).map(|size: u32| size as usize)
    }

    #[inline]
    pub(crate) fn set_ip_ttl(fd: BorrowedFd<'_>, ttl: u32) -> io::Result<()> {
        setsockopt(fd, c::IPPROTO_IP as _, c::IP_TTL, ttl)
    }

    #[inline]
    pub(crate) fn get_ip_ttl(fd: BorrowedFd<'_>) -> io::Result<u32> {
        getsockopt(fd, c::IPPROTO_IP as _, c::IP_TTL)
    }

    #[inline]
    pub(crate) fn set_ipv6_v6only(fd: BorrowedFd<'_>, only_v6: bool) -> io::Result<()> {
        setsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_V6ONLY, from_bool(only_v6))
    }

    #[inline]
    pub(crate) fn get_ipv6_v6only(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_V6ONLY).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_ip_multicast_loop(
        fd: BorrowedFd<'_>,
        multicast_loop: bool,
    ) -> io::Result<()> {
        setsockopt(
            fd,
            c::IPPROTO_IP as _,
            c::IP_MULTICAST_LOOP,
            from_bool(multicast_loop),
        )
    }

    #[inline]
    pub(crate) fn get_ip_multicast_loop(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::IPPROTO_IP as _, c::IP_MULTICAST_LOOP).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_ip_multicast_ttl(fd: BorrowedFd<'_>, multicast_ttl: u32) -> io::Result<()> {
        setsockopt(fd, c::IPPROTO_IP as _, c::IP_MULTICAST_TTL, multicast_ttl)
    }

    #[inline]
    pub(crate) fn get_ip_multicast_ttl(fd: BorrowedFd<'_>) -> io::Result<u32> {
        getsockopt(fd, c::IPPROTO_IP as _, c::IP_MULTICAST_TTL)
    }

    #[inline]
    pub(crate) fn set_ipv6_multicast_loop(
        fd: BorrowedFd<'_>,
        multicast_loop: bool,
    ) -> io::Result<()> {
        setsockopt(
            fd,
            c::IPPROTO_IPV6 as _,
            c::IPV6_MULTICAST_LOOP,
            from_bool(multicast_loop),
        )
    }

    #[inline]
    pub(crate) fn get_ipv6_multicast_loop(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_MULTICAST_LOOP).map(to_bool)
    }

    #[inline]
    pub(crate) fn set_ipv6_multicast_hops(
        fd: BorrowedFd<'_>,
        multicast_hops: u32,
    ) -> io::Result<()> {
        setsockopt(
            fd,
            c::IPPROTO_IP as _,
            c::IPV6_MULTICAST_HOPS,
            multicast_hops,
        )
    }

    #[inline]
    pub(crate) fn get_ipv6_multicast_hops(fd: BorrowedFd<'_>) -> io::Result<u32> {
        getsockopt(fd, c::IPPROTO_IP as _, c::IPV6_MULTICAST_HOPS)
    }

    #[inline]
    pub(crate) fn set_ip_add_membership(
        fd: BorrowedFd<'_>,
        multiaddr: &Ipv4Addr,
        interface: &Ipv4Addr,
    ) -> io::Result<()> {
        let mreq = to_imr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IP as _, c::IP_ADD_MEMBERSHIP, mreq)
    }

    #[inline]
    pub(crate) fn set_ipv6_add_membership(
        fd: BorrowedFd<'_>,
        multiaddr: &Ipv6Addr,
        interface: u32,
    ) -> io::Result<()> {
        #[cfg(not(any(bsd, solarish, target_os = "haiku", target_os = "l4re")))]
        use c::IPV6_ADD_MEMBERSHIP;
        #[cfg(any(bsd, solarish, target_os = "haiku", target_os = "l4re"))]
        use c::IPV6_JOIN_GROUP as IPV6_ADD_MEMBERSHIP;

        let mreq = to_ipv6mr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IPV6 as _, IPV6_ADD_MEMBERSHIP, mreq)
    }

    #[inline]
    pub(crate) fn set_ip_drop_membership(
        fd: BorrowedFd<'_>,
        multiaddr: &Ipv4Addr,
        interface: &Ipv4Addr,
    ) -> io::Result<()> {
        let mreq = to_imr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IP as _, c::IP_DROP_MEMBERSHIP, mreq)
    }

    #[inline]
    pub(crate) fn set_ipv6_drop_membership(
        fd: BorrowedFd<'_>,
        multiaddr: &Ipv6Addr,
        interface: u32,
    ) -> io::Result<()> {
        #[cfg(not(any(bsd, solarish, target_os = "haiku", target_os = "l4re")))]
        use c::IPV6_DROP_MEMBERSHIP;
        #[cfg(any(bsd, solarish, target_os = "haiku", target_os = "l4re"))]
        use c::IPV6_LEAVE_GROUP as IPV6_DROP_MEMBERSHIP;

        let mreq = to_ipv6mr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IPV6 as _, IPV6_DROP_MEMBERSHIP, mreq)
    }

    #[inline]
    pub(crate) fn get_ipv6_unicast_hops(fd: BorrowedFd<'_>) -> io::Result<u8> {
        getsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_UNICAST_HOPS).map(|hops: c::c_int| hops as u8)
    }

    #[inline]
    pub(crate) fn set_ipv6_unicast_hops(fd: BorrowedFd<'_>, hops: Option<u8>) -> io::Result<()> {
        let hops = match hops {
            Some(hops) => hops as c::c_int,
            None => -1,
        };
        setsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_UNICAST_HOPS, hops)
    }

    #[inline]
    pub(crate) fn set_tcp_nodelay(fd: BorrowedFd<'_>, nodelay: bool) -> io::Result<()> {
        setsockopt(fd, c::IPPROTO_TCP as _, c::TCP_NODELAY, from_bool(nodelay))
    }

    #[inline]
    pub(crate) fn get_tcp_nodelay(fd: BorrowedFd<'_>) -> io::Result<bool> {
        getsockopt(fd, c::IPPROTO_TCP as _, c::TCP_NODELAY).map(to_bool)
    }

    #[inline]
    fn to_imr(multiaddr: &Ipv4Addr, interface: &Ipv4Addr) -> c::ip_mreq {
        c::ip_mreq {
            imr_multiaddr: to_imr_addr(multiaddr),
            imr_interface: to_imr_addr(interface),
        }
    }

    #[inline]
    fn to_imr_addr(addr: &Ipv4Addr) -> c::in_addr {
        in_addr_new(u32::from_ne_bytes(addr.octets()))
    }

    #[inline]
    fn to_ipv6mr(multiaddr: &Ipv6Addr, interface: u32) -> c::ipv6_mreq {
        c::ipv6_mreq {
            ipv6mr_multiaddr: to_ipv6mr_multiaddr(multiaddr),
            ipv6mr_interface: to_ipv6mr_interface(interface),
        }
    }

    #[inline]
    fn to_ipv6mr_multiaddr(multiaddr: &Ipv6Addr) -> c::in6_addr {
        in6_addr_new(multiaddr.octets())
    }

    #[cfg(target_os = "android")]
    #[inline]
    fn to_ipv6mr_interface(interface: u32) -> c::c_int {
        interface as c::c_int
    }

    #[cfg(not(target_os = "android"))]
    #[inline]
    fn to_ipv6mr_interface(interface: u32) -> c::c_uint {
        interface as c::c_uint
    }

    // `getsockopt` and `setsockopt` represent boolean values as integers.
    #[cfg(not(windows))]
    type RawSocketBool = c::c_int;
    #[cfg(windows)]
    type RawSocketBool = BOOL;

    // Wrap `RawSocketBool` in a newtype to discourage misuse.
    #[repr(transparent)]
    #[derive(Copy, Clone)]
    struct SocketBool(RawSocketBool);

    // Convert from a `bool` to a `SocketBool`.
    #[inline]
    fn from_bool(value: bool) -> SocketBool {
        SocketBool(value as _)
    }

    // Convert from a `SocketBool` to a `bool`.
    #[inline]
    fn to_bool(value: SocketBool) -> bool {
        value.0 != 0
    }
}
