#![allow(unsafe_code)]

use super::super::c;

#[inline]
pub(crate) const fn in_addr_s_addr(addr: c::in_addr) -> u32 {
    addr.s_addr
}

#[inline]
pub(crate) const fn in_addr_new(s_addr: u32) -> c::in_addr {
    c::in_addr { s_addr }
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) const fn in6_addr_s6_addr(addr: c::in6_addr) -> [u8; 16] {
    // Safety: `in6_addr` is `repr(C)` and contains plain integer data.
    unsafe { addr.in6_u.u6_addr8 }
}

// TODO: With Rust 1.55, we can use the above `in6_addr_s6_addr` definition
// that uses a const-fn union access instead of doing a transmute.
#[cfg(not(not(feature = "std")))]
#[inline]
pub(crate) fn in6_addr_s6_addr(addr: c::in6_addr) -> [u8; 16] {
    // Safety: `in6_addr` is `repr(C)` and contains plain integer data.
    unsafe { core::mem::transmute(addr) }
}

#[inline]
pub(crate) const fn in6_addr_new(s6_addr: [u8; 16]) -> c::in6_addr {
    c::in6_addr {
        in6_u: linux_raw_sys::general::in6_addr__bindgen_ty_1 { u6_addr8: s6_addr },
    }
}

#[inline]
pub(crate) const fn sockaddr_in6_sin6_scope_id(addr: c::sockaddr_in6) -> u32 {
    addr.sin6_scope_id
}

#[cfg(not(feature = "std"))]
#[inline]
pub(crate) fn sockaddr_in6_sin6_scope_id_mut(addr: &mut c::sockaddr_in6) -> &mut u32 {
    &mut addr.sin6_scope_id
}

#[inline]
pub(crate) const fn sockaddr_in6_new(
    sin6_family: c::sa_family_t,
    sin6_port: u16,
    sin6_flowinfo: u32,
    sin6_addr: c::in6_addr,
    sin6_scope_id: u32,
) -> c::sockaddr_in6 {
    c::sockaddr_in6 {
        sin6_family,
        sin6_port,
        sin6_flowinfo,
        sin6_addr,
        sin6_scope_id,
    }
}
