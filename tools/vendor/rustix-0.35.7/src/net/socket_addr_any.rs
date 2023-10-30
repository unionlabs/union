//! A socket address for any kind of socket.
//!
//! This is similar to [`std::net::SocketAddr`], but also supports Unix-domain
//! socket addresses.
//!
//! # Safety
//!
//! The `read` and `write` functions allow decoding and encoding from and to
//! OS-specific socket address representations in memory.
#![allow(unsafe_code)]

#[cfg(unix)]
use crate::net::SocketAddrUnix;
use crate::net::{AddressFamily, SocketAddrV4, SocketAddrV6};
use crate::{backend, io};
#[cfg(feature = "std")]
use core::fmt;

pub use backend::net::addr::SocketAddrStorage;

/// `struct sockaddr_storage` as a Rust enum.
#[derive(Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
#[doc(alias = "sockaddr")]
#[non_exhaustive]
pub enum SocketAddrAny {
    /// `struct sockaddr_in`
    V4(SocketAddrV4),
    /// `struct sockaddr_in6`
    V6(SocketAddrV6),
    /// `struct sockaddr_un`
    #[cfg(unix)]
    Unix(SocketAddrUnix),
}

impl SocketAddrAny {
    /// Return the address family of this socket address.
    #[inline]
    pub const fn address_family(&self) -> AddressFamily {
        match self {
            SocketAddrAny::V4(_) => AddressFamily::INET,
            SocketAddrAny::V6(_) => AddressFamily::INET6,
            #[cfg(unix)]
            SocketAddrAny::Unix(_) => AddressFamily::UNIX,
        }
    }

    /// Writes a platform-specific encoding of this socket address to
    /// the memory pointed to by `storage`, and returns the number of
    /// bytes used.
    ///
    /// # Safety
    ///
    /// `storage` must point to valid memory for encoding the socket
    /// address.
    pub unsafe fn write(&self, storage: *mut SocketAddrStorage) -> usize {
        backend::net::write_sockaddr::write_sockaddr(self, storage)
    }

    /// Reads a platform-specific encoding of a socket address from
    /// the memory pointed to by `storage`, which uses `len` bytes.
    ///
    /// # Safety
    ///
    /// `storage` must point to valid memory for decoding a socket
    /// address.
    pub unsafe fn read(storage: *const SocketAddrStorage, len: usize) -> io::Result<Self> {
        backend::net::read_sockaddr::read_sockaddr(storage, len)
    }
}

#[cfg(feature = "std")]
impl fmt::Debug for SocketAddrAny {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SocketAddrAny::V4(v4) => v4.fmt(fmt),
            SocketAddrAny::V6(v6) => v6.fmt(fmt),
            #[cfg(unix)]
            SocketAddrAny::Unix(unix) => unix.fmt(fmt),
        }
    }
}
