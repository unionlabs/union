use std::mem::{self, size_of, MaybeUninit};
use std::net::{SocketAddr, SocketAddrV4, SocketAddrV6};
use std::{fmt, io};

use crate::sys::{
    sa_family_t, sockaddr, sockaddr_in, sockaddr_in6, sockaddr_storage, socklen_t, AF_INET,
    AF_INET6,
};
#[cfg(windows)]
use winapi::shared::ws2ipdef::SOCKADDR_IN6_LH_u;

/// The address of a socket.
///
/// `SockAddr`s may be constructed directly to and from the standard library
/// [`SocketAddr`], [`SocketAddrV4`], and [`SocketAddrV6`] types.
pub struct SockAddr {
    storage: sockaddr_storage,
    len: socklen_t,
}

#[allow(clippy::len_without_is_empty)]
impl SockAddr {
    /// Create a `SockAddr` from the underlying storage and its length.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the address family and length match the type of
    /// storage address. For example if `storage.ss_family` is set to `AF_INET`
    /// the `storage` must be initialised as `sockaddr_in`, setting the content
    /// and length appropriately.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # #[cfg(unix)] {
    /// use std::io;
    /// use std::mem;
    /// use std::os::unix::io::AsRawFd;
    ///
    /// use socket2::{SockAddr, Socket, Domain, Type};
    ///
    /// let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    ///
    /// // Initialise a `SocketAddr` byte calling `getsockname(2)`.
    /// let mut addr_storage: libc::sockaddr_storage = unsafe { mem::zeroed() };
    /// let mut len = mem::size_of_val(&addr_storage) as libc::socklen_t;
    ///
    /// // The `getsockname(2)` system call will intiliase `storage` for
    /// // us, setting `len` to the correct length.
    /// let res = unsafe {
    ///     libc::getsockname(
    ///         socket.as_raw_fd(),
    ///         (&mut addr_storage as *mut libc::sockaddr_storage).cast(),
    ///         &mut len,
    ///     )
    /// };
    /// if res == -1 {
    ///     return Err(io::Error::last_os_error());
    /// }
    ///
    /// let address = unsafe { SockAddr::new(addr_storage, len) };
    /// # drop(address);
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub const unsafe fn new(storage: sockaddr_storage, len: socklen_t) -> SockAddr {
        SockAddr { storage, len }
    }

    /// Initialise a `SockAddr` by calling the function `init`.
    ///
    /// The type of the address storage and length passed to the function `init`
    /// is OS/architecture specific.
    ///
    /// The address is zeroed before `init` is called and is thus valid to
    /// dereference and read from. The length initialised to the maximum length
    /// of the storage.
    ///
    /// # Safety
    ///
    /// Caller must ensure that the address family and length match the type of
    /// storage address. For example if `storage.ss_family` is set to `AF_INET`
    /// the `storage` must be initialised as `sockaddr_in`, setting the content
    /// and length appropriately.
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> std::io::Result<()> {
    /// # #[cfg(unix)] {
    /// use std::io;
    /// use std::os::unix::io::AsRawFd;
    ///
    /// use socket2::{SockAddr, Socket, Domain, Type};
    ///
    /// let socket = Socket::new(Domain::IPV4, Type::STREAM, None)?;
    ///
    /// // Initialise a `SocketAddr` byte calling `getsockname(2)`.
    /// let (_, address) = unsafe {
    ///     SockAddr::init(|addr_storage, len| {
    ///         // The `getsockname(2)` system call will intiliase `storage` for
    ///         // us, setting `len` to the correct length.
    ///         if libc::getsockname(socket.as_raw_fd(), addr_storage.cast(), len) == -1 {
    ///             Err(io::Error::last_os_error())
    ///         } else {
    ///             Ok(())
    ///         }
    ///     })
    /// }?;
    /// # drop(address);
    /// # }
    /// # Ok(())
    /// # }
    /// ```
    pub unsafe fn init<F, T>(init: F) -> io::Result<(T, SockAddr)>
    where
        F: FnOnce(*mut sockaddr_storage, *mut socklen_t) -> io::Result<T>,
    {
        const STORAGE_SIZE: socklen_t = size_of::<sockaddr_storage>() as socklen_t;
        // NOTE: `SockAddr::unix` depends on the storage being zeroed before
        // calling `init`.
        // NOTE: calling `recvfrom` with an empty buffer also depends on the
        // storage being zeroed before calling `init` as the OS might not
        // initialise it.
        let mut storage = MaybeUninit::<sockaddr_storage>::zeroed();
        let mut len = STORAGE_SIZE;
        init(storage.as_mut_ptr(), &mut len).map(|res| {
            debug_assert!(len <= STORAGE_SIZE, "overflown address storage");
            let addr = SockAddr {
                // Safety: zeroed-out `sockaddr_storage` is valid, caller must
                // ensure at least `len` bytes are valid.
                storage: storage.assume_init(),
                len,
            };
            (res, addr)
        })
    }

    /// Returns this address's family.
    pub const fn family(&self) -> sa_family_t {
        self.storage.ss_family
    }

    /// Returns the size of this address in bytes.
    pub const fn len(&self) -> socklen_t {
        self.len
    }

    /// Returns a raw pointer to the address.
    pub const fn as_ptr(&self) -> *const sockaddr {
        &self.storage as *const _ as *const _
    }

    /// Returns a raw pointer to the address storage.
    #[cfg(all(unix, not(target_os = "redox")))]
    pub(crate) const fn as_storage_ptr(&self) -> *const sockaddr_storage {
        &self.storage
    }

    /// Returns this address as a `SocketAddr` if it is in the `AF_INET` (IPv4)
    /// or `AF_INET6` (IPv6) family, otherwise returns `None`.
    pub fn as_socket(&self) -> Option<SocketAddr> {
        if self.storage.ss_family == AF_INET as sa_family_t {
            // Safety: if the ss_family field is AF_INET then storage must be a sockaddr_in.
            let addr = unsafe { &*(&self.storage as *const _ as *const sockaddr_in) };

            let ip = crate::sys::from_in_addr(addr.sin_addr);
            let port = u16::from_be(addr.sin_port);
            Some(SocketAddr::V4(SocketAddrV4::new(ip, port)))
        } else if self.storage.ss_family == AF_INET6 as sa_family_t {
            // Safety: if the ss_family field is AF_INET6 then storage must be a sockaddr_in6.
            let addr = unsafe { &*(&self.storage as *const _ as *const sockaddr_in6) };

            let ip = crate::sys::from_in6_addr(addr.sin6_addr);
            let port = u16::from_be(addr.sin6_port);
            Some(SocketAddr::V6(SocketAddrV6::new(
                ip,
                port,
                addr.sin6_flowinfo,
                #[cfg(unix)]
                addr.sin6_scope_id,
                #[cfg(windows)]
                unsafe {
                    *addr.u.sin6_scope_id()
                },
            )))
        } else {
            None
        }
    }

    /// Returns this address as a [`SocketAddrV4`] if it is in the `AF_INET`
    /// family.
    pub fn as_socket_ipv4(&self) -> Option<SocketAddrV4> {
        match self.as_socket() {
            Some(SocketAddr::V4(addr)) => Some(addr),
            _ => None,
        }
    }

    /// Returns this address as a [`SocketAddrV6`] if it is in the `AF_INET6`
    /// family.
    pub fn as_socket_ipv6(&self) -> Option<SocketAddrV6> {
        match self.as_socket() {
            Some(SocketAddr::V6(addr)) => Some(addr),
            _ => None,
        }
    }
}

impl From<SocketAddr> for SockAddr {
    fn from(addr: SocketAddr) -> SockAddr {
        match addr {
            SocketAddr::V4(addr) => addr.into(),
            SocketAddr::V6(addr) => addr.into(),
        }
    }
}

impl From<SocketAddrV4> for SockAddr {
    fn from(addr: SocketAddrV4) -> SockAddr {
        let sockaddr_in = sockaddr_in {
            sin_family: AF_INET as sa_family_t,
            sin_port: addr.port().to_be(),
            sin_addr: crate::sys::to_in_addr(addr.ip()),
            sin_zero: Default::default(),
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "haiku",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sin_len: 0,
        };
        let mut storage = MaybeUninit::<sockaddr_storage>::zeroed();
        // Safety: A `sockaddr_in` is memory compatible with a `sockaddr_storage`
        unsafe { (storage.as_mut_ptr() as *mut sockaddr_in).write(sockaddr_in) };
        SockAddr {
            storage: unsafe { storage.assume_init() },
            len: mem::size_of::<sockaddr_in>() as socklen_t,
        }
    }
}

impl From<SocketAddrV6> for SockAddr {
    fn from(addr: SocketAddrV6) -> SockAddr {
        #[cfg(windows)]
        let u = unsafe {
            let mut u = mem::zeroed::<SOCKADDR_IN6_LH_u>();
            *u.sin6_scope_id_mut() = addr.scope_id();
            u
        };

        let sockaddr_in6 = sockaddr_in6 {
            sin6_family: AF_INET6 as sa_family_t,
            sin6_port: addr.port().to_be(),
            sin6_addr: crate::sys::to_in6_addr(addr.ip()),
            sin6_flowinfo: addr.flowinfo(),
            #[cfg(unix)]
            sin6_scope_id: addr.scope_id(),
            #[cfg(windows)]
            u,
            #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "haiku",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd"
            ))]
            sin6_len: 0,
            #[cfg(any(target_os = "solaris", target_os = "illumos"))]
            __sin6_src_id: 0,
        };
        let mut storage = MaybeUninit::<sockaddr_storage>::zeroed();
        // Safety: A `sockaddr_in6` is memory compatible with a `sockaddr_storage`
        unsafe { (storage.as_mut_ptr() as *mut sockaddr_in6).write(sockaddr_in6) };
        SockAddr {
            storage: unsafe { storage.assume_init() },
            len: mem::size_of::<sockaddr_in6>() as socklen_t,
        }
    }
}

impl fmt::Debug for SockAddr {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut f = fmt.debug_struct("SockAddr");
        #[cfg(any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "haiku",
            target_os = "hermit",
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd",
            target_os = "vxworks",
        ))]
        f.field("ss_len", &self.storage.ss_len);
        f.field("ss_family", &self.storage.ss_family)
            .field("len", &self.len)
            .finish()
    }
}

#[test]
fn ipv4() {
    use std::net::Ipv4Addr;
    let std = SocketAddrV4::new(Ipv4Addr::new(1, 2, 3, 4), 9876);
    let addr = SockAddr::from(std);
    assert_eq!(addr.family(), AF_INET as sa_family_t);
    assert_eq!(addr.len(), size_of::<sockaddr_in>() as socklen_t);
    assert_eq!(addr.as_socket(), Some(SocketAddr::V4(std)));
    assert_eq!(addr.as_socket_ipv4(), Some(std));
    assert!(addr.as_socket_ipv6().is_none());

    let addr = SockAddr::from(SocketAddr::from(std));
    assert_eq!(addr.family(), AF_INET as sa_family_t);
    assert_eq!(addr.len(), size_of::<sockaddr_in>() as socklen_t);
    assert_eq!(addr.as_socket(), Some(SocketAddr::V4(std)));
    assert_eq!(addr.as_socket_ipv4(), Some(std));
    assert!(addr.as_socket_ipv6().is_none());
}

#[test]
fn ipv6() {
    use std::net::Ipv6Addr;
    let std = SocketAddrV6::new(Ipv6Addr::new(1, 2, 3, 4, 5, 6, 7, 8), 9876, 11, 12);
    let addr = SockAddr::from(std);
    assert_eq!(addr.family(), AF_INET6 as sa_family_t);
    assert_eq!(addr.len(), size_of::<sockaddr_in6>() as socklen_t);
    assert_eq!(addr.as_socket(), Some(SocketAddr::V6(std)));
    assert!(addr.as_socket_ipv4().is_none());
    assert_eq!(addr.as_socket_ipv6(), Some(std));

    let addr = SockAddr::from(SocketAddr::from(std));
    assert_eq!(addr.family(), AF_INET6 as sa_family_t);
    assert_eq!(addr.len(), size_of::<sockaddr_in6>() as socklen_t);
    assert_eq!(addr.as_socket(), Some(SocketAddr::V6(std)));
    assert!(addr.as_socket_ipv4().is_none());
    assert_eq!(addr.as_socket_ipv6(), Some(std));
}
