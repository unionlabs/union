//! Socket options as used by `setsockopt` and `getsockopt`.
use cfg_if::cfg_if;
use super::{GetSockOpt, SetSockOpt};
use crate::Result;
use crate::errno::Errno;
use crate::sys::time::TimeVal;
use libc::{self, c_int, c_void, socklen_t};
use std::mem::{
    self,
    MaybeUninit
};
use std::os::unix::io::RawFd;
use std::ffi::{OsStr, OsString};
#[cfg(target_family = "unix")]
use std::os::unix::ffi::OsStrExt;

// Constants
// TCP_CA_NAME_MAX isn't defined in user space include files
#[cfg(any(target_os = "freebsd", target_os = "linux"))] 
const TCP_CA_NAME_MAX: usize = 16;

/// Helper for implementing `SetSockOpt` for a given socket option. See
/// [`::sys::socket::SetSockOpt`](sys/socket/trait.SetSockOpt.html).
///
/// This macro aims to help implementing `SetSockOpt` for different socket options that accept
/// different kinds of data to be used with `setsockopt`.
///
/// Instead of using this macro directly consider using `sockopt_impl!`, especially if the option
/// you are implementing represents a simple type.
///
/// # Arguments
///
/// * `$name:ident`: name of the type you want to implement `SetSockOpt` for.
/// * `$level:expr` : socket layer, or a `protocol level`: could be *raw sockets*
///    (`libc::SOL_SOCKET`), *ip protocol* (libc::IPPROTO_IP), *tcp protocol* (`libc::IPPROTO_TCP`),
///    and more. Please refer to your system manual for more options. Will be passed as the second
///    argument (`level`) to the `setsockopt` call.
/// * `$flag:path`: a flag name to set. Some examples: `libc::SO_REUSEADDR`, `libc::TCP_NODELAY`,
///    `libc::IP_ADD_MEMBERSHIP` and others. Will be passed as the third argument (`option_name`)
///    to the `setsockopt` call.
/// * Type of the value that you are going to set.
/// * Type that implements the `Set` trait for the type from the previous item (like `SetBool` for
///    `bool`, `SetUsize` for `usize`, etc.).
macro_rules! setsockopt_impl {
    ($name:ident, $level:expr, $flag:path, $ty:ty, $setter:ty) => {
        impl SetSockOpt for $name {
            type Val = $ty;

            fn set(&self, fd: RawFd, val: &$ty) -> Result<()> {
                unsafe {
                    let setter: $setter = Set::new(val);

                    let res = libc::setsockopt(fd, $level, $flag,
                                               setter.ffi_ptr(),
                                               setter.ffi_len());
                    Errno::result(res).map(drop)
                }
            }
        }
    }
}

/// Helper for implementing `GetSockOpt` for a given socket option. See
/// [`::sys::socket::GetSockOpt`](sys/socket/trait.GetSockOpt.html).
///
/// This macro aims to help implementing `GetSockOpt` for different socket options that accept
/// different kinds of data to be use with `getsockopt`.
///
/// Instead of using this macro directly consider using `sockopt_impl!`, especially if the option
/// you are implementing represents a simple type.
///
/// # Arguments
///
/// * Name of the type you want to implement `GetSockOpt` for.
/// * Socket layer, or a `protocol level`: could be *raw sockets* (`lic::SOL_SOCKET`),  *ip
///    protocol* (libc::IPPROTO_IP), *tcp protocol* (`libc::IPPROTO_TCP`),  and more. Please refer
///    to your system manual for more options. Will be passed as the second argument (`level`) to
///    the `getsockopt` call.
/// * A flag to set. Some examples: `libc::SO_REUSEADDR`, `libc::TCP_NODELAY`,
///    `libc::SO_ORIGINAL_DST` and others. Will be passed as the third argument (`option_name`) to
///    the `getsockopt` call.
/// * Type of the value that you are going to get.
/// * Type that implements the `Get` trait for the type from the previous item (`GetBool` for
///    `bool`, `GetUsize` for `usize`, etc.).
macro_rules! getsockopt_impl {
    ($name:ident, $level:expr, $flag:path, $ty:ty, $getter:ty) => {
        impl GetSockOpt for $name {
            type Val = $ty;

            fn get(&self, fd: RawFd) -> Result<$ty> {
                unsafe {
                    let mut getter: $getter = Get::uninit();

                    let res = libc::getsockopt(fd, $level, $flag,
                                               getter.ffi_ptr(),
                                               getter.ffi_len());
                    Errno::result(res)?;

                    Ok(getter.assume_init())
                }
            }
        }
    }
}

/// Helper to generate the sockopt accessors. See
/// [`::sys::socket::GetSockOpt`](sys/socket/trait.GetSockOpt.html) and
/// [`::sys::socket::SetSockOpt`](sys/socket/trait.SetSockOpt.html).
///
/// This macro aims to help implementing `GetSockOpt` and `SetSockOpt` for different socket options
/// that accept different kinds of data to be use with `getsockopt` and `setsockopt` respectively.
///
/// Basically this macro wraps up the [`getsockopt_impl!`](macro.getsockopt_impl.html) and
/// [`setsockopt_impl!`](macro.setsockopt_impl.html) macros.
///
/// # Arguments
///
/// * `GetOnly`, `SetOnly` or `Both`: whether you want to implement only getter, only setter or
///    both of them.
/// * `$name:ident`: name of type `GetSockOpt`/`SetSockOpt` will be implemented for.
/// * `$level:expr` : socket layer, or a `protocol level`: could be *raw sockets*
///    (`lic::SOL_SOCKET`), *ip protocol* (libc::IPPROTO_IP), *tcp protocol* (`libc::IPPROTO_TCP`),
///    and more. Please refer to your system manual for more options. Will be passed as the second
///    argument (`level`) to the `getsockopt`/`setsockopt` call.
/// * `$flag:path`: a flag name to set. Some examples: `libc::SO_REUSEADDR`, `libc::TCP_NODELAY`,
///    `libc::IP_ADD_MEMBERSHIP` and others. Will be passed as the third argument (`option_name`)
///    to the `setsockopt`/`getsockopt` call.
/// * `$ty:ty`: type of the value that will be get/set.
/// * `$getter:ty`: `Get` implementation; optional; only for `GetOnly` and `Both`.
/// * `$setter:ty`: `Set` implementation; optional; only for `SetOnly` and `Both`.
macro_rules! sockopt_impl {
    ($(#[$attr:meta])* $name:ident, GetOnly, $level:expr, $flag:path, bool) => {
        sockopt_impl!($(#[$attr])*
                      $name, GetOnly, $level, $flag, bool, GetBool);
    };

    ($(#[$attr:meta])* $name:ident, GetOnly, $level:expr, $flag:path, u8) => {
        sockopt_impl!($(#[$attr])* $name, GetOnly, $level, $flag, u8, GetU8);
    };

    ($(#[$attr:meta])* $name:ident, GetOnly, $level:expr, $flag:path, usize) =>
    {
        sockopt_impl!($(#[$attr])*
                      $name, GetOnly, $level, $flag, usize, GetUsize);
    };

    ($(#[$attr:meta])* $name:ident, SetOnly, $level:expr, $flag:path, bool) => {
        sockopt_impl!($(#[$attr])*
                      $name, SetOnly, $level, $flag, bool, SetBool);
    };

    ($(#[$attr:meta])* $name:ident, SetOnly, $level:expr, $flag:path, u8) => {
        sockopt_impl!($(#[$attr])* $name, SetOnly, $level, $flag, u8, SetU8);
    };

    ($(#[$attr:meta])* $name:ident, SetOnly, $level:expr, $flag:path, usize) =>
    {
        sockopt_impl!($(#[$attr])*
                      $name, SetOnly, $level, $flag, usize, SetUsize);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path, bool) => {
        sockopt_impl!($(#[$attr])*
                      $name, Both, $level, $flag, bool, GetBool, SetBool);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path, u8) => {
        sockopt_impl!($(#[$attr])*
                      $name, Both, $level, $flag, u8, GetU8, SetU8);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path, usize) => {
        sockopt_impl!($(#[$attr])*
                      $name, Both, $level, $flag, usize, GetUsize, SetUsize);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path,
     OsString<$array:ty>) =>
    {
        sockopt_impl!($(#[$attr])*
                      $name, Both, $level, $flag, OsString, GetOsString<$array>,
                      SetOsString);
    };

    /*
     * Matchers with generic getter types must be placed at the end, so
     * they'll only match _after_ specialized matchers fail
     */
    ($(#[$attr:meta])* $name:ident, GetOnly, $level:expr, $flag:path, $ty:ty) =>
    {
        sockopt_impl!($(#[$attr])*
                      $name, GetOnly, $level, $flag, $ty, GetStruct<$ty>);
    };

    ($(#[$attr:meta])* $name:ident, GetOnly, $level:expr, $flag:path, $ty:ty,
     $getter:ty) =>
    {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name;

        getsockopt_impl!($name, $level, $flag, $ty, $getter);
    };

    ($(#[$attr:meta])* $name:ident, SetOnly, $level:expr, $flag:path, $ty:ty) =>
    {
        sockopt_impl!($(#[$attr])*
                      $name, SetOnly, $level, $flag, $ty, SetStruct<$ty>);
    };

    ($(#[$attr:meta])* $name:ident, SetOnly, $level:expr, $flag:path, $ty:ty,
     $setter:ty) =>
    {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name;

        setsockopt_impl!($name, $level, $flag, $ty, $setter);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path, $ty:ty,
     $getter:ty, $setter:ty) =>
    {
        $(#[$attr])*
        #[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
        pub struct $name;

        setsockopt_impl!($name, $level, $flag, $ty, $setter);
        getsockopt_impl!($name, $level, $flag, $ty, $getter);
    };

    ($(#[$attr:meta])* $name:ident, Both, $level:expr, $flag:path, $ty:ty) => {
        sockopt_impl!($(#[$attr])*
                      $name, Both, $level, $flag, $ty, GetStruct<$ty>,
                      SetStruct<$ty>);
    };
}

/*
 *
 * ===== Define sockopts =====
 *
 */

sockopt_impl!(
    /// Enables local address reuse
    ReuseAddr, Both, libc::SOL_SOCKET, libc::SO_REUSEADDR, bool
);
#[cfg(not(any(target_os = "illumos", target_os = "solaris")))]
sockopt_impl!(
    /// Permits multiple AF_INET or AF_INET6 sockets to be bound to an
    /// identical socket address.
    ReusePort, Both, libc::SOL_SOCKET, libc::SO_REUSEPORT, bool);
sockopt_impl!(
    /// Under most circumstances, TCP sends data when it is presented; when
    /// outstanding data has not yet been acknowledged, it gathers small amounts
    /// of output to be sent in a single packet once an acknowledgement is
    /// received.  For a small number of clients, such as window systems that
    /// send a stream of mouse events which receive no replies, this
    /// packetization may cause significant delays.  The boolean option
    /// TCP_NODELAY defeats this algorithm.
    TcpNoDelay, Both, libc::IPPROTO_TCP, libc::TCP_NODELAY, bool);
sockopt_impl!(
    /// When enabled,  a close(2) or shutdown(2) will not return until all
    /// queued messages for the socket have been successfully sent or the
    /// linger timeout has been reached.
    Linger, Both, libc::SOL_SOCKET, libc::SO_LINGER, libc::linger);
sockopt_impl!(
    /// Join a multicast group
    IpAddMembership, SetOnly, libc::IPPROTO_IP, libc::IP_ADD_MEMBERSHIP,
    super::IpMembershipRequest);
sockopt_impl!(
    /// Leave a multicast group.
    IpDropMembership, SetOnly, libc::IPPROTO_IP, libc::IP_DROP_MEMBERSHIP,
    super::IpMembershipRequest);
cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        sockopt_impl!(
            /// Join an IPv6 multicast group.
            Ipv6AddMembership, SetOnly, libc::IPPROTO_IPV6, libc::IPV6_ADD_MEMBERSHIP, super::Ipv6MembershipRequest);
        sockopt_impl!(
            /// Leave an IPv6 multicast group.
            Ipv6DropMembership, SetOnly, libc::IPPROTO_IPV6, libc::IPV6_DROP_MEMBERSHIP, super::Ipv6MembershipRequest);
    } else if #[cfg(any(target_os = "dragonfly",
                        target_os = "freebsd",
                        target_os = "illumos",
                        target_os = "ios",
                        target_os = "macos",
                        target_os = "netbsd",
                        target_os = "openbsd",
                        target_os = "solaris"))] {
        sockopt_impl!(
            /// Join an IPv6 multicast group.
            Ipv6AddMembership, SetOnly, libc::IPPROTO_IPV6,
            libc::IPV6_JOIN_GROUP, super::Ipv6MembershipRequest);
        sockopt_impl!(
            /// Leave an IPv6 multicast group.
            Ipv6DropMembership, SetOnly, libc::IPPROTO_IPV6,
            libc::IPV6_LEAVE_GROUP, super::Ipv6MembershipRequest);
    }
}
sockopt_impl!(
    /// Set or read the time-to-live value of outgoing multicast packets for
    /// this socket.
    IpMulticastTtl, Both, libc::IPPROTO_IP, libc::IP_MULTICAST_TTL, u8);
sockopt_impl!(
    /// Set or read a boolean integer argument that determines whether sent
    /// multicast packets should be looped back to the local sockets.
    IpMulticastLoop, Both, libc::IPPROTO_IP, libc::IP_MULTICAST_LOOP, bool);
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
sockopt_impl!(
    /// If enabled, this boolean option allows binding to an IP address that
    /// is nonlocal or does not (yet) exist.
    IpFreebind, Both, libc::IPPROTO_IP, libc::IP_FREEBIND, bool);
sockopt_impl!(
    /// Specify the receiving timeout until reporting an error.
    ReceiveTimeout, Both, libc::SOL_SOCKET, libc::SO_RCVTIMEO, TimeVal);
sockopt_impl!(
    /// Specify the sending timeout until reporting an error.
    SendTimeout, Both, libc::SOL_SOCKET, libc::SO_SNDTIMEO, TimeVal);
sockopt_impl!(
    /// Set or get the broadcast flag.
    Broadcast, Both, libc::SOL_SOCKET, libc::SO_BROADCAST, bool);
sockopt_impl!(
    /// If this option is enabled, out-of-band data is directly placed into
    /// the receive data stream.
    OobInline, Both, libc::SOL_SOCKET, libc::SO_OOBINLINE, bool);
sockopt_impl!(
    /// Get and clear the pending socket error.
    SocketError, GetOnly, libc::SOL_SOCKET, libc::SO_ERROR, i32);
sockopt_impl!(
    /// Enable sending of keep-alive messages on connection-oriented sockets.
    KeepAlive, Both, libc::SOL_SOCKET, libc::SO_KEEPALIVE, bool);
#[cfg(any(
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "macos",
        target_os = "ios"
))]
sockopt_impl!(
    /// Get the credentials of the peer process of a connected unix domain
    /// socket.
    LocalPeerCred, GetOnly, 0, libc::LOCAL_PEERCRED, super::XuCred);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Return the credentials of the foreign process connected to this socket.
    PeerCredentials, GetOnly, libc::SOL_SOCKET, libc::SO_PEERCRED, super::UnixCredentials);
#[cfg(any(target_os = "ios",
          target_os = "macos"))]
sockopt_impl!(
    /// Specify the amount of time, in seconds, that the connection must be idle
    /// before keepalive probes (if enabled) are sent.
    TcpKeepAlive, Both, libc::IPPROTO_TCP, libc::TCP_KEEPALIVE, u32);
#[cfg(any(target_os = "android",
          target_os = "dragonfly",
          target_os = "freebsd",
          target_os = "linux",
          target_os = "nacl"))]
sockopt_impl!(
    /// The time (in seconds) the connection needs to remain idle before TCP
    /// starts sending keepalive probes
    TcpKeepIdle, Both, libc::IPPROTO_TCP, libc::TCP_KEEPIDLE, u32);
cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        sockopt_impl!(
            /// The maximum segment size for outgoing TCP packets.
            TcpMaxSeg, Both, libc::IPPROTO_TCP, libc::TCP_MAXSEG, u32);
    } else {
        sockopt_impl!(
            /// The maximum segment size for outgoing TCP packets.
            TcpMaxSeg, GetOnly, libc::IPPROTO_TCP, libc::TCP_MAXSEG, u32);
    }
}
#[cfg(not(target_os = "openbsd"))]
sockopt_impl!(
    /// The maximum number of keepalive probes TCP should send before
    /// dropping the connection.
    TcpKeepCount, Both, libc::IPPROTO_TCP, libc::TCP_KEEPCNT, u32);
#[cfg(any(target_os = "android",
          target_os = "fuchsia",
          target_os = "linux"))]
sockopt_impl!(
    #[allow(missing_docs)]
    // Not documented by Linux!
    TcpRepair, Both, libc::IPPROTO_TCP, libc::TCP_REPAIR, u32);
#[cfg(not(target_os = "openbsd"))]
sockopt_impl!(
    /// The time (in seconds) between individual keepalive probes.
    TcpKeepInterval, Both, libc::IPPROTO_TCP, libc::TCP_KEEPINTVL, u32);
#[cfg(any(target_os = "fuchsia", target_os = "linux"))]
sockopt_impl!(
    /// Specifies the maximum amount of time in milliseconds that transmitted
    /// data may remain unacknowledged before TCP will forcibly close the
    /// corresponding connection
    TcpUserTimeout, Both, libc::IPPROTO_TCP, libc::TCP_USER_TIMEOUT, u32);
sockopt_impl!(
    /// Sets or gets the maximum socket receive buffer in bytes.
    RcvBuf, Both, libc::SOL_SOCKET, libc::SO_RCVBUF, usize);
sockopt_impl!(
    /// Sets or gets the maximum socket send buffer in bytes.
    SndBuf, Both, libc::SOL_SOCKET, libc::SO_SNDBUF, usize);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Using this socket option, a privileged (`CAP_NET_ADMIN`) process can
    /// perform the same task as `SO_RCVBUF`, but the `rmem_max limit` can be
    /// overridden.
    RcvBufForce, SetOnly, libc::SOL_SOCKET, libc::SO_RCVBUFFORCE, usize);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Using this socket option, a privileged (`CAP_NET_ADMIN`)  process can
    /// perform the same task as `SO_SNDBUF`, but the `wmem_max` limit can be
    /// overridden.
    SndBufForce, SetOnly, libc::SOL_SOCKET, libc::SO_SNDBUFFORCE, usize);
sockopt_impl!(
    /// Gets the socket type as an integer.
    SockType, GetOnly, libc::SOL_SOCKET, libc::SO_TYPE, super::SockType);
sockopt_impl!(
    /// Returns a value indicating whether or not this socket has been marked to
    /// accept connections with `listen(2)`.
    AcceptConn, GetOnly, libc::SOL_SOCKET, libc::SO_ACCEPTCONN, bool);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Bind this socket to a particular device like “eth0”.
    BindToDevice, Both, libc::SOL_SOCKET, libc::SO_BINDTODEVICE, OsString<[u8; libc::IFNAMSIZ]>);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    #[allow(missing_docs)]
    // Not documented by Linux!
    OriginalDst, GetOnly, libc::SOL_IP, libc::SO_ORIGINAL_DST, libc::sockaddr_in);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    #[allow(missing_docs)]
    // Not documented by Linux!
    Ip6tOriginalDst, GetOnly, libc::SOL_IPV6, libc::IP6T_SO_ORIGINAL_DST, libc::sockaddr_in6);
sockopt_impl!( 
    /// Enable or disable the receiving of the `SO_TIMESTAMP` control message.
    ReceiveTimestamp, Both, libc::SOL_SOCKET, libc::SO_TIMESTAMP, bool);
#[cfg(all(target_os = "linux"))]
sockopt_impl!(
    /// Enable or disable the receiving of the `SO_TIMESTAMPNS` control message.
    ReceiveTimestampns, Both, libc::SOL_SOCKET, libc::SO_TIMESTAMPNS, bool);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Setting this boolean option enables transparent proxying on this socket.
    IpTransparent, Both, libc::SOL_IP, libc::IP_TRANSPARENT, bool);
#[cfg(target_os = "openbsd")]
sockopt_impl!(
    /// Allows the socket to be bound to addresses which are not local to the
    /// machine, so it can be used to make a transparent proxy.
    BindAny, Both, libc::SOL_SOCKET, libc::SO_BINDANY, bool);
#[cfg(target_os = "freebsd")]
sockopt_impl!(
    /// Can `bind(2)` to any address, even one not bound to any available
    /// network interface in the system.
    BindAny, Both, libc::IPPROTO_IP, libc::IP_BINDANY, bool);
#[cfg(target_os = "linux")]
sockopt_impl!(
    /// Set the mark for each packet sent through this socket (similar to the
    /// netfilter MARK target but socket-based).
    Mark, Both, libc::SOL_SOCKET, libc::SO_MARK, u32);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Enable or disable the receiving of the `SCM_CREDENTIALS` control
    /// message.
    PassCred, Both, libc::SOL_SOCKET, libc::SO_PASSCRED, bool);
#[cfg(any(target_os = "freebsd", target_os = "linux"))] 
sockopt_impl!(
    /// This option allows the caller to set the TCP congestion control
    /// algorithm to be used,  on a per-socket basis.
    TcpCongestion, Both, libc::IPPROTO_TCP, libc::TCP_CONGESTION, OsString<[u8; TCP_CA_NAME_MAX]>);
#[cfg(any(
    target_os = "android",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
))]
sockopt_impl!(
    /// Pass an `IP_PKTINFO` ancillary message that contains a pktinfo
    /// structure that supplies some information about the incoming packet.
    Ipv4PacketInfo, Both, libc::IPPROTO_IP, libc::IP_PKTINFO, bool);
#[cfg(any(
    target_os = "android",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
sockopt_impl!(
    /// Set delivery of the `IPV6_PKTINFO` control message on incoming
    /// datagrams.
    Ipv6RecvPacketInfo, Both, libc::IPPROTO_IPV6, libc::IPV6_RECVPKTINFO, bool);
#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
sockopt_impl!(
    /// The `recvmsg(2)` call returns a `struct sockaddr_dl` corresponding to
    /// the interface on which the packet was received.
    Ipv4RecvIf, Both, libc::IPPROTO_IP, libc::IP_RECVIF, bool);
#[cfg(any(
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
sockopt_impl!(
    /// The `recvmsg(2)` call will return the destination IP address for a UDP
    /// datagram.
    Ipv4RecvDstAddr, Both, libc::IPPROTO_IP, libc::IP_RECVDSTADDR, bool);
#[cfg(target_os = "linux")]
sockopt_impl!(
    #[allow(missing_docs)]
    // Not documented by Linux!
    UdpGsoSegment, Both, libc::SOL_UDP, libc::UDP_SEGMENT, libc::c_int);
#[cfg(target_os = "linux")]
sockopt_impl!(
    #[allow(missing_docs)]
    // Not documented by Linux!
    UdpGroSegment, Both, libc::IPPROTO_UDP, libc::UDP_GRO, bool);
#[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
sockopt_impl!(
    /// Indicates that an unsigned 32-bit value ancillary message (cmsg) should
    /// be attached to received skbs indicating the number of packets dropped by
    /// the socket since its creation.
    RxqOvfl, Both, libc::SOL_SOCKET, libc::SO_RXQ_OVFL, libc::c_int);
sockopt_impl!(
    /// The socket is restricted to sending and receiving IPv6 packets only.
    Ipv6V6Only, Both, libc::IPPROTO_IPV6, libc::IPV6_V6ONLY, bool);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Enable extended reliable error message passing.
    Ipv4RecvErr, Both, libc::IPPROTO_IP, libc::IP_RECVERR, bool);
#[cfg(any(target_os = "android", target_os = "linux"))]
sockopt_impl!(
    /// Control receiving of asynchronous error options.
    Ipv6RecvErr, Both, libc::IPPROTO_IPV6, libc::IPV6_RECVERR, bool);
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
sockopt_impl!(
    /// Set or retrieve the current time-to-live field that is used in every
    /// packet sent from this socket.
    Ipv4Ttl, Both, libc::IPPROTO_IP, libc::IP_TTL, libc::c_int);
#[cfg(any(target_os = "android", target_os = "freebsd", target_os = "linux"))]
sockopt_impl!(
    /// Set the unicast hop limit for the socket.
    Ipv6Ttl, Both, libc::IPPROTO_IPV6, libc::IPV6_UNICAST_HOPS, libc::c_int);

#[allow(missing_docs)]
// Not documented by Linux!
#[cfg(any(target_os = "android", target_os = "linux"))]
#[derive(Copy, Clone, Debug)]
pub struct AlgSetAeadAuthSize;

// ALG_SET_AEAD_AUTH_SIZE read the length from passed `option_len`
// See https://elixir.bootlin.com/linux/v4.4/source/crypto/af_alg.c#L222
#[cfg(any(target_os = "android", target_os = "linux"))]
impl SetSockOpt for AlgSetAeadAuthSize {
    type Val = usize;

    fn set(&self, fd: RawFd, val: &usize) -> Result<()> {
        unsafe {
            let res = libc::setsockopt(fd,
                                       libc::SOL_ALG,
                                       libc::ALG_SET_AEAD_AUTHSIZE,
                                       ::std::ptr::null(),
                                       *val as libc::socklen_t);
            Errno::result(res).map(drop)
        }
    }
}

#[allow(missing_docs)]
// Not documented by Linux!
#[cfg(any(target_os = "android", target_os = "linux"))]
#[derive(Clone, Debug)]
pub struct AlgSetKey<T>(::std::marker::PhantomData<T>);

#[cfg(any(target_os = "android", target_os = "linux"))]
impl<T> Default for AlgSetKey<T> {
    fn default() -> Self {
        AlgSetKey(Default::default())
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
impl<T> SetSockOpt for AlgSetKey<T> where T: AsRef<[u8]> + Clone {
    type Val = T;

    fn set(&self, fd: RawFd, val: &T) -> Result<()> {
        unsafe {
            let res = libc::setsockopt(fd,
                                       libc::SOL_ALG,
                                       libc::ALG_SET_KEY,
                                       val.as_ref().as_ptr() as *const _,
                                       val.as_ref().len() as libc::socklen_t);
            Errno::result(res).map(drop)
        }
    }
}

/*
 *
 * ===== Accessor helpers =====
 *
 */

/// Helper trait that describes what is expected from a `GetSockOpt` getter.
trait Get<T> {
    /// Returns an uninitialized value.
    fn uninit() -> Self;
    /// Returns a pointer to the stored value. This pointer will be passed to the system's
    /// `getsockopt` call (`man 3p getsockopt`, argument `option_value`).
    fn ffi_ptr(&mut self) -> *mut c_void;
    /// Returns length of the stored value. This pointer will be passed to the system's
    /// `getsockopt` call (`man 3p getsockopt`, argument `option_len`).
    fn ffi_len(&mut self) -> *mut socklen_t;
    /// Returns the hopefully initialized inner value.
    unsafe fn assume_init(self) -> T;
}

/// Helper trait that describes what is expected from a `SetSockOpt` setter.
trait Set<'a, T> {
    /// Initialize the setter with a given value.
    fn new(val: &'a T) -> Self;
    /// Returns a pointer to the stored value. This pointer will be passed to the system's
    /// `setsockopt` call (`man 3p setsockopt`, argument `option_value`).
    fn ffi_ptr(&self) -> *const c_void;
    /// Returns length of the stored value. This pointer will be passed to the system's
    /// `setsockopt` call (`man 3p setsockopt`, argument `option_len`).
    fn ffi_len(&self) -> socklen_t;
}

/// Getter for an arbitrary `struct`.
struct GetStruct<T> {
    len: socklen_t,
    val: MaybeUninit<T>,
}

impl<T> Get<T> for GetStruct<T> {
    fn uninit() -> Self {
        GetStruct {
            len: mem::size_of::<T>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    unsafe fn assume_init(self) -> T {
        assert_eq!(self.len as usize, mem::size_of::<T>(), "invalid getsockopt implementation");
        self.val.assume_init()
    }
}

/// Setter for an arbitrary `struct`.
struct SetStruct<'a, T: 'static> {
    ptr: &'a T,
}

impl<'a, T> Set<'a, T> for SetStruct<'a, T> {
    fn new(ptr: &'a T) -> SetStruct<'a, T> {
        SetStruct { ptr }
    }

    fn ffi_ptr(&self) -> *const c_void {
        self.ptr as *const T as *const c_void
    }

    fn ffi_len(&self) -> socklen_t {
        mem::size_of::<T>() as socklen_t
    }
}

/// Getter for a boolean value.
struct GetBool {
    len: socklen_t,
    val: MaybeUninit<c_int>,
}

impl Get<bool> for GetBool {
    fn uninit() -> Self {
        GetBool {
            len: mem::size_of::<c_int>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    unsafe fn assume_init(self) -> bool {
        assert_eq!(self.len as usize, mem::size_of::<c_int>(), "invalid getsockopt implementation");
        self.val.assume_init() != 0
    }
}

/// Setter for a boolean value.
struct SetBool {
    val: c_int,
}

impl<'a> Set<'a, bool> for SetBool {
    fn new(val: &'a bool) -> SetBool {
        SetBool { val: if *val { 1 } else { 0 } }
    }

    fn ffi_ptr(&self) -> *const c_void {
        &self.val as *const c_int as *const c_void
    }

    fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

/// Getter for an `u8` value.
struct GetU8 {
    len: socklen_t,
    val: MaybeUninit<u8>,
}

impl Get<u8> for GetU8 {
    fn uninit() -> Self {
        GetU8 {
            len: mem::size_of::<u8>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    unsafe fn assume_init(self) -> u8 {
        assert_eq!(self.len as usize, mem::size_of::<u8>(), "invalid getsockopt implementation");
        self.val.assume_init()
    }
}

/// Setter for an `u8` value.
struct SetU8 {
    val: u8,
}

impl<'a> Set<'a, u8> for SetU8 {
    fn new(val: &'a u8) -> SetU8 {
        SetU8 { val: *val as u8 }
    }

    fn ffi_ptr(&self) -> *const c_void {
        &self.val as *const u8 as *const c_void
    }

    fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

/// Getter for an `usize` value.
struct GetUsize {
    len: socklen_t,
    val: MaybeUninit<c_int>,
}

impl Get<usize> for GetUsize {
    fn uninit() -> Self {
        GetUsize {
            len: mem::size_of::<c_int>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    unsafe fn assume_init(self) -> usize {
        assert_eq!(self.len as usize, mem::size_of::<c_int>(), "invalid getsockopt implementation");
        self.val.assume_init() as usize
    }
}

/// Setter for an `usize` value.
struct SetUsize {
    val: c_int,
}

impl<'a> Set<'a, usize> for SetUsize {
    fn new(val: &'a usize) -> SetUsize {
        SetUsize { val: *val as c_int }
    }

    fn ffi_ptr(&self) -> *const c_void {
        &self.val as *const c_int as *const c_void
    }

    fn ffi_len(&self) -> socklen_t {
        mem::size_of::<c_int>() as socklen_t
    }
}

/// Getter for a `OsString` value.
struct GetOsString<T: AsMut<[u8]>> {
    len: socklen_t,
    val: MaybeUninit<T>,
}

impl<T: AsMut<[u8]>> Get<OsString> for GetOsString<T> {
    fn uninit() -> Self {
        GetOsString {
            len: mem::size_of::<T>() as socklen_t,
            val: MaybeUninit::uninit(),
        }
    }

    fn ffi_ptr(&mut self) -> *mut c_void {
        self.val.as_mut_ptr() as *mut c_void
    }

    fn ffi_len(&mut self) -> *mut socklen_t {
        &mut self.len
    }

    unsafe fn assume_init(self) -> OsString {
        let len = self.len as usize;
        let mut v = self.val.assume_init();
        OsStr::from_bytes(&v.as_mut()[0..len]).to_owned()
    }
}

/// Setter for a `OsString` value.
struct SetOsString<'a> {
    val: &'a OsStr,
}

impl<'a> Set<'a, OsString> for SetOsString<'a> {
    fn new(val: &'a OsString) -> SetOsString {
        SetOsString { val: val.as_os_str() }
    }

    fn ffi_ptr(&self) -> *const c_void {
        self.val.as_bytes().as_ptr() as *const c_void
    }

    fn ffi_len(&self) -> socklen_t {
        self.val.len() as socklen_t
    }
}


#[cfg(test)]
mod test {
    #[cfg(any(target_os = "android", target_os = "linux"))]
    #[test]
    fn can_get_peercred_on_unix_socket() {
        use super::super::*;

        let (a, b) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty()).unwrap();
        let a_cred = getsockopt(a, super::PeerCredentials).unwrap();
        let b_cred = getsockopt(b, super::PeerCredentials).unwrap();
        assert_eq!(a_cred, b_cred);
        assert!(a_cred.pid() != 0);
    }

    #[test]
    fn is_socket_type_unix() {
        use super::super::*;
        use crate::unistd::close;

        let (a, b) = socketpair(AddressFamily::Unix, SockType::Stream, None, SockFlag::empty()).unwrap();
        let a_type = getsockopt(a, super::SockType).unwrap();
        assert_eq!(a_type, SockType::Stream);
        close(a).unwrap();
        close(b).unwrap();
    }

    #[test]
    fn is_socket_type_dgram() {
        use super::super::*;
        use crate::unistd::close;

        let s = socket(AddressFamily::Inet, SockType::Datagram, SockFlag::empty(), None).unwrap();
        let s_type = getsockopt(s, super::SockType).unwrap();
        assert_eq!(s_type, SockType::Datagram);
        close(s).unwrap();
    }

    #[cfg(any(target_os = "freebsd",
              target_os = "linux",
              target_os = "nacl"))]
    #[test]
    fn can_get_listen_on_tcp_socket() {
        use super::super::*;
        use crate::unistd::close;

        let s = socket(AddressFamily::Inet, SockType::Stream, SockFlag::empty(), None).unwrap();
        let s_listening = getsockopt(s, super::AcceptConn).unwrap();
        assert!(!s_listening);
        listen(s, 10).unwrap();
        let s_listening2 = getsockopt(s, super::AcceptConn).unwrap();
        assert!(s_listening2);
        close(s).unwrap();
    }

}
