use super::super::c;
use bitflags::bitflags;

/// A type for holding raw integer socket types.
#[doc(hidden)]
pub type RawSocketType = u32;

/// `SOCK_*` constants for use with [`socket`].
///
/// [`socket`]: crate::net::socket
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SocketType(pub(crate) RawSocketType);

#[rustfmt::skip]
impl SocketType {
    /// `SOCK_STREAM`
    pub const STREAM: Self = Self(c::SOCK_STREAM as u32);

    /// `SOCK_DGRAM`
    pub const DGRAM: Self = Self(c::SOCK_DGRAM as u32);

    /// `SOCK_SEQPACKET`
    pub const SEQPACKET: Self = Self(c::SOCK_SEQPACKET as u32);

    /// `SOCK_RAW`
    pub const RAW: Self = Self(c::SOCK_RAW as u32);

    /// `SOCK_RDM`
    #[cfg(not(target_os = "haiku"))]
    pub const RDM: Self = Self(c::SOCK_RDM as u32);

    /// Constructs a `SocketType` from a raw integer.
    #[inline]
    pub const fn from_raw(raw: RawSocketType) -> Self {
        Self(raw)
    }

    /// Returns the raw integer for this `SocketType`.
    #[inline]
    pub const fn as_raw(self) -> RawSocketType {
        self.0
    }
}

/// A type for holding raw integer address families.
#[doc(hidden)]
pub type RawAddressFamily = c::sa_family_t;

/// `AF_*` constants.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct AddressFamily(pub(crate) RawAddressFamily);

#[rustfmt::skip]
#[allow(non_upper_case_globals)]
impl AddressFamily {
    /// `AF_UNSPEC`
    pub const UNSPEC: Self = Self(c::AF_UNSPEC as _);
    /// `AF_INET`
    pub const INET: Self = Self(c::AF_INET as _);
    /// `AF_INET6`
    pub const INET6: Self = Self(c::AF_INET6 as _);
    /// `AF_NETLINK`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const NETLINK: Self = Self(c::AF_NETLINK as _);
    /// `AF_UNIX`, aka `AF_LOCAL`
    #[doc(alias = "LOCAL")]
    pub const UNIX: Self = Self(c::AF_UNIX as _);
    /// `AF_AX25`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const AX25: Self = Self(c::AF_AX25 as _);
    /// `AF_IPX`
    pub const IPX: Self = Self(c::AF_IPX as _);
    /// `AF_APPLETALK`
    pub const APPLETALK: Self = Self(c::AF_APPLETALK as _);
    /// `AF_NETROM`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const NETROM: Self = Self(c::AF_NETROM as _);
    /// `AF_BRIDGE`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const BRIDGE: Self = Self(c::AF_BRIDGE as _);
    /// `AF_ATMPVC`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const ATMPVC: Self = Self(c::AF_ATMPVC as _);
    /// `AF_X25`
    #[cfg(not(any(
        bsd,
        windows,
        target_os = "haiku",
    )))]
    pub const X25: Self = Self(c::AF_X25 as _);
    /// `AF_ROSE`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const ROSE: Self = Self(c::AF_ROSE as _);
    /// `AF_DECnet`
    #[cfg(not(target_os = "haiku"))]
    pub const DECnet: Self = Self(c::AF_DECnet as _);
    /// `AF_NETBEUI`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const NETBEUI: Self = Self(c::AF_NETBEUI as _);
    /// `AF_SECURITY`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const SECURITY: Self = Self(c::AF_SECURITY as _);
    /// `AF_KEY`
    #[cfg(not(any(
        bsd,
        windows,
        target_os = "haiku",
    )))]
    pub const KEY: Self = Self(c::AF_KEY as _);
    /// `AF_PACKET`
    #[cfg(not(any(
        bsd,
        windows,
        target_os = "haiku",
    )))]
    pub const PACKET: Self = Self(c::AF_PACKET as _);
    /// `AF_ASH`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const ASH: Self = Self(c::AF_ASH as _);
    /// `AF_ECONET`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const ECONET: Self = Self(c::AF_ECONET as _);
    /// `AF_ATMSVC`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const ATMSVC: Self = Self(c::AF_ATMSVC as _);
    /// `AF_RDS`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const RDS: Self = Self(c::AF_RDS as _);
    /// `AF_SNA`
    #[cfg(not(target_os = "haiku"))]
    pub const SNA: Self = Self(c::AF_SNA as _);
    /// `AF_IRDA`
    #[cfg(not(any(
        bsd,
        solarish,
        target_os = "haiku",
    )))]
    pub const IRDA: Self = Self(c::AF_IRDA as _);
    /// `AF_PPPOX`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const PPPOX: Self = Self(c::AF_PPPOX as _);
    /// `AF_WANPIPE`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const WANPIPE: Self = Self(c::AF_WANPIPE as _);
    /// `AF_LLC`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const LLC: Self = Self(c::AF_LLC as _);
    /// `AF_CAN`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const CAN: Self = Self(c::AF_CAN as _);
    /// `AF_TIPC`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const TIPC: Self = Self(c::AF_TIPC as _);
    /// `AF_BLUETOOTH`
    #[cfg(not(any(apple, solarish, windows)))]
    pub const BLUETOOTH: Self = Self(c::AF_BLUETOOTH as _);
    /// `AF_IUCV`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const IUCV: Self = Self(c::AF_IUCV as _);
    /// `AF_RXRPC`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const RXRPC: Self = Self(c::AF_RXRPC as _);
    /// `AF_ISDN`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const ISDN: Self = Self(c::AF_ISDN as _);
    /// `AF_PHONET`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const PHONET: Self = Self(c::AF_PHONET as _);
    /// `AF_IEEE802154`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const IEEE802154: Self = Self(c::AF_IEEE802154 as _);

    /// Constructs a `AddressFamily` from a raw integer.
    #[inline]
    pub const fn from_raw(raw: RawAddressFamily) -> Self {
        Self(raw)
    }

    /// Returns the raw integer for this `AddressFamily`.
    #[inline]
    pub const fn as_raw(self) -> RawAddressFamily {
        self.0
    }
}

/// A type for holding raw integer protocols.
#[doc(hidden)]
pub type RawProtocol = i32;

/// `IPPROTO_*` constants for use with [`socket`] and [`socket_with`].
///
/// [`socket`]: crate::net::socket
/// [`socket_with`]: crate::net::socket_with
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct Protocol(pub(crate) RawProtocol);

#[rustfmt::skip]
impl Protocol {
    /// `IPPROTO_IP`
    pub const IP: Self = Self(c::IPPROTO_IP as _);
    /// `IPPROTO_ICMP`
    pub const ICMP: Self = Self(c::IPPROTO_ICMP as _);
    /// `IPPROTO_IGMP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const IGMP: Self = Self(c::IPPROTO_IGMP as _);
    /// `IPPROTO_IPIP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const IPIP: Self = Self(c::IPPROTO_IPIP as _);
    /// `IPPROTO_TCP`
    pub const TCP: Self = Self(c::IPPROTO_TCP as _);
    /// `IPPROTO_EGP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const EGP: Self = Self(c::IPPROTO_EGP as _);
    /// `IPPROTO_PUP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const PUP: Self = Self(c::IPPROTO_PUP as _);
    /// `IPPROTO_UDP`
    pub const UDP: Self = Self(c::IPPROTO_UDP as _);
    /// `IPPROTO_IDP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const IDP: Self = Self(c::IPPROTO_IDP as _);
    /// `IPPROTO_TP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const TP: Self = Self(c::IPPROTO_TP as _);
    /// `IPPROTO_DCCP`
    #[cfg(not(any(
        apple,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
        target_os = "openbsd",
    )))]
    pub const DCCP: Self = Self(c::IPPROTO_DCCP as _);
    /// `IPPROTO_IPV6`
    pub const IPV6: Self = Self(c::IPPROTO_IPV6 as _);
    /// `IPPROTO_RSVP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const RSVP: Self = Self(c::IPPROTO_RSVP as _);
    /// `IPPROTO_GRE`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const GRE: Self = Self(c::IPPROTO_GRE as _);
    /// `IPPROTO_ESP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const ESP: Self = Self(c::IPPROTO_ESP as _);
    /// `IPPROTO_AH`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const AH: Self = Self(c::IPPROTO_AH as _);
    /// `IPPROTO_MTP`
    #[cfg(not(any(
        solarish,
        netbsdlike,
        windows,
        target_os = "haiku",
    )))]
    pub const MTP: Self = Self(c::IPPROTO_MTP as _);
    /// `IPPROTO_BEETPH`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const BEETPH: Self = Self(c::IPPROTO_BEETPH as _);
    /// `IPPROTO_ENCAP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const ENCAP: Self = Self(c::IPPROTO_ENCAP as _);
    /// `IPPROTO_PIM`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const PIM: Self = Self(c::IPPROTO_PIM as _);
    /// `IPPROTO_COMP`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "haiku",
    )))]
    pub const COMP: Self = Self(c::IPPROTO_COMP as _);
    /// `IPPROTO_SCTP`
    #[cfg(not(any(solarish, target_os = "dragonfly", target_os = "haiku", target_os = "openbsd")))]
    pub const SCTP: Self = Self(c::IPPROTO_SCTP as _);
    /// `IPPROTO_UDPLITE`
    #[cfg(not(any(
        apple,
        netbsdlike,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
    )))]
    pub const UDPLITE: Self = Self(c::IPPROTO_UDPLITE as _);
    /// `IPPROTO_MPLS`
    #[cfg(not(any(
        apple,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
        target_os = "netbsd",
    )))]
    pub const MPLS: Self = Self(c::IPPROTO_MPLS as _);
    /// `IPPROTO_RAW`
    pub const RAW: Self = Self(c::IPPROTO_RAW as _);
    /// `IPPROTO_MPTCP`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "haiku",
    )))]
    pub const MPTCP: Self = Self(c::IPPROTO_MPTCP as _);
    /// `IPPROTO_FRAGMENT`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const FRAGMENT: Self = Self(c::IPPROTO_FRAGMENT as _);
    /// `IPPROTO_ICMPV6`
    pub const ICMPV6: Self = Self(c::IPPROTO_ICMPV6 as _);
    /// `IPPROTO_MH`
    #[cfg(not(any(
        apple,
        netbsdlike,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
    )))]
    pub const MH: Self = Self(c::IPPROTO_MH as _);
    /// `IPPROTO_ROUTING`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const ROUTING: Self = Self(c::IPPROTO_ROUTING as _);

    /// Constructs a `Protocol` from a raw integer.
    #[inline]
    pub const fn from_raw(raw: RawProtocol) -> Self {
        Self(raw)
    }

    /// Returns the raw integer for this `Protocol`.
    #[inline]
    pub const fn as_raw(self) -> RawProtocol {
        self.0
    }
}

/// `SHUT_*` constants for use with [`shutdown`].
///
/// [`shutdown`]: crate::net::shutdown
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Shutdown {
    /// `SHUT_RD`—Disable further read operations.
    Read = c::SHUT_RD,
    /// `SHUT_WR`—Disable further write operations.
    Write = c::SHUT_WR,
    /// `SHUT_RDWR`—Disable further read and write operations.
    ReadWrite = c::SHUT_RDWR,
}

bitflags! {
    /// `SOCK_*` constants for use with [`socket_with`], [`accept_with`] and
    /// [`acceptfrom_with`].
    ///
    /// [`socket_with`]: crate::net::socket_with
    /// [`accept_with`]: crate::net::accept_with
    /// [`acceptfrom_with`]: crate::net::acceptfrom_with
    pub struct SocketFlags: c::c_int {
        /// `SOCK_NONBLOCK`
        #[cfg(not(any(apple, windows, target_os = "haiku")))]
        const NONBLOCK = c::SOCK_NONBLOCK;

        /// `SOCK_CLOEXEC`
        #[cfg(not(any(apple, windows, target_os = "haiku")))]
        const CLOEXEC = c::SOCK_CLOEXEC;
    }
}

/// Timeout identifier for use with [`set_socket_timeout`] and
/// [`get_socket_timeout`].
///
/// [`set_socket_timeout`]: crate::net::sockopt::set_socket_timeout.
/// [`get_socket_timeout`]: crate::net::sockopt::get_socket_timeout.
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(i32)]
pub enum Timeout {
    /// `SO_RCVTIMEO`—Timeout for receiving.
    Recv = c::SO_RCVTIMEO,

    /// `SO_SNDTIMEO`—Timeout for sending.
    Send = c::SO_SNDTIMEO,
}
