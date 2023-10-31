//! Utilities for dealing with message headers.
//!
//! These take closures rather than returning a `c::msghdr` directly because
//! the message headers may reference stack-local data.

use super::super::c;
use super::super::conv::{msg_control_len, msg_iov_len};
use super::super::net::write_sockaddr::{encode_sockaddr_v4, encode_sockaddr_v6};

use crate::io::{self, IoSlice, IoSliceMut};
use crate::net::{RecvAncillaryBuffer, SendAncillaryBuffer, SocketAddrV4, SocketAddrV6};
use crate::utils::as_ptr;

use core::convert::TryInto;
use core::mem::{size_of, zeroed, MaybeUninit};

/// Create a message header intended to receive a datagram.
pub(crate) fn with_recv_msghdr<R>(
    name: &mut MaybeUninit<c::sockaddr_storage>,
    iov: &mut [IoSliceMut<'_>],
    control: &mut RecvAncillaryBuffer<'_>,
    f: impl FnOnce(&mut c::msghdr) -> io::Result<R>,
) -> io::Result<R> {
    control.clear();

    let namelen = size_of::<c::sockaddr_storage>() as c::socklen_t;
    let mut msghdr = {
        let mut h: c::msghdr = unsafe { zeroed() };
        h.msg_name = name.as_mut_ptr().cast();
        h.msg_namelen = namelen;
        h.msg_iov = iov.as_mut_ptr().cast();
        h.msg_iovlen = msg_iov_len(iov.len());
        h.msg_control = control.as_control_ptr().cast();
        h.msg_controllen = msg_control_len(control.control_len());
        h
    };

    let res = f(&mut msghdr);

    // Reset the control length.
    if res.is_ok() {
        unsafe {
            control.set_control_len(msghdr.msg_controllen.try_into().unwrap_or(usize::MAX));
        }
    }

    res
}

/// Create a message header intended to send without an address.
pub(crate) fn with_noaddr_msghdr<R>(
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    f: impl FnOnce(c::msghdr) -> R,
) -> R {
    f({
        let mut h: c::msghdr = unsafe { zeroed() };
        h.msg_iov = iov.as_ptr() as _;
        h.msg_iovlen = msg_iov_len(iov.len());
        h.msg_control = control.as_control_ptr().cast();
        h.msg_controllen = msg_control_len(control.control_len());
        h
    })
}

/// Create a message header intended to send with an IPv4 address.
pub(crate) fn with_v4_msghdr<R>(
    addr: &SocketAddrV4,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    f: impl FnOnce(c::msghdr) -> R,
) -> R {
    let encoded = unsafe { encode_sockaddr_v4(addr) };

    f({
        let mut h: c::msghdr = unsafe { zeroed() };
        h.msg_name = as_ptr(&encoded) as _;
        h.msg_namelen = size_of::<SocketAddrV4>() as _;
        h.msg_iov = iov.as_ptr() as _;
        h.msg_iovlen = msg_iov_len(iov.len());
        h.msg_control = control.as_control_ptr().cast();
        h.msg_controllen = msg_control_len(control.control_len());
        h
    })
}

/// Create a message header intended to send with an IPv6 address.
pub(crate) fn with_v6_msghdr<R>(
    addr: &SocketAddrV6,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    f: impl FnOnce(c::msghdr) -> R,
) -> R {
    let encoded = unsafe { encode_sockaddr_v6(addr) };

    f({
        let mut h: c::msghdr = unsafe { zeroed() };
        h.msg_name = as_ptr(&encoded) as _;
        h.msg_namelen = size_of::<SocketAddrV6>() as _;
        h.msg_iov = iov.as_ptr() as _;
        h.msg_iovlen = msg_iov_len(iov.len());
        h.msg_control = control.as_control_ptr().cast();
        h.msg_controllen = msg_control_len(control.control_len());
        h
    })
}

/// Create a message header intended to send with a Unix address.
#[cfg(all(unix, not(target_os = "redox")))]
pub(crate) fn with_unix_msghdr<R>(
    addr: &crate::net::SocketAddrUnix,
    iov: &[IoSlice<'_>],
    control: &mut SendAncillaryBuffer<'_, '_, '_>,
    f: impl FnOnce(c::msghdr) -> R,
) -> R {
    f({
        let mut h: c::msghdr = unsafe { zeroed() };
        h.msg_name = as_ptr(addr) as _;
        h.msg_namelen = addr.addr_len();
        h.msg_iov = iov.as_ptr() as _;
        h.msg_iovlen = msg_iov_len(iov.len());
        h.msg_control = control.as_control_ptr().cast();
        h.msg_controllen = msg_control_len(control.control_len());
        h
    })
}
