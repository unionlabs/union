//! The BSD sockets API requires us to read the `ss_family` field before
//! we can interpret the rest of a `sockaddr` produced by the kernel.
#![allow(unsafe_code)]

use super::super::c;
use crate::net::{SocketAddrAny, SocketAddrStorage, SocketAddrUnix, SocketAddrV4, SocketAddrV6};
use core::mem::size_of;

pub(crate) unsafe fn write_sockaddr(
    addr: &SocketAddrAny,
    storage: *mut SocketAddrStorage,
) -> usize {
    match addr {
        SocketAddrAny::V4(v4) => write_sockaddr_v4(v4, storage),
        SocketAddrAny::V6(v6) => write_sockaddr_v6(v6, storage),
        SocketAddrAny::Unix(unix) => write_sockaddr_unix(unix, storage),
    }
}

pub(crate) unsafe fn encode_sockaddr_v4(v4: &SocketAddrV4) -> c::sockaddr_in {
    c::sockaddr_in {
        sin_family: c::AF_INET as _,
        sin_port: u16::to_be(v4.port()),
        sin_addr: c::in_addr {
            s_addr: u32::from_ne_bytes(v4.ip().octets()),
        },
        __pad: [0_u8; 8],
    }
}

unsafe fn write_sockaddr_v4(v4: &SocketAddrV4, storage: *mut SocketAddrStorage) -> usize {
    let encoded = encode_sockaddr_v4(v4);
    core::ptr::write(storage.cast(), encoded);
    size_of::<c::sockaddr_in>()
}

pub(crate) unsafe fn encode_sockaddr_v6(v6: &SocketAddrV6) -> c::sockaddr_in6 {
    c::sockaddr_in6 {
        sin6_family: c::AF_INET6 as _,
        sin6_port: u16::to_be(v6.port()),
        sin6_flowinfo: u32::to_be(v6.flowinfo()),
        sin6_addr: c::in6_addr {
            in6_u: linux_raw_sys::general::in6_addr__bindgen_ty_1 {
                u6_addr8: v6.ip().octets(),
            },
        },
        sin6_scope_id: v6.scope_id(),
    }
}

unsafe fn write_sockaddr_v6(v6: &SocketAddrV6, storage: *mut SocketAddrStorage) -> usize {
    let encoded = encode_sockaddr_v6(v6);
    core::ptr::write(storage.cast(), encoded);
    size_of::<c::sockaddr_in6>()
}

unsafe fn write_sockaddr_unix(unix: &SocketAddrUnix, storage: *mut SocketAddrStorage) -> usize {
    core::ptr::write(storage.cast(), unix.unix);
    unix.len()
}
