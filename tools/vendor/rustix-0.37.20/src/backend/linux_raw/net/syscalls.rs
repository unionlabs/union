//! linux_raw syscalls supporting `rustix::net`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
use super::super::conv::{
    by_mut, by_ref, c_int, c_uint, ret, ret_owned_fd, ret_usize, size_of, slice, slice_mut,
    socklen_t, zero,
};
use super::msghdr::{
    with_noaddr_msghdr, with_recv_msghdr, with_unix_msghdr, with_v4_msghdr, with_v6_msghdr,
};
use super::read_sockaddr::{initialize_family_to_unspec, maybe_read_sockaddr_os, read_sockaddr_os};
use super::send_recv::{RecvFlags, SendFlags};
use super::types::{AddressFamily, Protocol, Shutdown, SocketFlags, SocketType};
use super::write_sockaddr::{encode_sockaddr_v4, encode_sockaddr_v6};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{
    RecvAncillaryBuffer, RecvMsgReturn, SendAncillaryBuffer, SocketAddrAny, SocketAddrUnix,
    SocketAddrV4, SocketAddrV6,
};
use c::{sockaddr, sockaddr_in, sockaddr_in6, socklen_t};
use core::convert::TryInto;
use core::mem::MaybeUninit;
#[cfg(target_arch = "x86")]
use {
    super::super::conv::{slice_just_addr, x86_sys},
    super::super::reg::{ArgReg, SocketArg},
    linux_raw_sys::general::{
        SYS_ACCEPT, SYS_ACCEPT4, SYS_BIND, SYS_CONNECT, SYS_GETPEERNAME, SYS_GETSOCKNAME,
        SYS_GETSOCKOPT, SYS_LISTEN, SYS_RECV, SYS_RECVFROM, SYS_RECVMSG, SYS_SEND, SYS_SENDMSG,
        SYS_SENDTO, SYS_SETSOCKOPT, SYS_SHUTDOWN, SYS_SOCKET, SYS_SOCKETPAIR,
    },
};

#[inline]
pub(crate) fn socket(
    family: AddressFamily,
    type_: SocketType,
    protocol: Protocol,
) -> io::Result<OwnedFd> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret_owned_fd(syscall_readonly!(__NR_socket, family, type_, protocol))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SOCKET),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                family.into(),
                type_.into(),
                protocol.into(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn socket_with(
    family: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<OwnedFd> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_socket,
            family,
            (type_, flags),
            protocol
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SOCKET),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                family.into(),
                (type_, flags).into(),
                protocol.into(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn socketpair(
    family: AddressFamily,
    type_: SocketType,
    flags: SocketFlags,
    protocol: Protocol,
) -> io::Result<(OwnedFd, OwnedFd)> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(syscall!(
            __NR_socketpair,
            family,
            (type_, flags),
            protocol,
            &mut result
        ))?;
        let [fd0, fd1] = result.assume_init();
        Ok((fd0, fd1))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(syscall!(
            __NR_socketcall,
            x86_sys(SYS_SOCKETPAIR),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                family.into(),
                (type_, flags).into(),
                protocol.into(),
                (&mut result).into(),
            ])
        ))?;
        let [fd0, fd1] = result.assume_init();
        Ok((fd0, fd1))
    }
}

#[inline]
pub(crate) fn accept(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let fd = ret_owned_fd(syscall_readonly!(__NR_accept, fd, zero(), zero()))?;
        Ok(fd)
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let fd = ret_owned_fd(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_ACCEPT),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[fd.into(), zero(), zero()])
        ))?;
        Ok(fd)
    }
}

#[inline]
pub(crate) fn accept_with(fd: BorrowedFd<'_>, flags: SocketFlags) -> io::Result<OwnedFd> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let fd = ret_owned_fd(syscall_readonly!(__NR_accept4, fd, zero(), zero(), flags))?;
        Ok(fd)
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let fd = ret_owned_fd(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_ACCEPT4),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[fd.into(), zero(), zero(), flags.into()])
        ))?;
        Ok(fd)
    }
}

#[inline]
pub(crate) fn acceptfrom(fd: BorrowedFd<'_>) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        let fd = ret_owned_fd(syscall!(
            __NR_accept,
            fd,
            &mut storage,
            by_mut(&mut addrlen)
        ))?;
        Ok((
            fd,
            maybe_read_sockaddr_os(&storage.assume_init(), addrlen.try_into().unwrap()),
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        let fd = ret_owned_fd(syscall!(
            __NR_socketcall,
            x86_sys(SYS_ACCEPT),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                (&mut storage).into(),
                by_mut(&mut addrlen),
            ])
        ))?;
        Ok((
            fd,
            maybe_read_sockaddr_os(&storage.assume_init(), addrlen.try_into().unwrap()),
        ))
    }
}

#[inline]
pub(crate) fn acceptfrom_with(
    fd: BorrowedFd<'_>,
    flags: SocketFlags,
) -> io::Result<(OwnedFd, Option<SocketAddrAny>)> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        let fd = ret_owned_fd(syscall!(
            __NR_accept4,
            fd,
            &mut storage,
            by_mut(&mut addrlen),
            flags
        ))?;
        Ok((
            fd,
            maybe_read_sockaddr_os(&storage.assume_init(), addrlen.try_into().unwrap()),
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        let fd = ret_owned_fd(syscall!(
            __NR_socketcall,
            x86_sys(SYS_ACCEPT4),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                (&mut storage).into(),
                by_mut(&mut addrlen),
                flags.into(),
            ])
        ))?;
        Ok((
            fd,
            maybe_read_sockaddr_os(&storage.assume_init(), addrlen.try_into().unwrap()),
        ))
    }
}

#[inline]
pub(crate) fn recvmsg(
    sockfd: BorrowedFd<'_>,
    iov: &mut [IoSliceMut<'_>],
    control: &mut RecvAncillaryBuffer<'_>,
    msg_flags: RecvFlags,
) -> io::Result<RecvMsgReturn> {
    let mut storage = MaybeUninit::<c::sockaddr_storage>::uninit();

    with_recv_msghdr(&mut storage, iov, control, |msghdr| {
        #[cfg(not(target_arch = "x86"))]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_recvmsg,
                sockfd,
                by_mut(msghdr),
                c_uint(msg_flags.bits())
            ))
        };

        #[cfg(target_arch = "x86")]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_socketcall,
                x86_sys(SYS_RECVMSG),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    sockfd.into(),
                    by_mut(msghdr),
                    c_uint(msg_flags.bits()).into(),
                ])
            ))
        };

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

#[inline]
pub(crate) fn sendmsg_noaddr(
    sockfd: BorrowedFd<'_>,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_noaddr_msghdr(iov, control, |msghdr| {
        #[cfg(not(target_arch = "x86"))]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_sendmsg,
                sockfd,
                by_ref(&msghdr),
                c_uint(msg_flags.bits())
            ))
        };

        #[cfg(target_arch = "x86")]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_socketcall,
                x86_sys(SYS_SENDMSG),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    sockfd.into(),
                    by_ref(&msghdr),
                    c_uint(msg_flags.bits())
                ])
            ))
        };

        result
    })
}

#[inline]
pub(crate) fn sendmsg_v4(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrV4,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_v4_msghdr(addr, iov, control, |msghdr| {
        #[cfg(not(target_arch = "x86"))]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_sendmsg,
                sockfd,
                by_ref(&msghdr),
                c_uint(msg_flags.bits())
            ))
        };

        #[cfg(target_arch = "x86")]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_socketcall,
                x86_sys(SYS_SENDMSG),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    sockfd.into(),
                    by_ref(&msghdr),
                    c_uint(msg_flags.bits())
                ])
            ))
        };

        result
    })
}

#[inline]
pub(crate) fn sendmsg_v6(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrV6,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_v6_msghdr(addr, iov, control, |msghdr| {
        #[cfg(not(target_arch = "x86"))]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_sendmsg,
                sockfd,
                by_ref(&msghdr),
                c_uint(msg_flags.bits())
            ))
        };

        #[cfg(target_arch = "x86")]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_socketcall,
                x86_sys(SYS_SENDMSG),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    sockfd.into(),
                    by_ref(&msghdr),
                    c_uint(msg_flags.bits())
                ])
            ))
        };

        result
    })
}

#[inline]
pub(crate) fn sendmsg_unix(
    sockfd: BorrowedFd<'_>,
    addr: &SocketAddrUnix,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    msg_flags: SendFlags,
) -> io::Result<usize> {
    with_unix_msghdr(addr, iov, control, |msghdr| {
        #[cfg(not(target_arch = "x86"))]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_sendmsg,
                sockfd,
                by_ref(&msghdr),
                c_uint(msg_flags.bits())
            ))
        };

        #[cfg(target_arch = "x86")]
        let result = unsafe {
            ret_usize(syscall!(
                __NR_socketcall,
                x86_sys(SYS_SENDMSG),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    sockfd.into(),
                    by_ref(&msghdr),
                    c_uint(msg_flags.bits())
                ])
            ))
        };

        result
    })
}

#[inline]
pub(crate) fn shutdown(fd: BorrowedFd<'_>, how: Shutdown) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_shutdown,
            fd,
            c_uint(how as c::c_uint)
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SHUTDOWN),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[fd.into(), c_uint(how as c::c_uint)])
        ))
    }
}

#[inline]
pub(crate) fn send(fd: BorrowedFd<'_>, buf: &[u8], flags: SendFlags) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "mips64",
        target_arch = "riscv64",
        target_arch = "x86",
        target_arch = "x86_64",
    )))]
    unsafe {
        ret_usize(syscall_readonly!(__NR_send, fd, buf_addr, buf_len, flags))
    }
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "mips64",
        target_arch = "riscv64",
        target_arch = "x86_64",
    ))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_sendto,
            fd,
            buf_addr,
            buf_len,
            flags,
            zero(),
            zero()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SEND),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[fd.into(), buf_addr, buf_len, flags.into()])
        ))
    }
}

#[inline]
pub(crate) fn sendto_v4(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV4,
) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_sendto,
            fd,
            buf_addr,
            buf_len,
            flags,
            by_ref(&encode_sockaddr_v4(addr)),
            size_of::<sockaddr_in, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SENDTO),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                buf_addr,
                buf_len,
                flags.into(),
                by_ref(&encode_sockaddr_v4(addr)),
                size_of::<sockaddr_in, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn sendto_v6(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrV6,
) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_sendto,
            fd,
            buf_addr,
            buf_len,
            flags,
            by_ref(&encode_sockaddr_v6(addr)),
            size_of::<sockaddr_in6, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SENDTO),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                buf_addr,
                buf_len,
                flags.into(),
                by_ref(&encode_sockaddr_v6(addr)),
                size_of::<sockaddr_in6, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn sendto_unix(
    fd: BorrowedFd<'_>,
    buf: &[u8],
    flags: SendFlags,
    addr: &SocketAddrUnix,
) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_sendto,
            fd,
            buf_addr,
            buf_len,
            flags,
            by_ref(&addr.unix),
            socklen_t(addr.addr_len())
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_SENDTO),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                buf_addr,
                buf_len,
                flags.into(),
                by_ref(&addr.unix),
                socklen_t(addr.addr_len()),
            ])
        ))
    }
}

#[inline]
pub(crate) fn recv(fd: BorrowedFd<'_>, buf: &mut [u8], flags: RecvFlags) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);

    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "mips64",
        target_arch = "riscv64",
        target_arch = "x86",
        target_arch = "x86_64",
    )))]
    unsafe {
        ret_usize(syscall!(__NR_recv, fd, buf_addr_mut, buf_len, flags))
    }
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "mips64",
        target_arch = "riscv64",
        target_arch = "x86_64",
    ))]
    unsafe {
        ret_usize(syscall!(
            __NR_recvfrom,
            fd,
            buf_addr_mut,
            buf_len,
            flags,
            zero(),
            zero()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret_usize(syscall!(
            __NR_socketcall,
            x86_sys(SYS_RECV),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                buf_addr_mut,
                buf_len,
                flags.into(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn recvfrom(
    fd: BorrowedFd<'_>,
    buf: &mut [u8],
    flags: RecvFlags,
) -> io::Result<(usize, Option<SocketAddrAny>)> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);

    let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
    let mut storage = MaybeUninit::<sockaddr>::uninit();

    unsafe {
        // `recvfrom` does not write to the storage if the socket is
        // connection-oriented sockets, so we initialize the family field to
        // `AF_UNSPEC` so that we can detect this case.
        initialize_family_to_unspec(storage.as_mut_ptr());

        #[cfg(not(target_arch = "x86"))]
        let nread = ret_usize(syscall!(
            __NR_recvfrom,
            fd,
            buf_addr_mut,
            buf_len,
            flags,
            &mut storage,
            by_mut(&mut addrlen)
        ))?;
        #[cfg(target_arch = "x86")]
        let nread = ret_usize(syscall!(
            __NR_socketcall,
            x86_sys(SYS_RECVFROM),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                buf_addr_mut,
                buf_len,
                flags.into(),
                (&mut storage).into(),
                by_mut(&mut addrlen),
            ])
        ))?;

        Ok((
            nread,
            maybe_read_sockaddr_os(&storage.assume_init(), addrlen.try_into().unwrap()),
        ))
    }
}

#[inline]
pub(crate) fn getpeername(fd: BorrowedFd<'_>) -> io::Result<Option<SocketAddrAny>> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        ret(syscall!(
            __NR_getpeername,
            fd,
            &mut storage,
            by_mut(&mut addrlen)
        ))?;
        Ok(maybe_read_sockaddr_os(
            &storage.assume_init(),
            addrlen.try_into().unwrap(),
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        ret(syscall!(
            __NR_socketcall,
            x86_sys(SYS_GETPEERNAME),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                (&mut storage).into(),
                by_mut(&mut addrlen),
            ])
        ))?;
        Ok(maybe_read_sockaddr_os(
            &storage.assume_init(),
            addrlen.try_into().unwrap(),
        ))
    }
}

#[inline]
pub(crate) fn getsockname(fd: BorrowedFd<'_>) -> io::Result<SocketAddrAny> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        ret(syscall!(
            __NR_getsockname,
            fd,
            &mut storage,
            by_mut(&mut addrlen)
        ))?;
        Ok(read_sockaddr_os(
            &storage.assume_init(),
            addrlen.try_into().unwrap(),
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        let mut addrlen = core::mem::size_of::<sockaddr>() as socklen_t;
        let mut storage = MaybeUninit::<sockaddr>::uninit();
        ret(syscall!(
            __NR_socketcall,
            x86_sys(SYS_GETSOCKNAME),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                (&mut storage).into(),
                by_mut(&mut addrlen),
            ])
        ))?;
        Ok(read_sockaddr_os(
            &storage.assume_init(),
            addrlen.try_into().unwrap(),
        ))
    }
}

#[inline]
pub(crate) fn bind_v4(fd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_bind,
            fd,
            by_ref(&encode_sockaddr_v4(addr)),
            size_of::<sockaddr_in, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_BIND),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&encode_sockaddr_v4(addr)),
                size_of::<sockaddr_in, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn bind_v6(fd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_bind,
            fd,
            by_ref(&encode_sockaddr_v6(addr)),
            size_of::<sockaddr_in6, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_BIND),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&encode_sockaddr_v6(addr)),
                size_of::<sockaddr_in6, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn bind_unix(fd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_bind,
            fd,
            by_ref(&addr.unix),
            socklen_t(addr.addr_len())
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_BIND),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&addr.unix),
                socklen_t(addr.addr_len()),
            ])
        ))
    }
}

#[inline]
pub(crate) fn connect_v4(fd: BorrowedFd<'_>, addr: &SocketAddrV4) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_connect,
            fd,
            by_ref(&encode_sockaddr_v4(addr)),
            size_of::<sockaddr_in, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_CONNECT),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&encode_sockaddr_v4(addr)),
                size_of::<sockaddr_in, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn connect_v6(fd: BorrowedFd<'_>, addr: &SocketAddrV6) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_connect,
            fd,
            by_ref(&encode_sockaddr_v6(addr)),
            size_of::<sockaddr_in6, _>()
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_CONNECT),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&encode_sockaddr_v6(addr)),
                size_of::<sockaddr_in6, _>(),
            ])
        ))
    }
}

#[inline]
pub(crate) fn connect_unix(fd: BorrowedFd<'_>, addr: &SocketAddrUnix) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(
            __NR_connect,
            fd,
            by_ref(&addr.unix),
            socklen_t(addr.addr_len())
        ))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_CONNECT),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[
                fd.into(),
                by_ref(&addr.unix),
                socklen_t(addr.addr_len()),
            ])
        ))
    }
}

#[inline]
pub(crate) fn listen(fd: BorrowedFd<'_>, backlog: c::c_int) -> io::Result<()> {
    #[cfg(not(target_arch = "x86"))]
    unsafe {
        ret(syscall_readonly!(__NR_listen, fd, c_int(backlog)))
    }
    #[cfg(target_arch = "x86")]
    unsafe {
        ret(syscall_readonly!(
            __NR_socketcall,
            x86_sys(SYS_LISTEN),
            slice_just_addr::<ArgReg<SocketArg>, _>(&[fd.into(), c_int(backlog)])
        ))
    }
}

pub(crate) mod sockopt {
    use super::{c, BorrowedFd};
    use crate::io;
    use crate::net::sockopt::Timeout;
    use crate::net::{Ipv4Addr, Ipv6Addr, SocketType};
    use c::{SO_RCVTIMEO_NEW, SO_RCVTIMEO_OLD, SO_SNDTIMEO_NEW, SO_SNDTIMEO_OLD};
    use core::convert::TryInto;
    use core::time::Duration;
    use linux_raw_sys::general::{__kernel_timespec, timeval};

    // TODO: With Rust 1.53 we can use `Duration::ZERO` instead.
    const DURATION_ZERO: Duration = Duration::from_secs(0);

    #[inline]
    fn getsockopt<T: Copy>(fd: BorrowedFd<'_>, level: u32, optname: u32) -> io::Result<T> {
        use super::*;

        let mut optlen = core::mem::size_of::<T>();
        debug_assert!(
            optlen as usize >= core::mem::size_of::<c::c_int>(),
            "Socket APIs don't ever use `bool` directly"
        );

        #[cfg(not(target_arch = "x86"))]
        unsafe {
            let mut value = MaybeUninit::<T>::uninit();
            ret(syscall!(
                __NR_getsockopt,
                fd,
                c_uint(level),
                c_uint(optname),
                &mut value,
                by_mut(&mut optlen)
            ))?;

            assert_eq!(
                optlen as usize,
                core::mem::size_of::<T>(),
                "unexpected getsockopt size"
            );
            Ok(value.assume_init())
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            let mut value = MaybeUninit::<T>::uninit();
            ret(syscall!(
                __NR_socketcall,
                x86_sys(SYS_GETSOCKOPT),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    fd.into(),
                    c_uint(level),
                    c_uint(optname),
                    (&mut value).into(),
                    by_mut(&mut optlen),
                ])
            ))?;
            assert_eq!(
                optlen as usize,
                core::mem::size_of::<T>(),
                "unexpected getsockopt size"
            );
            Ok(value.assume_init())
        }
    }

    #[inline]
    fn setsockopt<T: Copy>(
        fd: BorrowedFd<'_>,
        level: u32,
        optname: u32,
        value: T,
    ) -> io::Result<()> {
        use super::*;

        let optlen = core::mem::size_of::<T>().try_into().unwrap();
        debug_assert!(
            optlen as usize >= core::mem::size_of::<c::c_int>(),
            "Socket APIs don't ever use `bool` directly"
        );

        #[cfg(not(target_arch = "x86"))]
        unsafe {
            ret(syscall_readonly!(
                __NR_setsockopt,
                fd,
                c_uint(level),
                c_uint(optname),
                by_ref(&value),
                socklen_t(optlen)
            ))
        }
        #[cfg(target_arch = "x86")]
        unsafe {
            ret(syscall_readonly!(
                __NR_socketcall,
                x86_sys(SYS_SETSOCKOPT),
                slice_just_addr::<ArgReg<SocketArg>, _>(&[
                    fd.into(),
                    c_uint(level),
                    c_uint(optname),
                    by_ref(&value),
                    socklen_t(optlen),
                ])
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
            l_onoff: c::c_int::from(linger.is_some()),
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

    #[inline]
    pub(crate) fn set_socket_passcred(fd: BorrowedFd<'_>, passcred: bool) -> io::Result<()> {
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_PASSCRED, from_bool(passcred))
    }

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
        let time = duration_to_linux(timeout)?;
        let optname = match id {
            Timeout::Recv => SO_RCVTIMEO_NEW,
            Timeout::Send => SO_SNDTIMEO_NEW,
        };
        match setsockopt(fd, c::SOL_SOCKET, optname, time) {
            Err(io::Errno::NOPROTOOPT) if SO_RCVTIMEO_NEW != SO_RCVTIMEO_OLD => {
                set_socket_timeout_old(fd, id, timeout)
            }
            otherwise => otherwise,
        }
    }

    /// Same as `set_socket_timeout` but uses `timeval` instead of
    /// `__kernel_timespec` and `_OLD` constants instead of `_NEW`.
    fn set_socket_timeout_old(
        fd: BorrowedFd<'_>,
        id: Timeout,
        timeout: Option<Duration>,
    ) -> io::Result<()> {
        let time = duration_to_linux_old(timeout)?;
        let optname = match id {
            Timeout::Recv => SO_RCVTIMEO_OLD,
            Timeout::Send => SO_SNDTIMEO_OLD,
        };
        setsockopt(fd, c::SOL_SOCKET, optname, time)
    }

    #[inline]
    pub(crate) fn get_socket_timeout(
        fd: BorrowedFd<'_>,
        id: Timeout,
    ) -> io::Result<Option<Duration>> {
        let optname = match id {
            Timeout::Recv => SO_RCVTIMEO_NEW,
            Timeout::Send => SO_SNDTIMEO_NEW,
        };
        let time: __kernel_timespec = match getsockopt(fd, c::SOL_SOCKET, optname) {
            Err(io::Errno::NOPROTOOPT) if SO_RCVTIMEO_NEW != SO_RCVTIMEO_OLD => {
                return get_socket_timeout_old(fd, id)
            }
            otherwise => otherwise?,
        };
        Ok(duration_from_linux(time))
    }

    /// Same as `get_socket_timeout` but uses `timeval` instead of
    /// `__kernel_timespec` and `_OLD` constants instead of `_NEW`.
    fn get_socket_timeout_old(fd: BorrowedFd<'_>, id: Timeout) -> io::Result<Option<Duration>> {
        let optname = match id {
            Timeout::Recv => SO_RCVTIMEO_OLD,
            Timeout::Send => SO_SNDTIMEO_OLD,
        };
        let time: timeval = getsockopt(fd, c::SOL_SOCKET, optname)?;
        Ok(duration_from_linux_old(time))
    }

    /// Convert a C `timespec` to a Rust `Option<Duration>`.
    #[inline]
    fn duration_from_linux(time: __kernel_timespec) -> Option<Duration> {
        if time.tv_sec == 0 && time.tv_nsec == 0 {
            None
        } else {
            Some(
                Duration::from_secs(time.tv_sec as u64) + Duration::from_nanos(time.tv_nsec as u64),
            )
        }
    }

    /// Like `duration_from_linux` but uses Linux's old 32-bit `timeval`.
    fn duration_from_linux_old(time: timeval) -> Option<Duration> {
        if time.tv_sec == 0 && time.tv_usec == 0 {
            None
        } else {
            Some(
                Duration::from_secs(time.tv_sec as u64)
                    + Duration::from_micros(time.tv_usec as u64),
            )
        }
    }

    /// Convert a Rust `Option<Duration>` to a C `timespec`.
    #[inline]
    fn duration_to_linux(timeout: Option<Duration>) -> io::Result<__kernel_timespec> {
        Ok(match timeout {
            Some(timeout) => {
                if timeout == DURATION_ZERO {
                    return Err(io::Errno::INVAL);
                }
                let mut timeout = __kernel_timespec {
                    tv_sec: timeout.as_secs().try_into().unwrap_or(i64::MAX),
                    tv_nsec: timeout.subsec_nanos().into(),
                };
                if timeout.tv_sec == 0 && timeout.tv_nsec == 0 {
                    timeout.tv_nsec = 1;
                }
                timeout
            }
            None => __kernel_timespec {
                tv_sec: 0,
                tv_nsec: 0,
            },
        })
    }

    /// Like `duration_to_linux` but uses Linux's old 32-bit `timeval`.
    fn duration_to_linux_old(timeout: Option<Duration>) -> io::Result<timeval> {
        Ok(match timeout {
            Some(timeout) => {
                if timeout == DURATION_ZERO {
                    return Err(io::Errno::INVAL);
                }

                // `subsec_micros` rounds down, so we use `subsec_nanos` and
                // manually round up.
                let mut timeout = timeval {
                    tv_sec: timeout.as_secs().try_into().unwrap_or(c::c_long::MAX),
                    tv_usec: ((timeout.subsec_nanos() + 999) / 1000) as _,
                };
                if timeout.tv_sec == 0 && timeout.tv_usec == 0 {
                    timeout.tv_usec = 1;
                }
                timeout
            }
            None => timeval {
                tv_sec: 0,
                tv_usec: 0,
            },
        })
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
        let size: c::c_int = size.try_into().map_err(|_| io::Errno::OVERFLOW)?;
        setsockopt(fd, c::SOL_SOCKET as _, c::SO_RCVBUF, size)
    }

    #[inline]
    pub(crate) fn get_socket_recv_buffer_size(fd: BorrowedFd<'_>) -> io::Result<usize> {
        getsockopt(fd, c::SOL_SOCKET as _, c::SO_RCVBUF).map(|size: u32| size as usize)
    }

    #[inline]
    pub(crate) fn set_socket_send_buffer_size(fd: BorrowedFd<'_>, size: usize) -> io::Result<()> {
        let size: c::c_int = size.try_into().map_err(|_| io::Errno::OVERFLOW)?;
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
        let mreq = to_ipv6mr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_ADD_MEMBERSHIP, mreq)
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
        let mreq = to_ipv6mr(multiaddr, interface);
        setsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_DROP_MEMBERSHIP, mreq)
    }

    #[inline]
    pub(crate) fn get_ipv6_unicast_hops(fd: BorrowedFd<'_>) -> io::Result<u8> {
        getsockopt(fd, c::IPPROTO_IPV6 as _, c::IPV6_UNICAST_HOPS).map(|hops: c::c_int| hops as u8)
    }

    #[inline]
    pub(crate) fn set_ipv6_unicast_hops(fd: BorrowedFd<'_>, hops: Option<u8>) -> io::Result<()> {
        let hops = match hops {
            Some(hops) => hops.into(),
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
        c::in_addr {
            s_addr: u32::from_ne_bytes(addr.octets()),
        }
    }

    #[inline]
    fn to_ipv6mr(multiaddr: &Ipv6Addr, interface: u32) -> c::ipv6_mreq {
        c::ipv6_mreq {
            ipv6mr_multiaddr: to_ipv6mr_multiaddr(multiaddr),
            ipv6mr_ifindex: to_ipv6mr_interface(interface),
        }
    }

    #[inline]
    fn to_ipv6mr_multiaddr(multiaddr: &Ipv6Addr) -> c::in6_addr {
        c::in6_addr {
            in6_u: linux_raw_sys::general::in6_addr__bindgen_ty_1 {
                u6_addr8: multiaddr.octets(),
            },
        }
    }

    #[inline]
    fn to_ipv6mr_interface(interface: u32) -> c::c_int {
        interface as c::c_int
    }

    #[inline]
    fn from_bool(value: bool) -> c::c_uint {
        c::c_uint::from(value)
    }

    #[inline]
    fn to_bool(value: c::c_uint) -> bool {
        value != 0
    }
}
