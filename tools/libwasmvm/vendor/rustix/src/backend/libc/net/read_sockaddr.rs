use super::super::c;
#[cfg(unix)]
use super::addr::SocketAddrUnix;
use super::ext::{in6_addr_s6_addr, in_addr_s_addr, sockaddr_in6_sin6_scope_id};
#[cfg(not(windows))]
use crate::ffi::CStr;
use crate::io;
use crate::net::{Ipv4Addr, Ipv6Addr, SocketAddrAny, SocketAddrV4, SocketAddrV6};
use core::mem::size_of;

// This must match the header of `sockaddr`.
#[repr(C)]
struct sockaddr_header {
    #[cfg(any(bsd, target_os = "haiku"))]
    sa_len: u8,
    #[cfg(any(bsd, target_os = "haiku"))]
    ss_family: u8,
    #[cfg(not(any(bsd, target_os = "haiku")))]
    ss_family: u16,
}

#[inline]
unsafe fn read_ss_family(storage: *const c::sockaddr_storage) -> u16 {
    // Assert that we know the layout of `sockaddr`.
    let _ = c::sockaddr {
        #[cfg(any(bsd, target_os = "haiku"))]
        sa_len: 0_u8,
        #[cfg(any(bsd, target_os = "haiku"))]
        sa_family: 0_u8,
        #[cfg(not(any(bsd, target_os = "haiku")))]
        sa_family: 0_u16,
        #[cfg(not(target_os = "haiku"))]
        sa_data: [0; 14],
        #[cfg(target_os = "haiku")]
        sa_data: [0; 30],
    };

    (*storage.cast::<sockaddr_header>()).ss_family.into()
}

/// Set the `ss_family` field of a socket address to `AF_UNSPEC`, so that we
/// can test for `AF_UNSPEC` to test whether it was stored to.
pub(crate) unsafe fn initialize_family_to_unspec(storage: *mut c::sockaddr_storage) {
    (*storage.cast::<sockaddr_header>()).ss_family = c::AF_UNSPEC as _;
}

pub(crate) unsafe fn read_sockaddr(
    storage: *const c::sockaddr_storage,
    len: usize,
) -> io::Result<SocketAddrAny> {
    #[cfg(unix)]
    let offsetof_sun_path = super::addr::offsetof_sun_path();

    if len < size_of::<c::sa_family_t>() {
        return Err(io::Errno::INVAL);
    }
    match read_ss_family(storage).into() {
        c::AF_INET => {
            if len < size_of::<c::sockaddr_in>() {
                return Err(io::Errno::INVAL);
            }
            let decode = &*storage.cast::<c::sockaddr_in>();
            Ok(SocketAddrAny::V4(SocketAddrV4::new(
                Ipv4Addr::from(u32::from_be(in_addr_s_addr(decode.sin_addr))),
                u16::from_be(decode.sin_port),
            )))
        }
        c::AF_INET6 => {
            if len < size_of::<c::sockaddr_in6>() {
                return Err(io::Errno::INVAL);
            }
            let decode = &*storage.cast::<c::sockaddr_in6>();
            #[cfg(not(windows))]
            let s6_addr = decode.sin6_addr.s6_addr;
            #[cfg(windows)]
            let s6_addr = decode.sin6_addr.u.Byte;
            #[cfg(not(windows))]
            let sin6_scope_id = decode.sin6_scope_id;
            #[cfg(windows)]
            let sin6_scope_id = decode.Anonymous.sin6_scope_id;
            Ok(SocketAddrAny::V6(SocketAddrV6::new(
                Ipv6Addr::from(s6_addr),
                u16::from_be(decode.sin6_port),
                u32::from_be(decode.sin6_flowinfo),
                sin6_scope_id,
            )))
        }
        #[cfg(unix)]
        c::AF_UNIX => {
            if len < offsetof_sun_path {
                return Err(io::Errno::INVAL);
            }
            if len == offsetof_sun_path {
                SocketAddrUnix::new(&[][..]).map(SocketAddrAny::Unix)
            } else {
                let decode = &*storage.cast::<c::sockaddr_un>();

                // On Linux check for Linux's [abstract namespace].
                //
                // [abstract namespace]: https://man7.org/linux/man-pages/man7/unix.7.html
                #[cfg(linux_kernel)]
                if decode.sun_path[0] == 0 {
                    return SocketAddrUnix::new_abstract_name(core::mem::transmute::<
                        &[c::c_char],
                        &[u8],
                    >(
                        &decode.sun_path[1..len - offsetof_sun_path],
                    ))
                    .map(SocketAddrAny::Unix);
                }

                // Otherwise we expect a NUL-terminated filesystem path.

                // Trim off unused bytes from the end of `path_bytes`.
                let path_bytes = if cfg!(target_os = "freebsd") {
                    // FreeBSD sometimes sets the length to longer than the length
                    // of the NUL-terminated string. Find the NUL and truncate the
                    // string accordingly.
                    &decode.sun_path[..decode
                        .sun_path
                        .iter()
                        .position(|b| *b == 0)
                        .ok_or(io::Errno::INVAL)?]
                } else {
                    // Otherwise, use the provided length.
                    let provided_len = len - 1 - offsetof_sun_path;
                    if decode.sun_path[provided_len] != 0 {
                        return Err(io::Errno::INVAL);
                    }
                    debug_assert_eq!(
                        CStr::from_ptr(decode.sun_path.as_ptr()).to_bytes().len(),
                        provided_len
                    );
                    &decode.sun_path[..provided_len]
                };

                SocketAddrUnix::new(core::mem::transmute::<&[c::c_char], &[u8]>(path_bytes))
                    .map(SocketAddrAny::Unix)
            }
        }
        _ => Err(io::Errno::INVAL),
    }
}

pub(crate) unsafe fn maybe_read_sockaddr_os(
    storage: *const c::sockaddr_storage,
    len: usize,
) -> Option<SocketAddrAny> {
    if len == 0 {
        return None;
    }

    assert!(len >= size_of::<c::sa_family_t>());
    let family = read_ss_family(storage).into();
    if family == c::AF_UNSPEC {
        None
    } else {
        Some(inner_read_sockaddr_os(family, storage, len))
    }
}

pub(crate) unsafe fn read_sockaddr_os(
    storage: *const c::sockaddr_storage,
    len: usize,
) -> SocketAddrAny {
    assert!(len >= size_of::<c::sa_family_t>());
    let family = read_ss_family(storage).into();
    inner_read_sockaddr_os(family, storage, len)
}

unsafe fn inner_read_sockaddr_os(
    family: c::c_int,
    storage: *const c::sockaddr_storage,
    len: usize,
) -> SocketAddrAny {
    #[cfg(unix)]
    let offsetof_sun_path = super::addr::offsetof_sun_path();

    assert!(len >= size_of::<c::sa_family_t>());
    match family {
        c::AF_INET => {
            assert!(len >= size_of::<c::sockaddr_in>());
            let decode = &*storage.cast::<c::sockaddr_in>();
            SocketAddrAny::V4(SocketAddrV4::new(
                Ipv4Addr::from(u32::from_be(in_addr_s_addr(decode.sin_addr))),
                u16::from_be(decode.sin_port),
            ))
        }
        c::AF_INET6 => {
            assert!(len >= size_of::<c::sockaddr_in6>());
            let decode = &*storage.cast::<c::sockaddr_in6>();
            SocketAddrAny::V6(SocketAddrV6::new(
                Ipv6Addr::from(in6_addr_s6_addr(decode.sin6_addr)),
                u16::from_be(decode.sin6_port),
                u32::from_be(decode.sin6_flowinfo),
                sockaddr_in6_sin6_scope_id(decode),
            ))
        }
        #[cfg(unix)]
        c::AF_UNIX => {
            assert!(len >= offsetof_sun_path);
            if len == offsetof_sun_path {
                SocketAddrAny::Unix(SocketAddrUnix::new(&[][..]).unwrap())
            } else {
                let decode = &*storage.cast::<c::sockaddr_un>();

                // On Linux check for Linux's [abstract namespace].
                //
                // [abstract namespace]: https://man7.org/linux/man-pages/man7/unix.7.html
                #[cfg(linux_kernel)]
                if decode.sun_path[0] == 0 {
                    return SocketAddrAny::Unix(
                        SocketAddrUnix::new_abstract_name(core::mem::transmute::<
                            &[c::c_char],
                            &[u8],
                        >(
                            &decode.sun_path[1..len - offsetof_sun_path],
                        ))
                        .unwrap(),
                    );
                }

                // Otherwise we expect a NUL-terminated filesystem path.
                assert_eq!(decode.sun_path[len - 1 - offsetof_sun_path], 0);
                let path_bytes = &decode.sun_path[..len - 1 - offsetof_sun_path];

                // FreeBSD sometimes sets the length to longer than the length
                // of the NUL-terminated string. Find the NUL and truncate the
                // string accordingly.
                #[cfg(target_os = "freebsd")]
                let path_bytes = &path_bytes[..path_bytes.iter().position(|b| *b == 0).unwrap()];

                SocketAddrAny::Unix(
                    SocketAddrUnix::new(core::mem::transmute::<&[c::c_char], &[u8]>(path_bytes))
                        .unwrap(),
                )
            }
        }
        other => unimplemented!("{:?}", other),
    }
}
