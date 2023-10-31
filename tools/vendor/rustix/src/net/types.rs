#![allow(unsafe_code)]

use crate::backend::c;
use bitflags::bitflags;

/// A type for holding raw integer socket types.
#[doc(hidden)]
pub type RawSocketType = u32;

/// `SOCK_*` constants for use with [`socket`].
///
/// [`socket`]: crate::net::socket()
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
pub struct SocketType(pub(crate) RawSocketType);

#[rustfmt::skip]
impl SocketType {
    /// `SOCK_STREAM`
    pub const STREAM: Self = Self(c::SOCK_STREAM as _);

    /// `SOCK_DGRAM`
    pub const DGRAM: Self = Self(c::SOCK_DGRAM as _);

    /// `SOCK_SEQPACKET`
    pub const SEQPACKET: Self = Self(c::SOCK_SEQPACKET as _);

    /// `SOCK_RAW`
    pub const RAW: Self = Self(c::SOCK_RAW as _);

    /// `SOCK_RDM`
    #[cfg(not(target_os = "haiku"))]
    pub const RDM: Self = Self(c::SOCK_RDM as _);

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
    ///
    /// # References
    ///  - [Linux]
    ///
    /// [Linux]: https://man7.org/linux/man-pages/man7/ip.7.html>
    pub const INET: Self = Self(c::AF_INET as _);
    /// `AF_INET6`
    ///
    /// # References
    ///  - [Linux]
    ///
    /// [Linux]: https://man7.org/linux/man-pages/man7/ipv6.7.html
    pub const INET6: Self = Self(c::AF_INET6 as _);
    /// `AF_NETLINK`
    ///
    /// # References
    ///  - [Linux]
    ///
    /// [Linux]: https://man7.org/linux/man-pages/man7/netlink.7.html
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
    ///
    /// # References
    ///  - [Linux]
    ///
    /// [Linux]: https://man7.org/linux/man-pages/man7/packet.7.html
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
    /// `AF_802`
    #[cfg(solarish)]
    pub const EIGHT_ZERO_TWO: Self = Self(c::AF_802 as _);
    #[cfg(target_os = "fuchsia")]
    /// `AF_ALG`
    pub const ALG: Self = Self(c::AF_ALG as _);
    #[cfg(any(target_os = "freebsd", target_os = "netbsd", target_os = "nto"))]
    /// `AF_ARP`
    pub const ARP: Self = Self(c::AF_ARP as _);
    /// `AF_ATM`
    #[cfg(freebsdlike)]
    pub const ATM: Self = Self(c::AF_ATM as _);
    /// `AF_CAIF`
    #[cfg(any(target_os = "android", target_os = "emscripten", target_os = "fuchsia"))]
    pub const CAIF: Self = Self(c::AF_CAIF as _);
    /// `AF_CCITT`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const CCITT: Self = Self(c::AF_CCITT as _);
    /// `AF_CHAOS`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const CHAOS: Self = Self(c::AF_CHAOS as _);
    /// `AF_CNT`
    #[cfg(any(bsd, target_os = "nto"))]
    pub const CNT: Self = Self(c::AF_CNT as _);
    /// `AF_COIP`
    #[cfg(any(bsd, target_os = "nto"))]
    pub const COIP: Self = Self(c::AF_COIP as _);
    /// `AF_DATAKIT`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const DATAKIT: Self = Self(c::AF_DATAKIT as _);
    /// `AF_DLI`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "haiku", target_os = "nto"))]
    pub const DLI: Self = Self(c::AF_DLI as _);
    /// `AF_E164`
    #[cfg(any(bsd, target_os = "nto"))]
    pub const E164: Self = Self(c::AF_E164 as _);
    /// `AF_ECMA`
    #[cfg(any(apple, freebsdlike, solarish, target_os = "aix", target_os = "nto", target_os = "openbsd"))]
    pub const ECMA: Self = Self(c::AF_ECMA as _);
    /// `AF_ENCAP`
    #[cfg(target_os = "openbsd")]
    pub const ENCAP: Self = Self(c::AF_ENCAP as _);
    /// `AF_FILE`
    #[cfg(solarish)]
    pub const FILE: Self = Self(c::AF_FILE as _);
    /// `AF_GOSIP`
    #[cfg(solarish)]
    pub const GOSIP: Self = Self(c::AF_GOSIP as _);
    /// `AF_HYLINK`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const HYLINK: Self = Self(c::AF_HYLINK as _);
    /// `AF_IB`
    #[cfg(any(target_os = "emscripten", target_os = "fuchsia"))]
    pub const IB: Self = Self(c::AF_IB as _);
    /// `AF_IMPLINK`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const IMPLINK: Self = Self(c::AF_IMPLINK as _);
    /// `AF_IEEE80211`
    #[cfg(any(apple, freebsdlike, linuxlike, target_os = "netbsd"))]
    pub const IEEE80211: Self = Self(c::AF_IEEE80211 as _);
    /// `AF_INET6_SDP`
    #[cfg(target_os = "freebsd")]
    pub const INET6_SDP: Self = Self(c::AF_INET6_SDP as _);
    /// `AF_INET_OFFLOAD`
    #[cfg(solarish)]
    pub const INET_OFFLOAD: Self = Self(c::AF_INET_OFFLOAD as _);
    /// `AF_INET_SDP`
    #[cfg(target_os = "freebsd")]
    pub const INET_SDP: Self = Self(c::AF_INET_SDP as _);
    /// `AF_INTF`
    #[cfg(target_os = "aix")]
    pub const INTF: Self = Self(c::AF_INTF as _);
    /// `AF_ISO`
    #[cfg(any(bsd, target_os = "aix", target_os = "nto"))]
    pub const ISO: Self = Self(c::AF_ISO as _);
    /// `AF_LAT`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const LAT: Self = Self(c::AF_LAT as _);
    /// `AF_LINK`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "haiku", target_os = "nto"))]
    pub const LINK: Self = Self(c::AF_LINK as _);
    /// `AF_MPLS`
    #[cfg(any(netbsdlike, target_os = "dragonfly", target_os = "emscripten", target_os = "fuchsia"))]
    pub const MPLS: Self = Self(c::AF_MPLS as _);
    /// `AF_NATM`
    #[cfg(any(bsd, target_os = "aix", target_os = "nto"))]
    pub const NATM: Self = Self(c::AF_NATM as _);
    /// `AF_NBS`
    #[cfg(solarish)]
    pub const NBS: Self = Self(c::AF_NBS as _);
    /// `AF_NCA`
    #[cfg(solarish)]
    pub const NCA: Self = Self(c::AF_NCA as _);
    /// `AF_NDD`
    #[cfg(target_os = "aix")]
    pub const NDD: Self = Self(c::AF_NDD as _);
    /// `AF_NDRV`
    #[cfg(apple)]
    pub const NDRV: Self = Self(c::AF_NDRV as _);
    /// `AF_NETBIOS`
    #[cfg(any(apple, freebsdlike))]
    pub const NETBIOS: Self = Self(c::AF_NETBIOS as _);
    /// `AF_NETGRAPH`
    #[cfg(freebsdlike)]
    pub const NETGRAPH: Self = Self(c::AF_NETGRAPH as _);
    /// `AF_NIT`
    #[cfg(solarish)]
    pub const NIT: Self = Self(c::AF_NIT as _);
    /// `AF_NOTIFY`
    #[cfg(target_os = "haiku")]
    pub const NOTIFY: Self = Self(c::AF_NOTIFY as _);
    /// `AF_NFC`
    #[cfg(any(target_os = "emscripten", target_os = "fuchsia"))]
    pub const NFC: Self = Self(c::AF_NFC as _);
    /// `AF_NS`
    #[cfg(any(apple, solarish, netbsdlike, target_os = "aix", target_os = "nto"))]
    pub const NS: Self = Self(c::AF_NS as _);
    /// `AF_OROUTE`
    #[cfg(target_os = "netbsd")]
    pub const OROUTE: Self = Self(c::AF_OROUTE as _);
    /// `AF_OSI`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const OSI: Self = Self(c::AF_OSI as _);
    /// `AF_OSINET`
    #[cfg(solarish)]
    pub const OSINET: Self = Self(c::AF_OSINET as _);
    /// `AF_POLICY`
    #[cfg(solarish)]
    pub const POLICY: Self = Self(c::AF_POLICY as _);
    /// `AF_PPP`
    #[cfg(apple)]
    pub const PPP: Self = Self(c::AF_PPP as _);
    /// `AF_PUP`
    #[cfg(any(bsd, solarish, target_os = "aix", target_os = "nto"))]
    pub const PUP: Self = Self(c::AF_PUP as _);
    /// `AF_RIF`
    #[cfg(target_os = "aix")]
    pub const RIF: Self = Self(c::AF_RIF as _);
    /// `AF_ROUTE`
    #[cfg(any(bsd, solarish, target_os = "android", target_os = "emscripten", target_os = "fuchsia", target_os = "haiku", target_os = "nto"))]
    pub const ROUTE: Self = Self(c::AF_ROUTE as _);
    /// `AF_SCLUSTER`
    #[cfg(target_os = "freebsd")]
    pub const SCLUSTER: Self = Self(c::AF_SCLUSTER as _);
    /// `AF_SIP`
    #[cfg(any(apple, target_os = "freebsd", target_os = "opensbd"))]
    pub const SIP: Self = Self(c::AF_SIP as _);
    /// `AF_SLOW`
    #[cfg(target_os = "freebsd")]
    pub const SLOW: Self = Self(c::AF_SLOW as _);
    /// `AF_SYS_CONTROL`
    #[cfg(apple)]
    pub const SYS_CONTROL: Self = Self(c::AF_SYS_CONTROL as _);
    /// `AF_SYSTEM`
    #[cfg(apple)]
    pub const SYSTEM: Self = Self(c::AF_SYSTEM as _);
    /// `AF_TRILL`
    #[cfg(solarish)]
    pub const TRILL: Self = Self(c::AF_TRILL as _);
    /// `AF_UTUN`
    #[cfg(apple)]
    pub const UTUN: Self = Self(c::AF_UTUN as _);
    /// `AF_VSOCK`
    #[cfg(any(apple, target_os = "emscripten", target_os = "fuchsia"))]
    pub const VSOCK: Self = Self(c::AF_VSOCK as _);

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
pub type RawProtocol = core::num::NonZeroU32;

/// `IPPROTO_*` and other constants for use with [`socket`], [`socket_with`],
/// and [`socketpair`] when a nondefault value is desired. See the [`ipproto`],
/// [`sysproto`], and [`netlink`] modules for possible values.
///
/// For the default values, such as `IPPROTO_IP` or `NETLINK_ROUTE`, pass
/// `None` as the `protocol` argument in these functions.
///
/// [`socket`]: crate::net::socket()
/// [`socket_with`]: crate::net::socket_with
/// [`socketpair`]: crate::net::socketpair()
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
#[repr(transparent)]
#[doc(alias = "IPPROTO_IP")]
#[doc(alias = "NETLINK_ROUTE")]
pub struct Protocol(pub(crate) RawProtocol);

/// `IPPROTO_*` constants.
///
/// For `IPPROTO_IP`, pass `None` as the `protocol` argument.
pub mod ipproto {
    use super::{Protocol, RawProtocol};
    use crate::backend::c;

    /// `IPPROTO_ICMP`
    pub const ICMP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ICMP as _) });
    /// `IPPROTO_IGMP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const IGMP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_IGMP as _) });
    /// `IPPROTO_IPIP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const IPIP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_IPIP as _) });
    /// `IPPROTO_TCP`
    pub const TCP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_TCP as _) });
    /// `IPPROTO_EGP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const EGP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_EGP as _) });
    /// `IPPROTO_PUP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const PUP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_PUP as _) });
    /// `IPPROTO_UDP`
    pub const UDP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_UDP as _) });
    /// `IPPROTO_IDP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const IDP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_IDP as _) });
    /// `IPPROTO_TP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const TP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_TP as _) });
    /// `IPPROTO_DCCP`
    #[cfg(not(any(
        apple,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
        target_os = "openbsd",
    )))]
    pub const DCCP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_DCCP as _) });
    /// `IPPROTO_IPV6`
    pub const IPV6: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_IPV6 as _) });
    /// `IPPROTO_RSVP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const RSVP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_RSVP as _) });
    /// `IPPROTO_GRE`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const GRE: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_GRE as _) });
    /// `IPPROTO_ESP`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const ESP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ESP as _) });
    /// `IPPROTO_AH`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const AH: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_AH as _) });
    /// `IPPROTO_MTP`
    #[cfg(not(any(solarish, netbsdlike, windows, target_os = "haiku")))]
    pub const MTP: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_MTP as _) });
    /// `IPPROTO_BEETPH`
    #[cfg(not(any(bsd, solarish, windows, target_os = "haiku")))]
    pub const BEETPH: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_BEETPH as _) });
    /// `IPPROTO_ENCAP`
    #[cfg(not(any(solarish, windows, target_os = "haiku")))]
    pub const ENCAP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ENCAP as _) });
    /// `IPPROTO_PIM`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const PIM: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_PIM as _) });
    /// `IPPROTO_COMP`
    #[cfg(not(any(bsd, solarish, windows, target_os = "haiku")))]
    pub const COMP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_COMP as _) });
    /// `IPPROTO_SCTP`
    #[cfg(not(any(
        solarish,
        target_os = "dragonfly",
        target_os = "haiku",
        target_os = "openbsd"
    )))]
    pub const SCTP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_SCTP as _) });
    /// `IPPROTO_UDPLITE`
    #[cfg(not(any(
        apple,
        netbsdlike,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
    )))]
    pub const UDPLITE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_UDPLITE as _) });
    /// `IPPROTO_MPLS`
    #[cfg(not(any(
        apple,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
        target_os = "netbsd",
    )))]
    pub const MPLS: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_MPLS as _) });
    /// `IPPROTO_ETHERNET`
    #[cfg(linux_kernel)]
    pub const ETHERNET: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ETHERNET as _) });
    /// `IPPROTO_RAW`
    pub const RAW: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_RAW as _) });
    /// `IPPROTO_MPTCP`
    #[cfg(not(any(
        bsd,
        solarish,
        windows,
        target_os = "emscripten",
        target_os = "fuchsia",
        target_os = "haiku",
    )))]
    pub const MPTCP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_MPTCP as _) });
    /// `IPPROTO_FRAGMENT`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const FRAGMENT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_FRAGMENT as _) });
    /// `IPPROTO_ICMPV6`
    pub const ICMPV6: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ICMPV6 as _) });
    /// `IPPROTO_MH`
    #[cfg(not(any(
        apple,
        netbsdlike,
        solarish,
        windows,
        target_os = "dragonfly",
        target_os = "haiku",
    )))]
    pub const MH: Protocol = Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_MH as _) });
    /// `IPPROTO_ROUTING`
    #[cfg(not(any(solarish, target_os = "haiku")))]
    pub const ROUTING: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::IPPROTO_ROUTING as _) });
}

/// `SYSPROTO_*` constants.
pub mod sysproto {
    #[cfg(apple)]
    use {
        super::{Protocol, RawProtocol},
        crate::backend::c,
    };

    /// `SYSPROTO_EVENT`
    #[cfg(apple)]
    pub const EVENT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::SYSPROTO_EVENT as _) });

    /// `SYSPROTO_CONTROL`
    #[cfg(apple)]
    pub const CONTROL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::SYSPROTO_CONTROL as _) });
}

/// `NETLINK_*` constants.
///
/// For `NETLINK_ROUTE`, pass `None` as the `protocol` argument.
pub mod netlink {
    #[cfg(linux_kernel)]
    use {
        super::{Protocol, RawProtocol},
        crate::backend::c,
    };

    /// `NETLINK_UNUSED`
    #[cfg(linux_kernel)]
    pub const UNUSED: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_UNUSED as _) });
    /// `NETLINK_USERSOCK`
    #[cfg(linux_kernel)]
    pub const USERSOCK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_USERSOCK as _) });
    /// `NETLINK_FIREWALL`
    #[cfg(linux_kernel)]
    pub const FIREWALL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_FIREWALL as _) });
    /// `NETLINK_SOCK_DIAG`
    #[cfg(linux_kernel)]
    pub const SOCK_DIAG: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_SOCK_DIAG as _) });
    /// `NETLINK_NFLOG`
    #[cfg(linux_kernel)]
    pub const NFLOG: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_NFLOG as _) });
    /// `NETLINK_XFRM`
    #[cfg(linux_kernel)]
    pub const XFRM: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_XFRM as _) });
    /// `NETLINK_SELINUX`
    #[cfg(linux_kernel)]
    pub const SELINUX: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_SELINUX as _) });
    /// `NETLINK_ISCSI`
    #[cfg(linux_kernel)]
    pub const ISCSI: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_ISCSI as _) });
    /// `NETLINK_AUDIT`
    #[cfg(linux_kernel)]
    pub const AUDIT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_AUDIT as _) });
    /// `NETLINK_FIB_LOOKUP`
    #[cfg(linux_kernel)]
    pub const FIB_LOOKUP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_FIB_LOOKUP as _) });
    /// `NETLINK_CONNECTOR`
    #[cfg(linux_kernel)]
    pub const CONNECTOR: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_CONNECTOR as _) });
    /// `NETLINK_NETFILTER`
    #[cfg(linux_kernel)]
    pub const NETFILTER: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_NETFILTER as _) });
    /// `NETLINK_IP6_FW`
    #[cfg(linux_kernel)]
    pub const IP6_FW: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_IP6_FW as _) });
    /// `NETLINK_DNRTMSG`
    #[cfg(linux_kernel)]
    pub const DNRTMSG: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_DNRTMSG as _) });
    /// `NETLINK_KOBJECT_UEVENT`
    #[cfg(linux_kernel)]
    pub const KOBJECT_UEVENT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_KOBJECT_UEVENT as _) });
    /// `NETLINK_GENERIC`
    // This is defined on FreeBSD too, but it has the value 0, so it doesn't
    // fit in or `NonZeroU32`. It's unclear whether FreeBSD intends
    // `NETLINK_GENERIC` to be the default when Linux has `NETLINK_ROUTE`
    // as the default.
    #[cfg(linux_kernel)]
    pub const GENERIC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_GENERIC as _) });
    /// `NETLINK_SCSITRANSPORT`
    #[cfg(linux_kernel)]
    pub const SCSITRANSPORT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_SCSITRANSPORT as _) });
    /// `NETLINK_ECRYPTFS`
    #[cfg(linux_kernel)]
    pub const ECRYPTFS: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_ECRYPTFS as _) });
    /// `NETLINK_RDMA`
    #[cfg(linux_kernel)]
    pub const RDMA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_RDMA as _) });
    /// `NETLINK_CRYPTO`
    #[cfg(linux_kernel)]
    pub const CRYPTO: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_CRYPTO as _) });
    /// `NETLINK_INET_DIAG`
    #[cfg(linux_kernel)]
    pub const INET_DIAG: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_INET_DIAG as _) });
    /// `NETLINK_ADD_MEMBERSHIP`
    #[cfg(linux_kernel)]
    pub const ADD_MEMBERSHIP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_ADD_MEMBERSHIP as _) });
    /// `NETLINK_DROP_MEMBERSHIP`
    #[cfg(linux_kernel)]
    pub const DROP_MEMBERSHIP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_DROP_MEMBERSHIP as _) });
    /// `NETLINK_PKTINFO`
    #[cfg(linux_kernel)]
    pub const PKTINFO: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_PKTINFO as _) });
    /// `NETLINK_BROADCAST_ERROR`
    #[cfg(linux_kernel)]
    pub const BROADCAST_ERROR: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_BROADCAST_ERROR as _) });
    /// `NETLINK_NO_ENOBUFS`
    #[cfg(linux_kernel)]
    pub const NO_ENOBUFS: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_NO_ENOBUFS as _) });
    /// `NETLINK_RX_RING`
    #[cfg(linux_kernel)]
    pub const RX_RING: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_RX_RING as _) });
    /// `NETLINK_TX_RING`
    #[cfg(linux_kernel)]
    pub const TX_RING: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_TX_RING as _) });
    /// `NETLINK_LISTEN_ALL_NSID`
    #[cfg(linux_kernel)]
    pub const LISTEN_ALL_NSID: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_LISTEN_ALL_NSID as _) });
    /// `NETLINK_LIST_MEMBERSHIPS`
    #[cfg(linux_kernel)]
    pub const LIST_MEMBERSHIPS: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_LIST_MEMBERSHIPS as _) });
    /// `NETLINK_CAP_ACK`
    #[cfg(linux_kernel)]
    pub const CAP_ACK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_CAP_ACK as _) });
    /// `NETLINK_EXT_ACK`
    #[cfg(linux_kernel)]
    pub const EXT_ACK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_EXT_ACK as _) });
    /// `NETLINK_GET_STRICT_CHK`
    #[cfg(linux_kernel)]
    pub const GET_STRICT_CHK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked(c::NETLINK_GET_STRICT_CHK as _) });
}

/// `ETH_P_*` constants.
///
// These are translated into 16-bit big-endian form because that's what the
// `AddressFamily::PACKET` address family [expects].
//
// [expects]: https://man7.org/linux/man-pages/man7/packet.7.html
pub mod eth {
    #[cfg(linux_kernel)]
    use {
        super::{Protocol, RawProtocol},
        crate::backend::c,
    };

    /// `ETH_P_LOOP`
    #[cfg(linux_kernel)]
    pub const LOOP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LOOP as u16).to_be() as u32) });
    /// `ETH_P_PUP`
    #[cfg(linux_kernel)]
    pub const PUP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PUP as u16).to_be() as u32) });
    /// `ETH_P_PUPAT`
    #[cfg(linux_kernel)]
    pub const PUPAT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PUPAT as u16).to_be() as u32) });
    /// `ETH_P_TSN`
    #[cfg(linux_kernel)]
    pub const TSN: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TSN as u16).to_be() as u32) });
    /// `ETH_P_ERSPAN2`
    #[cfg(linux_kernel)]
    pub const ERSPAN2: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ERSPAN2 as u16).to_be() as u32) });
    /// `ETH_P_IP`
    #[cfg(linux_kernel)]
    pub const IP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IP as u16).to_be() as u32) });
    /// `ETH_P_X25`
    #[cfg(linux_kernel)]
    pub const X25: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_X25 as u16).to_be() as u32) });
    /// `ETH_P_ARP`
    #[cfg(linux_kernel)]
    pub const ARP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ARP as u16).to_be() as u32) });
    /// `ETH_P_BPQ`
    #[cfg(linux_kernel)]
    pub const BPQ: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_BPQ as u16).to_be() as u32) });
    /// `ETH_P_IEEEPUP`
    #[cfg(linux_kernel)]
    pub const IEEEPUP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IEEEPUP as u16).to_be() as u32) });
    /// `ETH_P_IEEEPUPAT`
    #[cfg(linux_kernel)]
    pub const IEEEPUPAT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IEEEPUPAT as u16).to_be() as u32) });
    /// `ETH_P_BATMAN`
    #[cfg(linux_kernel)]
    pub const BATMAN: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_BATMAN as u16).to_be() as u32) });
    /// `ETH_P_DEC`
    #[cfg(linux_kernel)]
    pub const DEC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DEC as u16).to_be() as u32) });
    /// `ETH_P_DNA_DL`
    #[cfg(linux_kernel)]
    pub const DNA_DL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DNA_DL as u16).to_be() as u32) });
    /// `ETH_P_DNA_RC`
    #[cfg(linux_kernel)]
    pub const DNA_RC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DNA_RC as u16).to_be() as u32) });
    /// `ETH_P_DNA_RT`
    #[cfg(linux_kernel)]
    pub const DNA_RT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DNA_RT as u16).to_be() as u32) });
    /// `ETH_P_LAT`
    #[cfg(linux_kernel)]
    pub const LAT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LAT as u16).to_be() as u32) });
    /// `ETH_P_DIAG`
    #[cfg(linux_kernel)]
    pub const DIAG: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DIAG as u16).to_be() as u32) });
    /// `ETH_P_CUST`
    #[cfg(linux_kernel)]
    pub const CUST: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CUST as u16).to_be() as u32) });
    /// `ETH_P_SCA`
    #[cfg(linux_kernel)]
    pub const SCA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_SCA as u16).to_be() as u32) });
    /// `ETH_P_TEB`
    #[cfg(linux_kernel)]
    pub const TEB: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TEB as u16).to_be() as u32) });
    /// `ETH_P_RARP`
    #[cfg(linux_kernel)]
    pub const RARP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_RARP as u16).to_be() as u32) });
    /// `ETH_P_ATALK`
    #[cfg(linux_kernel)]
    pub const ATALK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ATALK as u16).to_be() as u32) });
    /// `ETH_P_AARP`
    #[cfg(linux_kernel)]
    pub const AARP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_AARP as u16).to_be() as u32) });
    /// `ETH_P_8021Q`
    #[cfg(linux_kernel)]
    pub const P_8021Q: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_8021Q as u16).to_be() as u32) });
    /// `ETH_P_ERSPAN`
    #[cfg(linux_kernel)]
    pub const ERSPAN: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ERSPAN as u16).to_be() as u32) });
    /// `ETH_P_IPX`
    #[cfg(linux_kernel)]
    pub const IPX: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IPX as u16).to_be() as u32) });
    /// `ETH_P_IPV6`
    #[cfg(linux_kernel)]
    pub const IPV6: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IPV6 as u16).to_be() as u32) });
    /// `ETH_P_PAUSE`
    #[cfg(linux_kernel)]
    pub const PAUSE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PAUSE as u16).to_be() as u32) });
    /// `ETH_P_SLOW`
    #[cfg(linux_kernel)]
    pub const SLOW: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_SLOW as u16).to_be() as u32) });
    /// `ETH_P_WCCP`
    #[cfg(linux_kernel)]
    pub const WCCP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_WCCP as u16).to_be() as u32) });
    /// `ETH_P_MPLS_UC`
    #[cfg(linux_kernel)]
    pub const MPLS_UC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MPLS_UC as u16).to_be() as u32) });
    /// `ETH_P_MPLS_MC`
    #[cfg(linux_kernel)]
    pub const MPLS_MC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MPLS_MC as u16).to_be() as u32) });
    /// `ETH_P_ATMMPOA`
    #[cfg(linux_kernel)]
    pub const ATMMPOA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ATMMPOA as u16).to_be() as u32) });
    /// `ETH_P_PPP_DISC`
    #[cfg(linux_kernel)]
    pub const PPP_DISC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PPP_DISC as u16).to_be() as u32) });
    /// `ETH_P_PPP_SES`
    #[cfg(linux_kernel)]
    pub const PPP_SES: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PPP_SES as u16).to_be() as u32) });
    /// `ETH_P_LINK_CTL`
    #[cfg(linux_kernel)]
    pub const LINK_CTL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LINK_CTL as u16).to_be() as u32) });
    /// `ETH_P_ATMFATE`
    #[cfg(linux_kernel)]
    pub const ATMFATE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ATMFATE as u16).to_be() as u32) });
    /// `ETH_P_PAE`
    #[cfg(linux_kernel)]
    pub const PAE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PAE as u16).to_be() as u32) });
    /// `ETH_P_PROFINET`
    #[cfg(linux_kernel)]
    pub const PROFINET: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PROFINET as u16).to_be() as u32) });
    /// `ETH_P_REALTEK`
    #[cfg(linux_kernel)]
    pub const REALTEK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_REALTEK as u16).to_be() as u32) });
    /// `ETH_P_AOE`
    #[cfg(linux_kernel)]
    pub const AOE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_AOE as u16).to_be() as u32) });
    /// `ETH_P_ETHERCAT`
    #[cfg(linux_kernel)]
    pub const ETHERCAT: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ETHERCAT as u16).to_be() as u32) });
    /// `ETH_P_8021AD`
    #[cfg(linux_kernel)]
    pub const P_8021AD: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_8021AD as u16).to_be() as u32) });
    /// `ETH_P_802_EX1`
    #[cfg(linux_kernel)]
    pub const P_802_EX1: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_802_EX1 as u16).to_be() as u32) });
    /// `ETH_P_PREAUTH`
    #[cfg(linux_kernel)]
    pub const PREAUTH: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PREAUTH as u16).to_be() as u32) });
    /// `ETH_P_TIPC`
    #[cfg(linux_kernel)]
    pub const TIPC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TIPC as u16).to_be() as u32) });
    /// `ETH_P_LLDP`
    #[cfg(linux_kernel)]
    pub const LLDP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LLDP as u16).to_be() as u32) });
    /// `ETH_P_MRP`
    #[cfg(linux_kernel)]
    pub const MRP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MRP as u16).to_be() as u32) });
    /// `ETH_P_MACSEC`
    #[cfg(linux_kernel)]
    pub const MACSEC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MACSEC as u16).to_be() as u32) });
    /// `ETH_P_8021AH`
    #[cfg(linux_kernel)]
    pub const P_8021AH: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_8021AH as u16).to_be() as u32) });
    /// `ETH_P_MVRP`
    #[cfg(linux_kernel)]
    pub const MVRP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MVRP as u16).to_be() as u32) });
    /// `ETH_P_1588`
    #[cfg(linux_kernel)]
    pub const P_1588: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_1588 as u16).to_be() as u32) });
    /// `ETH_P_NCSI`
    #[cfg(linux_kernel)]
    pub const NCSI: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_NCSI as u16).to_be() as u32) });
    /// `ETH_P_PRP`
    #[cfg(linux_kernel)]
    pub const PRP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PRP as u16).to_be() as u32) });
    /// `ETH_P_CFM`
    #[cfg(linux_kernel)]
    pub const CFM: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CFM as u16).to_be() as u32) });
    /// `ETH_P_FCOE`
    #[cfg(linux_kernel)]
    pub const FCOE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_FCOE as u16).to_be() as u32) });
    /// `ETH_P_IBOE`
    #[cfg(linux_kernel)]
    pub const IBOE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IBOE as u16).to_be() as u32) });
    /// `ETH_P_TDLS`
    #[cfg(linux_kernel)]
    pub const TDLS: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TDLS as u16).to_be() as u32) });
    /// `ETH_P_FIP`
    #[cfg(linux_kernel)]
    pub const FIP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_FIP as u16).to_be() as u32) });
    /// `ETH_P_80221`
    #[cfg(linux_kernel)]
    pub const P_80221: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_80221 as u16).to_be() as u32) });
    /// `ETH_P_HSR`
    #[cfg(linux_kernel)]
    pub const HSR: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_HSR as u16).to_be() as u32) });
    /// `ETH_P_NSH`
    #[cfg(linux_kernel)]
    pub const NSH: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_NSH as u16).to_be() as u32) });
    /// `ETH_P_LOOPBACK`
    #[cfg(linux_kernel)]
    pub const LOOPBACK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LOOPBACK as u16).to_be() as u32) });
    /// `ETH_P_QINQ1`
    #[cfg(linux_kernel)]
    pub const QINQ1: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_QINQ1 as u16).to_be() as u32) });
    /// `ETH_P_QINQ2`
    #[cfg(linux_kernel)]
    pub const QINQ2: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_QINQ2 as u16).to_be() as u32) });
    /// `ETH_P_QINQ3`
    #[cfg(linux_kernel)]
    pub const QINQ3: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_QINQ3 as u16).to_be() as u32) });
    /// `ETH_P_EDSA`
    #[cfg(linux_kernel)]
    pub const EDSA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_EDSA as u16).to_be() as u32) });
    /// `ETH_P_DSA_8021Q`
    #[cfg(linux_kernel)]
    pub const DSA_8021Q: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DSA_8021Q as u16).to_be() as u32) });
    /// `ETH_P_DSA_A5PSW`
    #[cfg(linux_kernel)]
    pub const DSA_A5PSW: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DSA_A5PSW as u16).to_be() as u32) });
    /// `ETH_P_IFE`
    #[cfg(linux_kernel)]
    pub const IFE: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IFE as u16).to_be() as u32) });
    /// `ETH_P_AF_IUCV`
    #[cfg(linux_kernel)]
    pub const AF_IUCV: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_AF_IUCV as u16).to_be() as u32) });
    /// `ETH_P_802_3_MIN`
    #[cfg(linux_kernel)]
    pub const P_802_3_MIN: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_802_3_MIN as u16).to_be() as u32) });
    /// `ETH_P_802_3`
    #[cfg(linux_kernel)]
    pub const P_802_3: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_802_3 as u16).to_be() as u32) });
    /// `ETH_P_AX25`
    #[cfg(linux_kernel)]
    pub const AX25: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_AX25 as u16).to_be() as u32) });
    /// `ETH_P_ALL`
    #[cfg(linux_kernel)]
    pub const ALL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ALL as u16).to_be() as u32) });
    /// `ETH_P_802_2`
    #[cfg(linux_kernel)]
    pub const P_802_2: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_802_2 as u16).to_be() as u32) });
    /// `ETH_P_SNAP`
    #[cfg(linux_kernel)]
    pub const SNAP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_SNAP as u16).to_be() as u32) });
    /// `ETH_P_DDCMP`
    #[cfg(linux_kernel)]
    pub const DDCMP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DDCMP as u16).to_be() as u32) });
    /// `ETH_P_WAN_PPP`
    #[cfg(linux_kernel)]
    pub const WAN_PPP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_WAN_PPP as u16).to_be() as u32) });
    /// `ETH_P_PPP_MP`
    #[cfg(linux_kernel)]
    pub const PPP_MP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PPP_MP as u16).to_be() as u32) });
    /// `ETH_P_LOCALTALK`
    #[cfg(linux_kernel)]
    pub const LOCALTALK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_LOCALTALK as u16).to_be() as u32) });
    /// `ETH_P_CAN`
    #[cfg(linux_kernel)]
    pub const CAN: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CAN as u16).to_be() as u32) });
    /// `ETH_P_CANFD`
    #[cfg(linux_kernel)]
    pub const CANFD: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CANFD as u16).to_be() as u32) });
    /// `ETH_P_CANXL`
    #[cfg(linux_kernel)]
    pub const CANXL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CANXL as u16).to_be() as u32) });
    /// `ETH_P_PPPTALK`
    #[cfg(linux_kernel)]
    pub const PPPTALK: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PPPTALK as u16).to_be() as u32) });
    /// `ETH_P_TR_802_2`
    #[cfg(linux_kernel)]
    pub const TR_802_2: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TR_802_2 as u16).to_be() as u32) });
    /// `ETH_P_MOBITEX`
    #[cfg(linux_kernel)]
    pub const MOBITEX: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MOBITEX as u16).to_be() as u32) });
    /// `ETH_P_CONTROL`
    #[cfg(linux_kernel)]
    pub const CONTROL: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CONTROL as u16).to_be() as u32) });
    /// `ETH_P_IRDA`
    #[cfg(linux_kernel)]
    pub const IRDA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_IRDA as u16).to_be() as u32) });
    /// `ETH_P_ECONET`
    #[cfg(linux_kernel)]
    pub const ECONET: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ECONET as u16).to_be() as u32) });
    /// `ETH_P_HDLC`
    #[cfg(linux_kernel)]
    pub const HDLC: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_HDLC as u16).to_be() as u32) });
    /// `ETH_P_ARCNET`
    #[cfg(linux_kernel)]
    pub const ARCNET: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_ARCNET as u16).to_be() as u32) });
    /// `ETH_P_DSA`
    #[cfg(linux_kernel)]
    pub const DSA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_DSA as u16).to_be() as u32) });
    /// `ETH_P_TRAILER`
    #[cfg(linux_kernel)]
    pub const TRAILER: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_TRAILER as u16).to_be() as u32) });
    /// `ETH_P_PHONET`
    #[cfg(linux_kernel)]
    pub const PHONET: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_PHONET as u16).to_be() as u32) });
    /// `ETH_P_IEEE802154`
    #[cfg(linux_kernel)]
    pub const IEEE802154: Protocol = Protocol(unsafe {
        RawProtocol::new_unchecked((c::ETH_P_IEEE802154 as u16).to_be() as u32)
    });
    /// `ETH_P_CAIF`
    #[cfg(linux_kernel)]
    pub const CAIF: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_CAIF as u16).to_be() as u32) });
    /// `ETH_P_XDSA`
    #[cfg(linux_kernel)]
    pub const XDSA: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_XDSA as u16).to_be() as u32) });
    /// `ETH_P_MAP`
    #[cfg(linux_kernel)]
    pub const MAP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MAP as u16).to_be() as u32) });
    /// `ETH_P_MCTP`
    #[cfg(linux_kernel)]
    pub const MCTP: Protocol =
        Protocol(unsafe { RawProtocol::new_unchecked((c::ETH_P_MCTP as u16).to_be() as u32) });
}

#[rustfmt::skip]
impl Protocol {
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
#[repr(u32)]
pub enum Shutdown {
    /// `SHUT_RD`—Disable further read operations.
    Read = c::SHUT_RD as _,
    /// `SHUT_WR`—Disable further write operations.
    Write = c::SHUT_WR as _,
    /// `SHUT_RDWR`—Disable further read and write operations.
    ReadWrite = c::SHUT_RDWR as _,
}

bitflags! {
    /// `SOCK_*` constants for use with [`socket_with`], [`accept_with`] and
    /// [`acceptfrom_with`].
    ///
    /// [`socket_with`]: crate::net::socket_with
    /// [`accept_with`]: crate::net::accept_with
    /// [`acceptfrom_with`]: crate::net::acceptfrom_with
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct SocketFlags: c::c_uint {
        /// `SOCK_NONBLOCK`
        #[cfg(not(any(apple, windows, target_os = "haiku")))]
        const NONBLOCK = bitcast!(c::SOCK_NONBLOCK);

        /// `SOCK_CLOEXEC`
        #[cfg(not(any(apple, windows, target_os = "haiku")))]
        const CLOEXEC = bitcast!(c::SOCK_CLOEXEC);
    }
}

#[test]
fn test_sizes() {
    use c::c_int;
    use core::mem::size_of;

    // Backend code needs to cast these to `c_int` so make sure that cast
    // isn't lossy.
    assert_eq!(size_of::<RawProtocol>(), size_of::<c_int>());
    assert_eq!(size_of::<Protocol>(), size_of::<c_int>());
    assert_eq!(size_of::<Option<Protocol>>(), size_of::<c_int>());
    assert_eq!(size_of::<RawSocketType>(), size_of::<c_int>());
    assert_eq!(size_of::<SocketType>(), size_of::<c_int>());
    assert_eq!(size_of::<SocketFlags>(), size_of::<c_int>());
}
