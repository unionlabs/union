//! Socket interface functions
//!
//! [Further reading](https://man7.org/linux/man-pages/man7/socket.7.html)
use cfg_if::cfg_if;
use crate::{Result, errno::Errno};
use libc::{self, c_void, c_int, iovec, socklen_t, size_t,
        CMSG_FIRSTHDR, CMSG_NXTHDR, CMSG_DATA, CMSG_LEN};
use memoffset::offset_of;
use std::{mem, ptr, slice};
use std::os::unix::io::RawFd;
#[cfg(all(target_os = "linux"))]
use crate::sys::time::TimeSpec;
use crate::sys::time::TimeVal;
use crate::sys::uio::IoVec;

mod addr;
#[deny(missing_docs)]
pub mod sockopt;

/*
 *
 * ===== Re-exports =====
 *
 */

#[cfg(not(any(target_os = "illumos", target_os = "solaris")))]
pub use self::addr::{
    AddressFamily,
    SockAddr,
    InetAddr,
    UnixAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
    LinkAddr,
};
#[cfg(any(target_os = "illumos", target_os = "solaris"))]
pub use self::addr::{
    AddressFamily,
    SockAddr,
    InetAddr,
    UnixAddr,
    IpAddr,
    Ipv4Addr,
    Ipv6Addr,
};

#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::sys::socket::addr::netlink::NetlinkAddr;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::sys::socket::addr::alg::AlgAddr;
#[cfg(any(target_os = "android", target_os = "linux"))]
pub use crate::sys::socket::addr::vsock::VsockAddr;

pub use libc::{
    cmsghdr,
    msghdr,
    sa_family_t,
    sockaddr,
    sockaddr_in,
    sockaddr_in6,
    sockaddr_storage,
    sockaddr_un,
};

// Needed by the cmsg_space macro
#[doc(hidden)]
pub use libc::{c_uint, CMSG_SPACE};

/// These constants are used to specify the communication semantics
/// when creating a socket with [`socket()`](fn.socket.html)
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(i32)]
#[non_exhaustive]
pub enum SockType {
    /// Provides sequenced, reliable, two-way, connection-
    /// based byte streams.  An out-of-band data transmission
    /// mechanism may be supported.
    Stream = libc::SOCK_STREAM,
    /// Supports datagrams (connectionless, unreliable
    /// messages of a fixed maximum length).
    Datagram = libc::SOCK_DGRAM,
    /// Provides a sequenced, reliable, two-way connection-
    /// based data transmission path for datagrams of fixed
    /// maximum length; a consumer is required to read an
    /// entire packet with each input system call.
    SeqPacket = libc::SOCK_SEQPACKET,
    /// Provides raw network protocol access.
    Raw = libc::SOCK_RAW,
    /// Provides a reliable datagram layer that does not
    /// guarantee ordering.
    Rdm = libc::SOCK_RDM,
}

/// Constants used in [`socket`](fn.socket.html) and [`socketpair`](fn.socketpair.html)
/// to specify the protocol to use.
#[repr(i32)]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
#[non_exhaustive]
pub enum SockProtocol {
    /// TCP protocol ([ip(7)](https://man7.org/linux/man-pages/man7/ip.7.html))
    Tcp = libc::IPPROTO_TCP,
    /// UDP protocol ([ip(7)](https://man7.org/linux/man-pages/man7/ip.7.html))
    Udp = libc::IPPROTO_UDP,
    /// Allows applications and other KEXTs to be notified when certain kernel events occur
    /// ([ref](https://developer.apple.com/library/content/documentation/Darwin/Conceptual/NKEConceptual/control/control.html))
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    KextEvent = libc::SYSPROTO_EVENT,
    /// Allows applications to configure and control a KEXT
    /// ([ref](https://developer.apple.com/library/content/documentation/Darwin/Conceptual/NKEConceptual/control/control.html))
    #[cfg(any(target_os = "ios", target_os = "macos"))]
    KextControl = libc::SYSPROTO_CONTROL,
    /// Receives routing and link updates and may be used to modify the routing tables (both IPv4 and IPv6), IP addresses, link
    // parameters, neighbor setups, queueing disciplines, traffic classes and packet classifiers
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkRoute = libc::NETLINK_ROUTE,
    /// Reserved for user-mode socket protocols
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkUserSock = libc::NETLINK_USERSOCK,
    /// Query information about sockets of various protocol families from the kernel
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSockDiag = libc::NETLINK_SOCK_DIAG,
    /// SELinux event notifications.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSELinux = libc::NETLINK_SELINUX,
    /// Open-iSCSI
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkISCSI = libc::NETLINK_ISCSI,
    /// Auditing
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkAudit = libc::NETLINK_AUDIT,
    /// Access to FIB lookup from user space
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkFIBLookup = libc::NETLINK_FIB_LOOKUP,
    /// Netfilter subsystem
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkNetFilter = libc::NETLINK_NETFILTER,
    /// SCSI Transports
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkSCSITransport = libc::NETLINK_SCSITRANSPORT,
    /// Infiniband RDMA
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkRDMA = libc::NETLINK_RDMA,
    /// Transport IPv6 packets from netfilter to user space.  Used by ip6_queue kernel module.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkIPv6Firewall = libc::NETLINK_IP6_FW,
    /// DECnet routing messages
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkDECNetRoutingMessage = libc::NETLINK_DNRTMSG,
    /// Kernel messages to user space
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkKObjectUEvent = libc::NETLINK_KOBJECT_UEVENT,
    /// Netlink interface to request information about ciphers registered with the kernel crypto API as well as allow
    /// configuration of the kernel crypto API.
    /// ([ref](https://www.man7.org/linux/man-pages/man7/netlink.7.html))
    #[cfg(any(target_os = "android", target_os = "linux"))]
    NetlinkCrypto = libc::NETLINK_CRYPTO,
}

libc_bitflags!{
    /// Additional socket options
    pub struct SockFlag: c_int {
        /// Set non-blocking mode on the new socket
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "illumos",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        SOCK_NONBLOCK;
        /// Set close-on-exec on the new descriptor
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "illumos",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        SOCK_CLOEXEC;
        /// Return `EPIPE` instead of raising `SIGPIPE`
        #[cfg(target_os = "netbsd")]
        SOCK_NOSIGPIPE;
        /// For domains `AF_INET(6)`, only allow `connect(2)`, `sendto(2)`, or `sendmsg(2)`
        /// to the DNS port (typically 53)
        #[cfg(target_os = "openbsd")]
        SOCK_DNS;
    }
}

libc_bitflags!{
    /// Flags for send/recv and their relatives
    pub struct MsgFlags: c_int {
        /// Sends or requests out-of-band data on sockets that support this notion
        /// (e.g., of type [`Stream`](enum.SockType.html)); the underlying protocol must also
        /// support out-of-band data.
        MSG_OOB;
        /// Peeks at an incoming message. The data is treated as unread and the next
        /// [`recv()`](fn.recv.html)
        /// or similar function shall still return this data.
        MSG_PEEK;
        /// Receive operation blocks until the full amount of data can be
        /// returned. The function may return smaller amount of data if a signal
        /// is caught, an error or disconnect occurs.
        MSG_WAITALL;
        /// Enables nonblocking operation; if the operation would block,
        /// `EAGAIN` or `EWOULDBLOCK` is returned.  This provides similar
        /// behavior to setting the `O_NONBLOCK` flag
        /// (via the [`fcntl`](../../fcntl/fn.fcntl.html)
        /// `F_SETFL` operation), but differs in that `MSG_DONTWAIT` is a per-
        /// call option, whereas `O_NONBLOCK` is a setting on the open file
        /// description (see [open(2)](https://man7.org/linux/man-pages/man2/open.2.html)),
        /// which will affect all threads in
        /// the calling process and as well as other processes that hold
        /// file descriptors referring to the same open file description.
        MSG_DONTWAIT;
        /// Receive flags: Control Data was discarded (buffer too small)
        MSG_CTRUNC;
        /// For raw ([`Packet`](addr/enum.AddressFamily.html)), Internet datagram
        /// (since Linux 2.4.27/2.6.8),
        /// netlink (since Linux 2.6.22) and UNIX datagram (since Linux 3.4)
        /// sockets: return the real length of the packet or datagram, even
        /// when it was longer than the passed buffer. Not implemented for UNIX
        /// domain ([unix(7)](https://linux.die.net/man/7/unix)) sockets.
        ///
        /// For use with Internet stream sockets, see [tcp(7)](https://linux.die.net/man/7/tcp).
        MSG_TRUNC;
        /// Terminates a record (when this notion is supported, as for
        /// sockets of type [`SeqPacket`](enum.SockType.html)).
        MSG_EOR;
        /// This flag specifies that queued errors should be received from
        /// the socket error queue. (For more details, see
        /// [recvfrom(2)](https://linux.die.net/man/2/recvfrom))
        #[cfg(any(target_os = "android", target_os = "linux"))]
        MSG_ERRQUEUE;
        /// Set the `close-on-exec` flag for the file descriptor received via a UNIX domain
        /// file descriptor using the `SCM_RIGHTS` operation (described in
        /// [unix(7)](https://linux.die.net/man/7/unix)).
        /// This flag is useful for the same reasons as the `O_CLOEXEC` flag of
        /// [open(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/open.html).
        ///
        /// Only used in [`recvmsg`](fn.recvmsg.html) function.
        #[cfg(any(target_os = "android",
                  target_os = "dragonfly",
                  target_os = "freebsd",
                  target_os = "linux",
                  target_os = "netbsd",
                  target_os = "openbsd"))]
        MSG_CMSG_CLOEXEC;
    }
}

cfg_if! {
    if #[cfg(any(target_os = "android", target_os = "linux"))] {
        /// Unix credentials of the sending process.
        ///
        /// This struct is used with the `SO_PEERCRED` ancillary message
        /// and the `SCM_CREDENTIALS` control message for UNIX sockets.
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct UnixCredentials(libc::ucred);

        impl UnixCredentials {
            /// Creates a new instance with the credentials of the current process
            pub fn new() -> Self {
                UnixCredentials(libc::ucred {
                    pid: crate::unistd::getpid().as_raw(),
                    uid: crate::unistd::getuid().as_raw(),
                    gid: crate::unistd::getgid().as_raw(),
                })
            }

            /// Returns the process identifier
            pub fn pid(&self) -> libc::pid_t {
                self.0.pid
            }

            /// Returns the user identifier
            pub fn uid(&self) -> libc::uid_t {
                self.0.uid
            }

            /// Returns the group identifier
            pub fn gid(&self) -> libc::gid_t {
                self.0.gid
            }
        }

        impl Default for UnixCredentials {
            fn default() -> Self {
                Self::new()
            }
        }

        impl From<libc::ucred> for UnixCredentials {
            fn from(cred: libc::ucred) -> Self {
                UnixCredentials(cred)
            }
        }

        impl From<UnixCredentials> for libc::ucred {
            fn from(uc: UnixCredentials) -> Self {
                uc.0
            }
        }
    } else if #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))] {
        /// Unix credentials of the sending process.
        ///
        /// This struct is used with the `SCM_CREDS` ancillary message for UNIX sockets.
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct UnixCredentials(libc::cmsgcred);

        impl UnixCredentials {
            /// Returns the process identifier
            pub fn pid(&self) -> libc::pid_t {
                self.0.cmcred_pid
            }

            /// Returns the real user identifier
            pub fn uid(&self) -> libc::uid_t {
                self.0.cmcred_uid
            }

            /// Returns the effective user identifier
            pub fn euid(&self) -> libc::uid_t {
                self.0.cmcred_euid
            }

            /// Returns the real group identifier
            pub fn gid(&self) -> libc::gid_t {
                self.0.cmcred_gid
            }

            /// Returns a list group identifiers (the first one being the effective GID)
            pub fn groups(&self) -> &[libc::gid_t] {
                unsafe { slice::from_raw_parts(self.0.cmcred_groups.as_ptr() as *const libc::gid_t, self.0.cmcred_ngroups as _) }
            }
        }

        impl From<libc::cmsgcred> for UnixCredentials {
            fn from(cred: libc::cmsgcred) -> Self {
                UnixCredentials(cred)
            }
        }
    }
}

cfg_if!{
    if #[cfg(any(
                target_os = "dragonfly",
                target_os = "freebsd",
                target_os = "macos",
                target_os = "ios"
        ))] {
        /// Return type of [`LocalPeerCred`](crate::sys::socket::sockopt::LocalPeerCred)
        #[repr(transparent)]
        #[derive(Clone, Copy, Debug, Eq, PartialEq)]
        pub struct XuCred(libc::xucred);

        impl XuCred {
            /// Structure layout version
            pub fn version(&self) -> u32 {
                self.0.cr_version
            }

            /// Effective user ID
            pub fn uid(&self) -> libc::uid_t {
                self.0.cr_uid
            }

            /// Returns a list of group identifiers (the first one being the
            /// effective GID)
            pub fn groups(&self) -> &[libc::gid_t] {
                &self.0.cr_groups
            }
        }
    }
}

/// Request for multicast socket operations
///
/// This is a wrapper type around `ip_mreq`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IpMembershipRequest(libc::ip_mreq);

impl IpMembershipRequest {
    /// Instantiate a new `IpMembershipRequest`
    ///
    /// If `interface` is `None`, then `Ipv4Addr::any()` will be used for the interface.
    pub fn new(group: Ipv4Addr, interface: Option<Ipv4Addr>) -> Self {
        IpMembershipRequest(libc::ip_mreq {
            imr_multiaddr: group.0,
            imr_interface: interface.unwrap_or_else(Ipv4Addr::any).0,
        })
    }
}

/// Request for ipv6 multicast socket operations
///
/// This is a wrapper type around `ipv6_mreq`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Ipv6MembershipRequest(libc::ipv6_mreq);

impl Ipv6MembershipRequest {
    /// Instantiate a new `Ipv6MembershipRequest`
    pub const fn new(group: Ipv6Addr) -> Self {
        Ipv6MembershipRequest(libc::ipv6_mreq {
            ipv6mr_multiaddr: group.0,
            ipv6mr_interface: 0,
        })
    }
}

/// Create a buffer large enough for storing some control messages as returned
/// by [`recvmsg`](fn.recvmsg.html).
///
/// # Examples
///
/// ```
/// # #[macro_use] extern crate nix;
/// # use nix::sys::time::TimeVal;
/// # use std::os::unix::io::RawFd;
/// # fn main() {
/// // Create a buffer for a `ControlMessageOwned::ScmTimestamp` message
/// let _ = cmsg_space!(TimeVal);
/// // Create a buffer big enough for a `ControlMessageOwned::ScmRights` message
/// // with two file descriptors
/// let _ = cmsg_space!([RawFd; 2]);
/// // Create a buffer big enough for a `ControlMessageOwned::ScmRights` message
/// // and a `ControlMessageOwned::ScmTimestamp` message
/// let _ = cmsg_space!(RawFd, TimeVal);
/// # }
/// ```
// Unfortunately, CMSG_SPACE isn't a const_fn, or else we could return a
// stack-allocated array.
#[macro_export]
macro_rules! cmsg_space {
    ( $( $x:ty ),* ) => {
        {
            let mut space = 0;
            $(
                // CMSG_SPACE is always safe
                space += unsafe {
                    $crate::sys::socket::CMSG_SPACE(::std::mem::size_of::<$x>() as $crate::sys::socket::c_uint)
                } as usize;
            )*
            Vec::<u8>::with_capacity(space)
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RecvMsg<'a> {
    pub bytes: usize,
    cmsghdr: Option<&'a cmsghdr>,
    pub address: Option<SockAddr>,
    pub flags: MsgFlags,
    mhdr: msghdr,
}

impl<'a> RecvMsg<'a> {
    /// Iterate over the valid control messages pointed to by this
    /// msghdr.
    pub fn cmsgs(&self) -> CmsgIterator {
        CmsgIterator {
            cmsghdr: self.cmsghdr,
            mhdr: &self.mhdr
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CmsgIterator<'a> {
    /// Control message buffer to decode from. Must adhere to cmsg alignment.
    cmsghdr: Option<&'a cmsghdr>,
    mhdr: &'a msghdr
}

impl<'a> Iterator for CmsgIterator<'a> {
    type Item = ControlMessageOwned;

    fn next(&mut self) -> Option<ControlMessageOwned> {
        match self.cmsghdr {
            None => None,   // No more messages
            Some(hdr) => {
                // Get the data.
                // Safe if cmsghdr points to valid data returned by recvmsg(2)
                let cm = unsafe { Some(ControlMessageOwned::decode_from(hdr))};
                // Advance the internal pointer.  Safe if mhdr and cmsghdr point
                // to valid data returned by recvmsg(2)
                self.cmsghdr = unsafe {
                    let p = CMSG_NXTHDR(self.mhdr as *const _, hdr as *const _);
                    p.as_ref()
                };
                cm
            }
        }
    }
}

/// A type-safe wrapper around a single control message, as used with
/// [`recvmsg`](#fn.recvmsg).
///
/// [Further reading](https://man7.org/linux/man-pages/man3/cmsg.3.html)
//  Nix version 0.13.0 and earlier used ControlMessage for both recvmsg and
//  sendmsg.  However, on some platforms the messages returned by recvmsg may be
//  unaligned.  ControlMessageOwned takes those messages by copy, obviating any
//  alignment issues.
//
//  See https://github.com/nix-rust/nix/issues/999
#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlMessageOwned {
    /// Received version of [`ControlMessage::ScmRights`]
    ScmRights(Vec<RawFd>),
    /// Received version of [`ControlMessage::ScmCredentials`]
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmCredentials(UnixCredentials),
    /// Received version of [`ControlMessage::ScmCreds`]
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ScmCreds(UnixCredentials),
    /// A message of type `SCM_TIMESTAMP`, containing the time the
    /// packet was received by the kernel.
    ///
    /// See the kernel's explanation in "SO_TIMESTAMP" of
    /// [networking/timestamping](https://www.kernel.org/doc/Documentation/networking/timestamping.txt).
    ///
    /// # Examples
    ///
    /// ```
    /// # #[macro_use] extern crate nix;
    /// # use nix::sys::socket::*;
    /// # use nix::sys::uio::IoVec;
    /// # use nix::sys::time::*;
    /// # use std::time::*;
    /// # fn main() {
    /// // Set up
    /// let message = "Ohayō!".as_bytes();
    /// let in_socket = socket(
    ///     AddressFamily::Inet,
    ///     SockType::Datagram,
    ///     SockFlag::empty(),
    ///     None).unwrap();
    /// setsockopt(in_socket, sockopt::ReceiveTimestamp, &true).unwrap();
    /// let localhost = InetAddr::new(IpAddr::new_v4(127, 0, 0, 1), 0);
    /// bind(in_socket, &SockAddr::new_inet(localhost)).unwrap();
    /// let address = getsockname(in_socket).unwrap();
    /// // Get initial time
    /// let time0 = SystemTime::now();
    /// // Send the message
    /// let iov = [IoVec::from_slice(message)];
    /// let flags = MsgFlags::empty();
    /// let l = sendmsg(in_socket, &iov, &[], flags, Some(&address)).unwrap();
    /// assert_eq!(message.len(), l);
    /// // Receive the message
    /// let mut buffer = vec![0u8; message.len()];
    /// let mut cmsgspace = cmsg_space!(TimeVal);
    /// let iov = [IoVec::from_mut_slice(&mut buffer)];
    /// let r = recvmsg(in_socket, &iov, Some(&mut cmsgspace), flags).unwrap();
    /// let rtime = match r.cmsgs().next() {
    ///     Some(ControlMessageOwned::ScmTimestamp(rtime)) => rtime,
    ///     Some(_) => panic!("Unexpected control message"),
    ///     None => panic!("No control message")
    /// };
    /// // Check the final time
    /// let time1 = SystemTime::now();
    /// // the packet's received timestamp should lie in-between the two system
    /// // times, unless the system clock was adjusted in the meantime.
    /// let rduration = Duration::new(rtime.tv_sec() as u64,
    ///                               rtime.tv_usec() as u32 * 1000);
    /// assert!(time0.duration_since(UNIX_EPOCH).unwrap() <= rduration);
    /// assert!(rduration <= time1.duration_since(UNIX_EPOCH).unwrap());
    /// // Close socket
    /// nix::unistd::close(in_socket).unwrap();
    /// # }
    /// ```
    ScmTimestamp(TimeVal),
    /// Nanoseconds resolution timestamp
    ///
    /// [Further reading](https://www.kernel.org/doc/html/latest/networking/timestamping.html)
    #[cfg(all(target_os = "linux"))]
    ScmTimestampns(TimeSpec),
    #[cfg(any(
        target_os = "android",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "netbsd",
    ))]
    Ipv4PacketInfo(libc::in_pktinfo),
    #[cfg(any(
        target_os = "android",
        target_os = "dragonfly",
        target_os = "freebsd",
        target_os = "ios",
        target_os = "linux",
        target_os = "macos",
        target_os = "openbsd",
        target_os = "netbsd",
    ))]
    Ipv6PacketInfo(libc::in6_pktinfo),
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    Ipv4RecvIf(libc::sockaddr_dl),
    #[cfg(any(
        target_os = "freebsd",
        target_os = "ios",
        target_os = "macos",
        target_os = "netbsd",
        target_os = "openbsd",
    ))]
    Ipv4RecvDstAddr(libc::in_addr),

    /// UDP Generic Receive Offload (GRO) allows receiving multiple UDP
    /// packets from a single sender.
    /// Fixed-size payloads are following one by one in a receive buffer.
    /// This Control Message indicates the size of all smaller packets,
    /// except, maybe, the last one.
    ///
    /// `UdpGroSegment` socket option should be enabled on a socket
    /// to allow receiving GRO packets.
    #[cfg(target_os = "linux")]
    UdpGroSegments(u16),

    /// SO_RXQ_OVFL indicates that an unsigned 32 bit value
    /// ancilliary msg (cmsg) should be attached to recieved
    /// skbs indicating the number of packets dropped by the
    /// socket between the last recieved packet and this
    /// received packet.
    ///
    /// `RxqOvfl` socket option should be enabled on a socket
    /// to allow receiving the drop counter.
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    RxqOvfl(u32),

    /// Socket error queue control messages read with the `MSG_ERRQUEUE` flag.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    Ipv4RecvErr(libc::sock_extended_err, Option<sockaddr_in>),
    /// Socket error queue control messages read with the `MSG_ERRQUEUE` flag.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    Ipv6RecvErr(libc::sock_extended_err, Option<sockaddr_in6>),

    /// Catch-all variant for unimplemented cmsg types.
    #[doc(hidden)]
    Unknown(UnknownCmsg),
}

impl ControlMessageOwned {
    /// Decodes a `ControlMessageOwned` from raw bytes.
    ///
    /// This is only safe to call if the data is correct for the message type
    /// specified in the header. Normally, the kernel ensures that this is the
    /// case. "Correct" in this case includes correct length, alignment and
    /// actual content.
    // Clippy complains about the pointer alignment of `p`, not understanding
    // that it's being fed to a function that can handle that.
    #[allow(clippy::cast_ptr_alignment)]
    unsafe fn decode_from(header: &cmsghdr) -> ControlMessageOwned
    {
        let p = CMSG_DATA(header);
        let len = header as *const _ as usize + header.cmsg_len as usize
            - p as usize;
        match (header.cmsg_level, header.cmsg_type) {
            (libc::SOL_SOCKET, libc::SCM_RIGHTS) => {
                let n = len / mem::size_of::<RawFd>();
                let mut fds = Vec::with_capacity(n);
                for i in 0..n {
                    let fdp = (p as *const RawFd).add(i);
                    fds.push(ptr::read_unaligned(fdp));
                }
                ControlMessageOwned::ScmRights(fds)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SCM_CREDENTIALS) => {
                let cred: libc::ucred = ptr::read_unaligned(p as *const _);
                ControlMessageOwned::ScmCredentials(cred.into())
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            (libc::SOL_SOCKET, libc::SCM_CREDS) => {
                let cred: libc::cmsgcred = ptr::read_unaligned(p as *const _);
                ControlMessageOwned::ScmCreds(cred.into())
            }
            (libc::SOL_SOCKET, libc::SCM_TIMESTAMP) => {
                let tv: libc::timeval = ptr::read_unaligned(p as *const _);
                ControlMessageOwned::ScmTimestamp(TimeVal::from(tv))
            },
            #[cfg(all(target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SCM_TIMESTAMPNS) => {
                let ts: libc::timespec = ptr::read_unaligned(p as *const _);
                ControlMessageOwned::ScmTimestampns(TimeSpec::from(ts))
            }
            #[cfg(any(
                target_os = "android",
                target_os = "freebsd",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos"
            ))]
            (libc::IPPROTO_IPV6, libc::IPV6_PKTINFO) => {
                let info = ptr::read_unaligned(p as *const libc::in6_pktinfo);
                ControlMessageOwned::Ipv6PacketInfo(info)
            }
            #[cfg(any(
                target_os = "android",
                target_os = "ios",
                target_os = "linux",
                target_os = "macos",
                target_os = "netbsd",
            ))]
            (libc::IPPROTO_IP, libc::IP_PKTINFO) => {
                let info = ptr::read_unaligned(p as *const libc::in_pktinfo);
                ControlMessageOwned::Ipv4PacketInfo(info)
            }
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            (libc::IPPROTO_IP, libc::IP_RECVIF) => {
                let dl = ptr::read_unaligned(p as *const libc::sockaddr_dl);
                ControlMessageOwned::Ipv4RecvIf(dl)
            },
            #[cfg(any(
                target_os = "freebsd",
                target_os = "ios",
                target_os = "macos",
                target_os = "netbsd",
                target_os = "openbsd",
            ))]
            (libc::IPPROTO_IP, libc::IP_RECVDSTADDR) => {
                let dl = ptr::read_unaligned(p as *const libc::in_addr);
                ControlMessageOwned::Ipv4RecvDstAddr(dl)
            },
            #[cfg(target_os = "linux")]
            (libc::SOL_UDP, libc::UDP_GRO) => {
                let gso_size: u16 = ptr::read_unaligned(p as *const _);
                ControlMessageOwned::UdpGroSegments(gso_size)
            },
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            (libc::SOL_SOCKET, libc::SO_RXQ_OVFL) => {
                let drop_counter = ptr::read_unaligned(p as *const u32);
                ControlMessageOwned::RxqOvfl(drop_counter)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::IPPROTO_IP, libc::IP_RECVERR) => {
                let (err, addr) = Self::recv_err_helper::<sockaddr_in>(p, len);
                ControlMessageOwned::Ipv4RecvErr(err, addr)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            (libc::IPPROTO_IPV6, libc::IPV6_RECVERR) => {
                let (err, addr) = Self::recv_err_helper::<sockaddr_in6>(p, len);
                ControlMessageOwned::Ipv6RecvErr(err, addr)
            },
            (_, _) => {
                let sl = slice::from_raw_parts(p, len);
                let ucmsg = UnknownCmsg(*header, Vec::<u8>::from(sl));
                ControlMessageOwned::Unknown(ucmsg)
            }
        }
    }

    #[cfg(any(target_os = "android", target_os = "linux"))]
    unsafe fn recv_err_helper<T>(p: *mut libc::c_uchar, len: usize) -> (libc::sock_extended_err, Option<T>) {
        let ee = p as *const libc::sock_extended_err;
        let err = ptr::read_unaligned(ee);

        // For errors originating on the network, SO_EE_OFFENDER(ee) points inside the p[..len]
        // CMSG_DATA buffer.  For local errors, there is no address included in the control
        // message, and SO_EE_OFFENDER(ee) points beyond the end of the buffer.  So, we need to
        // validate that the address object is in-bounds before we attempt to copy it.
        let addrp = libc::SO_EE_OFFENDER(ee) as *const T;

        if addrp.offset(1) as usize - (p as usize) > len {
            (err, None)
        } else {
            (err, Some(ptr::read_unaligned(addrp)))
        }
    }
}

/// A type-safe zero-copy wrapper around a single control message, as used wih
/// [`sendmsg`](#fn.sendmsg).  More types may be added to this enum; do not
/// exhaustively pattern-match it.
///
/// [Further reading](https://man7.org/linux/man-pages/man3/cmsg.3.html)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ControlMessage<'a> {
    /// A message of type `SCM_RIGHTS`, containing an array of file
    /// descriptors passed between processes.
    ///
    /// See the description in the "Ancillary messages" section of the
    /// [unix(7) man page](https://man7.org/linux/man-pages/man7/unix.7.html).
    ///
    /// Using multiple `ScmRights` messages for a single `sendmsg` call isn't
    /// recommended since it causes platform-dependent behaviour: It might
    /// swallow all but the first `ScmRights` message or fail with `EINVAL`.
    /// Instead, you can put all fds to be passed into a single `ScmRights`
    /// message.
    ScmRights(&'a [RawFd]),
    /// A message of type `SCM_CREDENTIALS`, containing the pid, uid and gid of
    /// a process connected to the socket.
    ///
    /// This is similar to the socket option `SO_PEERCRED`, but requires a
    /// process to explicitly send its credentials. A process running as root is
    /// allowed to specify any credentials, while credentials sent by other
    /// processes are verified by the kernel.
    ///
    /// For further information, please refer to the
    /// [`unix(7)`](https://man7.org/linux/man-pages/man7/unix.7.html) man page.
    #[cfg(any(target_os = "android", target_os = "linux"))]
    ScmCredentials(&'a UnixCredentials),
    /// A message of type `SCM_CREDS`, containing the pid, uid, euid, gid and groups of
    /// a process connected to the socket.
    ///
    /// This is similar to the socket options `LOCAL_CREDS` and `LOCAL_PEERCRED`, but
    /// requires a process to explicitly send its credentials.
    ///
    /// Credentials are always overwritten by the kernel, so this variant does have
    /// any data, unlike the receive-side
    /// [`ControlMessageOwned::ScmCreds`].
    ///
    /// For further information, please refer to the
    /// [`unix(4)`](https://www.freebsd.org/cgi/man.cgi?query=unix) man page.
    #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
    ScmCreds,

    /// Set IV for `AF_ALG` crypto API.
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://kernel.readthedocs.io/en/sphinx-samples/crypto-API.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetIv(&'a [u8]),
    /// Set crypto operation for `AF_ALG` crypto API. It may be one of
    /// `ALG_OP_ENCRYPT` or `ALG_OP_DECRYPT`
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://kernel.readthedocs.io/en/sphinx-samples/crypto-API.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetOp(&'a libc::c_int),
    /// Set the length of associated authentication data (AAD) (applicable only to AEAD algorithms)
    /// for `AF_ALG` crypto API.
    ///
    /// For further information, please refer to the
    /// [`documentation`](https://kernel.readthedocs.io/en/sphinx-samples/crypto-API.html)
    #[cfg(any(
        target_os = "android",
        target_os = "linux",
    ))]
    AlgSetAeadAssoclen(&'a u32),

    /// UDP GSO makes it possible for applications to generate network packets
    /// for a virtual MTU much greater than the real one.
    /// The length of the send data no longer matches the expected length on
    /// the wire.
    /// The size of the datagram payload as it should appear on the wire may be
    /// passed through this control message.
    /// Send buffer should consist of multiple fixed-size wire payloads
    /// following one by one, and the last, possibly smaller one.
    #[cfg(target_os = "linux")]
    UdpGsoSegments(&'a u16),

    /// Configure the sending addressing and interface for v4
    ///
    /// For further information, please refer to the
    /// [`ip(7)`](https://man7.org/linux/man-pages/man7/ip.7.html) man page.
    #[cfg(any(target_os = "linux",
              target_os = "macos",
              target_os = "netbsd",
              target_os = "android",
              target_os = "ios",))]
    Ipv4PacketInfo(&'a libc::in_pktinfo),

    /// Configure the sending addressing and interface for v6
    ///
    /// For further information, please refer to the
    /// [`ipv6(7)`](https://man7.org/linux/man-pages/man7/ipv6.7.html) man page.
    #[cfg(any(target_os = "linux",
              target_os = "macos",
              target_os = "netbsd",
              target_os = "freebsd",
              target_os = "android",
              target_os = "ios",))]
    Ipv6PacketInfo(&'a libc::in6_pktinfo),

    /// SO_RXQ_OVFL indicates that an unsigned 32 bit value
    /// ancilliary msg (cmsg) should be attached to recieved
    /// skbs indicating the number of packets dropped by the
    /// socket between the last recieved packet and this
    /// received packet.
    #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
    RxqOvfl(&'a u32),
}

// An opaque structure used to prevent cmsghdr from being a public type
#[doc(hidden)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UnknownCmsg(cmsghdr, Vec<u8>);

impl<'a> ControlMessage<'a> {
    /// The value of CMSG_SPACE on this message.
    /// Safe because CMSG_SPACE is always safe
    fn space(&self) -> usize {
        unsafe{CMSG_SPACE(self.len() as libc::c_uint) as usize}
    }

    /// The value of CMSG_LEN on this message.
    /// Safe because CMSG_LEN is always safe
    #[cfg(any(target_os = "android",
              all(target_os = "linux", not(target_env = "musl"))))]
    fn cmsg_len(&self) -> usize {
        unsafe{CMSG_LEN(self.len() as libc::c_uint) as usize}
    }

    #[cfg(not(any(target_os = "android",
                  all(target_os = "linux", not(target_env = "musl")))))]
    fn cmsg_len(&self) -> libc::c_uint {
        unsafe{CMSG_LEN(self.len() as libc::c_uint)}
    }

    /// Return a reference to the payload data as a byte pointer
    fn copy_to_cmsg_data(&self, cmsg_data: *mut u8) {
        let data_ptr = match *self {
            ControlMessage::ScmRights(fds) => {
                fds as *const _ as *const u8
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(creds) => {
                &creds.0 as *const libc::ucred as *const u8
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => {
                // The kernel overwrites the data, we just zero it
                // to make sure it's not uninitialized memory
                unsafe { ptr::write_bytes(cmsg_data, 0, self.len()) };
                return
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(iv) => {
                #[allow(deprecated)] // https://github.com/rust-lang/libc/issues/1501
                let af_alg_iv = libc::af_alg_iv {
                    ivlen: iv.len() as u32,
                    iv: [0u8; 0],
                };

                let size = mem::size_of_val(&af_alg_iv);

                unsafe {
                    ptr::copy_nonoverlapping(
                        &af_alg_iv as *const _ as *const u8,
                        cmsg_data,
                        size,
                    );
                    ptr::copy_nonoverlapping(
                        iv.as_ptr(),
                        cmsg_data.add(size),
                        iv.len()
                    );
                };

                return
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(op) => {
                op as *const _ as *const u8
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(len) => {
                len as *const _ as *const u8
            },
            #[cfg(target_os = "linux")]
            ControlMessage::UdpGsoSegments(gso_size) => {
                gso_size as *const _ as *const u8
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            ControlMessage::Ipv4PacketInfo(info) => info as *const _ as *const u8,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "freebsd",
                      target_os = "android", target_os = "ios",))]
            ControlMessage::Ipv6PacketInfo(info) => info as *const _ as *const u8,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(drop_count) => {
                drop_count as *const _ as *const u8
            },
        };
        unsafe {
            ptr::copy_nonoverlapping(
                data_ptr,
                cmsg_data,
                self.len()
            )
        };
    }

    /// The size of the payload, excluding its cmsghdr
    fn len(&self) -> usize {
        match *self {
            ControlMessage::ScmRights(fds) => {
                mem::size_of_val(fds)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(creds) => {
                mem::size_of_val(creds)
            }
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => {
                mem::size_of::<libc::cmsgcred>()
            }
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(iv) => {
                mem::size_of_val(&iv) + iv.len()
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(op) => {
                mem::size_of_val(op)
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(len) => {
                mem::size_of_val(len)
            },
            #[cfg(target_os = "linux")]
            ControlMessage::UdpGsoSegments(gso_size) => {
                mem::size_of_val(gso_size)
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "android",
              target_os = "ios",))]
            ControlMessage::Ipv4PacketInfo(info) => mem::size_of_val(info),
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "freebsd",
              target_os = "android", target_os = "ios",))]
            ControlMessage::Ipv6PacketInfo(info) => mem::size_of_val(info),
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(drop_count) => {
                mem::size_of_val(drop_count)
            },
        }
    }

    /// Returns the value to put into the `cmsg_level` field of the header.
    fn cmsg_level(&self) -> libc::c_int {
        match *self {
            ControlMessage::ScmRights(_) => libc::SOL_SOCKET,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(_) => libc::SOL_SOCKET,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => libc::SOL_SOCKET,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(_) | ControlMessage::AlgSetOp(_) |
                ControlMessage::AlgSetAeadAssoclen(_) => libc::SOL_ALG,
            #[cfg(target_os = "linux")]
            ControlMessage::UdpGsoSegments(_) => libc::SOL_UDP,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            ControlMessage::Ipv4PacketInfo(_) => libc::IPPROTO_IP,
            #[cfg(any(target_os = "linux", target_os = "macos",
              target_os = "netbsd", target_os = "freebsd",
              target_os = "android", target_os = "ios",))]
            ControlMessage::Ipv6PacketInfo(_) => libc::IPPROTO_IPV6,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(_) => libc::SOL_SOCKET,
        }
    }

    /// Returns the value to put into the `cmsg_type` field of the header.
    fn cmsg_type(&self) -> libc::c_int {
        match *self {
            ControlMessage::ScmRights(_) => libc::SCM_RIGHTS,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::ScmCredentials(_) => libc::SCM_CREDENTIALS,
            #[cfg(any(target_os = "freebsd", target_os = "dragonfly"))]
            ControlMessage::ScmCreds => libc::SCM_CREDS,
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetIv(_) => {
                libc::ALG_SET_IV
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetOp(_) => {
                libc::ALG_SET_OP
            },
            #[cfg(any(target_os = "android", target_os = "linux"))]
            ControlMessage::AlgSetAeadAssoclen(_) => {
                libc::ALG_SET_AEAD_ASSOCLEN
            },
            #[cfg(target_os = "linux")]
            ControlMessage::UdpGsoSegments(_) => {
                libc::UDP_SEGMENT
            },
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "android",
                      target_os = "ios",))]
            ControlMessage::Ipv4PacketInfo(_) => libc::IP_PKTINFO,
            #[cfg(any(target_os = "linux", target_os = "macos",
                      target_os = "netbsd", target_os = "freebsd",
                      target_os = "android", target_os = "ios",))]
            ControlMessage::Ipv6PacketInfo(_) => libc::IPV6_PKTINFO,
            #[cfg(any(target_os = "android", target_os = "fuchsia", target_os = "linux"))]
            ControlMessage::RxqOvfl(_) => {
                libc::SO_RXQ_OVFL
            },
        }
    }

    // Unsafe: cmsg must point to a valid cmsghdr with enough space to
    // encode self.
    unsafe fn encode_into(&self, cmsg: *mut cmsghdr) {
        (*cmsg).cmsg_level = self.cmsg_level();
        (*cmsg).cmsg_type = self.cmsg_type();
        (*cmsg).cmsg_len = self.cmsg_len();
        self.copy_to_cmsg_data(CMSG_DATA(cmsg));
    }
}


/// Send data in scatter-gather vectors to a socket, possibly accompanied
/// by ancillary data. Optionally direct the message at the given address,
/// as with sendto.
///
/// Allocates if cmsgs is nonempty.
pub fn sendmsg(fd: RawFd, iov: &[IoVec<&[u8]>], cmsgs: &[ControlMessage],
               flags: MsgFlags, addr: Option<&SockAddr>) -> Result<usize>
{
    let capacity = cmsgs.iter().map(|c| c.space()).sum();

    // First size the buffer needed to hold the cmsgs.  It must be zeroed,
    // because subsequent code will not clear the padding bytes.
    let mut cmsg_buffer = vec![0u8; capacity];

    let mhdr = pack_mhdr_to_send(&mut cmsg_buffer[..], &iov, &cmsgs, addr);

    let ret = unsafe { libc::sendmsg(fd, &mhdr, flags.bits()) };

    Errno::result(ret).map(|r| r as usize)
}

#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug)]
pub struct SendMmsgData<'a, I, C>
    where
        I: AsRef<[IoVec<&'a [u8]>]>,
        C: AsRef<[ControlMessage<'a>]>
{
    pub iov: I,
    pub cmsgs: C,
    pub addr: Option<SockAddr>,
    pub _lt: std::marker::PhantomData<&'a I>,
}

/// An extension of `sendmsg` that allows the caller to transmit multiple
/// messages on a socket using a single system call. This has performance
/// benefits for some applications.
///
/// Allocations are performed for cmsgs and to build `msghdr` buffer
///
/// # Arguments
///
/// * `fd`:             Socket file descriptor
/// * `data`:           Struct that implements `IntoIterator` with `SendMmsgData` items
/// * `flags`:          Optional flags passed directly to the operating system.
///
/// # Returns
/// `Vec` with numbers of sent bytes on each sent message.
///
/// # References
/// [`sendmsg`](fn.sendmsg.html)
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
pub fn sendmmsg<'a, I, C>(
    fd: RawFd,
    data: impl std::iter::IntoIterator<Item=&'a SendMmsgData<'a, I, C>>,
    flags: MsgFlags
) -> Result<Vec<usize>>
    where
        I: AsRef<[IoVec<&'a [u8]>]> + 'a,
        C: AsRef<[ControlMessage<'a>]> + 'a,
{
    let iter = data.into_iter();

    let size_hint = iter.size_hint();
    let reserve_items = size_hint.1.unwrap_or(size_hint.0);

    let mut output = Vec::<libc::mmsghdr>::with_capacity(reserve_items);

    let mut cmsgs_buffers = Vec::<Vec<u8>>::with_capacity(reserve_items);

    for d in iter {
        let capacity: usize = d.cmsgs.as_ref().iter().map(|c| c.space()).sum();
        let mut cmsgs_buffer = vec![0u8; capacity];

        output.push(libc::mmsghdr {
            msg_hdr: pack_mhdr_to_send(
                &mut cmsgs_buffer,
                &d.iov,
                &d.cmsgs,
                d.addr.as_ref()
            ),
            msg_len: 0,
        });
        cmsgs_buffers.push(cmsgs_buffer);
    };

    let ret = unsafe { libc::sendmmsg(fd, output.as_mut_ptr(), output.len() as _, flags.bits() as _) };

    let sent_messages = Errno::result(ret)? as usize;
    let mut sent_bytes = Vec::with_capacity(sent_messages);

    for item in &output {
        sent_bytes.push(item.msg_len as usize);
    }

    Ok(sent_bytes)
}


#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[derive(Debug)]
pub struct RecvMmsgData<'a, I>
    where
        I: AsRef<[IoVec<&'a mut [u8]>]> + 'a,
{
    pub iov: I,
    pub cmsg_buffer: Option<&'a mut Vec<u8>>,
}

/// An extension of `recvmsg` that allows the caller to receive multiple
/// messages from a socket using a single system call. This has
/// performance benefits for some applications.
///
/// `iov` and `cmsg_buffer` should be constructed similarly to `recvmsg`
///
/// Multiple allocations are performed
///
/// # Arguments
///
/// * `fd`:             Socket file descriptor
/// * `data`:           Struct that implements `IntoIterator` with `RecvMmsgData` items
/// * `flags`:          Optional flags passed directly to the operating system.
///
/// # RecvMmsgData
///
/// * `iov`:            Scatter-gather list of buffers to receive the message
/// * `cmsg_buffer`:    Space to receive ancillary data.  Should be created by
///                     [`cmsg_space!`](macro.cmsg_space.html)
///
/// # Returns
/// A `Vec` with multiple `RecvMsg`, one per received message
///
/// # References
/// - [`recvmsg`](fn.recvmsg.html)
/// - [`RecvMsg`](struct.RecvMsg.html)
#[cfg(any(
    target_os = "linux",
    target_os = "android",
    target_os = "freebsd",
    target_os = "netbsd",
))]
#[allow(clippy::needless_collect)]  // Complicated false positive
pub fn recvmmsg<'a, I>(
    fd: RawFd,
    data: impl std::iter::IntoIterator<Item=&'a mut RecvMmsgData<'a, I>,
        IntoIter=impl ExactSizeIterator + Iterator<Item=&'a mut RecvMmsgData<'a, I>>>,
    flags: MsgFlags,
    timeout: Option<crate::sys::time::TimeSpec>
) -> Result<Vec<RecvMsg<'a>>>
    where
        I: AsRef<[IoVec<&'a mut [u8]>]> + 'a,
{
    let iter = data.into_iter();

    let num_messages = iter.len();

    let mut output: Vec<libc::mmsghdr> = Vec::with_capacity(num_messages);

    // Addresses should be pre-allocated.  pack_mhdr_to_receive will store them
    // as raw pointers, so we may not move them.  Turn the vec into a boxed
    // slice so we won't inadvertently reallocate the vec.
    let mut addresses = vec![mem::MaybeUninit::uninit(); num_messages]
        .into_boxed_slice();

    let results: Vec<_> = iter.enumerate().map(|(i, d)| {
        let (msg_controllen, mhdr) = unsafe {
            pack_mhdr_to_receive(
                d.iov.as_ref(),
                &mut d.cmsg_buffer,
                addresses[i].as_mut_ptr(),
            )
        };

        output.push(
            libc::mmsghdr {
                msg_hdr: mhdr,
                msg_len: 0,
            }
        );

        (msg_controllen as usize, &mut d.cmsg_buffer)
    }).collect();

    let timeout = if let Some(mut t) = timeout {
        t.as_mut() as *mut libc::timespec
    } else {
        ptr::null_mut()
    };

    let ret = unsafe { libc::recvmmsg(fd, output.as_mut_ptr(), output.len() as _, flags.bits() as _, timeout) };

    let _ = Errno::result(ret)?;

    Ok(output
        .into_iter()
        .take(ret as usize)
        .zip(addresses.iter().map(|addr| unsafe{addr.assume_init()}))
        .zip(results.into_iter())
        .map(|((mmsghdr, address), (msg_controllen, cmsg_buffer))| {
            unsafe {
                read_mhdr(
                    mmsghdr.msg_hdr,
                    mmsghdr.msg_len as isize,
                    msg_controllen,
                    address,
                    cmsg_buffer
                )
            }
        })
        .collect())
}

unsafe fn read_mhdr<'a, 'b>(
    mhdr: msghdr,
    r: isize,
    msg_controllen: usize,
    address: sockaddr_storage,
    cmsg_buffer: &'a mut Option<&'b mut Vec<u8>>
) -> RecvMsg<'b> {
    let cmsghdr = {
        if mhdr.msg_controllen > 0 {
            // got control message(s)
            cmsg_buffer
                .as_mut()
                .unwrap()
                .set_len(mhdr.msg_controllen as usize);
            debug_assert!(!mhdr.msg_control.is_null());
            debug_assert!(msg_controllen >= mhdr.msg_controllen as usize);
            CMSG_FIRSTHDR(&mhdr as *const msghdr)
        } else {
            ptr::null()
        }.as_ref()
    };

    let address = sockaddr_storage_to_addr(
        &address ,
         mhdr.msg_namelen as usize
    ).ok();

    RecvMsg {
        bytes: r as usize,
        cmsghdr,
        address,
        flags: MsgFlags::from_bits_truncate(mhdr.msg_flags),
        mhdr,
    }
}

unsafe fn pack_mhdr_to_receive<'a, I>(
    iov: I,
    cmsg_buffer: &mut Option<&mut Vec<u8>>,
    address: *mut sockaddr_storage,
) -> (usize, msghdr)
    where
        I: AsRef<[IoVec<&'a mut [u8]>]> + 'a,
{
    let (msg_control, msg_controllen) = cmsg_buffer.as_mut()
        .map(|v| (v.as_mut_ptr(), v.capacity()))
        .unwrap_or((ptr::null_mut(), 0));

    let mhdr = {
        // Musl's msghdr has private fields, so this is the only way to
        // initialize it.
        let mut mhdr = mem::MaybeUninit::<msghdr>::zeroed();
        let p = mhdr.as_mut_ptr();
        (*p).msg_name = address as *mut c_void;
        (*p).msg_namelen = mem::size_of::<sockaddr_storage>() as socklen_t;
        (*p).msg_iov = iov.as_ref().as_ptr() as *mut iovec;
        (*p).msg_iovlen = iov.as_ref().len() as _;
        (*p).msg_control = msg_control as *mut c_void;
        (*p).msg_controllen = msg_controllen as _;
        (*p).msg_flags = 0;
        mhdr.assume_init()
    };

    (msg_controllen, mhdr)
}

fn pack_mhdr_to_send<'a, I, C>(
    cmsg_buffer: &mut [u8],
    iov: I,
    cmsgs: C,
    addr: Option<&SockAddr>
) -> msghdr
    where
        I: AsRef<[IoVec<&'a [u8]>]>,
        C: AsRef<[ControlMessage<'a>]>
{
    let capacity = cmsg_buffer.len();

    // Next encode the sending address, if provided
    let (name, namelen) = match addr {
        Some(addr) => {
            let (x, y) = addr.as_ffi_pair();
            (x as *const _, y)
        },
        None => (ptr::null(), 0),
    };

    // The message header must be initialized before the individual cmsgs.
    let cmsg_ptr = if capacity > 0 {
        cmsg_buffer.as_ptr() as *mut c_void
    } else {
        ptr::null_mut()
    };

    let mhdr = unsafe {
        // Musl's msghdr has private fields, so this is the only way to
        // initialize it.
        let mut mhdr = mem::MaybeUninit::<msghdr>::zeroed();
        let p = mhdr.as_mut_ptr();
        (*p).msg_name = name as *mut _;
        (*p).msg_namelen = namelen;
        // transmute iov into a mutable pointer.  sendmsg doesn't really mutate
        // the buffer, but the standard says that it takes a mutable pointer
        (*p).msg_iov = iov.as_ref().as_ptr() as *mut _;
        (*p).msg_iovlen = iov.as_ref().len() as _;
        (*p).msg_control = cmsg_ptr;
        (*p).msg_controllen = capacity as _;
        (*p).msg_flags = 0;
        mhdr.assume_init()
    };

    // Encode each cmsg.  This must happen after initializing the header because
    // CMSG_NEXT_HDR and friends read the msg_control and msg_controllen fields.
    // CMSG_FIRSTHDR is always safe
    let mut pmhdr: *mut cmsghdr = unsafe { CMSG_FIRSTHDR(&mhdr as *const msghdr) };
    for cmsg in cmsgs.as_ref() {
        assert_ne!(pmhdr, ptr::null_mut());
        // Safe because we know that pmhdr is valid, and we initialized it with
        // sufficient space
        unsafe { cmsg.encode_into(pmhdr) };
        // Safe because mhdr is valid
        pmhdr = unsafe { CMSG_NXTHDR(&mhdr as *const msghdr, pmhdr) };
    }

    mhdr
}

/// Receive message in scatter-gather vectors from a socket, and
/// optionally receive ancillary data into the provided buffer.
/// If no ancillary data is desired, use () as the type parameter.
///
/// # Arguments
///
/// * `fd`:             Socket file descriptor
/// * `iov`:            Scatter-gather list of buffers to receive the message
/// * `cmsg_buffer`:    Space to receive ancillary data.  Should be created by
///                     [`cmsg_space!`](macro.cmsg_space.html)
/// * `flags`:          Optional flags passed directly to the operating system.
///
/// # References
/// [recvmsg(2)](https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvmsg.html)
pub fn recvmsg<'a>(fd: RawFd, iov: &[IoVec<&mut [u8]>],
                   mut cmsg_buffer: Option<&'a mut Vec<u8>>,
                   flags: MsgFlags) -> Result<RecvMsg<'a>>
{
    let mut address = mem::MaybeUninit::uninit();

    let (msg_controllen, mut mhdr) = unsafe {
        pack_mhdr_to_receive(&iov, &mut cmsg_buffer, address.as_mut_ptr())
    };

    let ret = unsafe { libc::recvmsg(fd, &mut mhdr, flags.bits()) };

    let r = Errno::result(ret)?;

    Ok(unsafe { read_mhdr(mhdr, r, msg_controllen, address.assume_init(), &mut cmsg_buffer) })
}


/// Create an endpoint for communication
///
/// The `protocol` specifies a particular protocol to be used with the
/// socket.  Normally only a single protocol exists to support a
/// particular socket type within a given protocol family, in which case
/// protocol can be specified as `None`.  However, it is possible that many
/// protocols may exist, in which case a particular protocol must be
/// specified in this manner.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/socket.html)
pub fn socket<T: Into<Option<SockProtocol>>>(domain: AddressFamily, ty: SockType, flags: SockFlag, protocol: T) -> Result<RawFd> {
    let protocol = match protocol.into() {
        None => 0,
        Some(p) => p as c_int,
    };

    // SockFlags are usually embedded into `ty`, but we don't do that in `nix` because it's a
    // little easier to understand by separating it out. So we have to merge these bitfields
    // here.
    let mut ty = ty as c_int;
    ty |= flags.bits();

    let res = unsafe { libc::socket(domain as c_int, ty, protocol) };

    Errno::result(res)
}

/// Create a pair of connected sockets
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/socketpair.html)
pub fn socketpair<T: Into<Option<SockProtocol>>>(domain: AddressFamily, ty: SockType, protocol: T,
                  flags: SockFlag) -> Result<(RawFd, RawFd)> {
    let protocol = match protocol.into() {
        None => 0,
        Some(p) => p as c_int,
    };

    // SockFlags are usually embedded into `ty`, but we don't do that in `nix` because it's a
    // little easier to understand by separating it out. So we have to merge these bitfields
    // here.
    let mut ty = ty as c_int;
    ty |= flags.bits();

    let mut fds = [-1, -1];

    let res = unsafe { libc::socketpair(domain as c_int, ty, protocol, fds.as_mut_ptr()) };
    Errno::result(res)?;

    Ok((fds[0], fds[1]))
}

/// Listen for connections on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/listen.html)
pub fn listen(sockfd: RawFd, backlog: usize) -> Result<()> {
    let res = unsafe { libc::listen(sockfd, backlog as c_int) };

    Errno::result(res).map(drop)
}

/// Bind a name to a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/bind.html)
pub fn bind(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        libc::bind(fd, ptr, len)
    };

    Errno::result(res).map(drop)
}

/// Accept a connection on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/accept.html)
pub fn accept(sockfd: RawFd) -> Result<RawFd> {
    let res = unsafe { libc::accept(sockfd, ptr::null_mut(), ptr::null_mut()) };

    Errno::result(res)
}

/// Accept a connection on a socket
///
/// [Further reading](https://man7.org/linux/man-pages/man2/accept.2.html)
#[cfg(any(all(
            target_os = "android",
            any(
                target_arch = "aarch64",
                target_arch = "x86",
                target_arch = "x86_64"
            )
          ),
          target_os = "freebsd",
          target_os = "linux",
          target_os = "openbsd"))]
pub fn accept4(sockfd: RawFd, flags: SockFlag) -> Result<RawFd> {
    let res = unsafe { libc::accept4(sockfd, ptr::null_mut(), ptr::null_mut(), flags.bits()) };

    Errno::result(res)
}

/// Initiate a connection on a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html)
pub fn connect(fd: RawFd, addr: &SockAddr) -> Result<()> {
    let res = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        libc::connect(fd, ptr, len)
    };

    Errno::result(res).map(drop)
}

/// Receive data from a connection-oriented socket. Returns the number of
/// bytes read
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/recv.html)
pub fn recv(sockfd: RawFd, buf: &mut [u8], flags: MsgFlags) -> Result<usize> {
    unsafe {
        let ret = libc::recv(
            sockfd,
            buf.as_ptr() as *mut c_void,
            buf.len() as size_t,
            flags.bits());

        Errno::result(ret).map(|r| r as usize)
    }
}

/// Receive data from a connectionless or connection-oriented socket. Returns
/// the number of bytes read and, for connectionless sockets,  the socket
/// address of the sender.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/recvfrom.html)
pub fn recvfrom(sockfd: RawFd, buf: &mut [u8])
    -> Result<(usize, Option<SockAddr>)>
{
    unsafe {
        let mut addr: sockaddr_storage = mem::zeroed();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = Errno::result(libc::recvfrom(
            sockfd,
            buf.as_ptr() as *mut c_void,
            buf.len() as size_t,
            0,
            &mut addr as *mut libc::sockaddr_storage as *mut libc::sockaddr,
            &mut len as *mut socklen_t))? as usize;

        match sockaddr_storage_to_addr(&addr, len as usize) {
            Err(Errno::ENOTCONN) => Ok((ret, None)),
            Ok(addr) => Ok((ret, Some(addr))),
            Err(e) => Err(e)
        }
    }
}

/// Send a message to a socket
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/sendto.html)
pub fn sendto(fd: RawFd, buf: &[u8], addr: &SockAddr, flags: MsgFlags) -> Result<usize> {
    let ret = unsafe {
        let (ptr, len) = addr.as_ffi_pair();
        libc::sendto(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags.bits(), ptr, len)
    };

    Errno::result(ret).map(|r| r as usize)
}

/// Send data to a connection-oriented socket. Returns the number of bytes read
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/send.html)
pub fn send(fd: RawFd, buf: &[u8], flags: MsgFlags) -> Result<usize> {
    let ret = unsafe {
        libc::send(fd, buf.as_ptr() as *const c_void, buf.len() as size_t, flags.bits())
    };

    Errno::result(ret).map(|r| r as usize)
}

/*
 *
 * ===== Socket Options =====
 *
 */

/// Represents a socket option that can be retrieved.
pub trait GetSockOpt : Copy {
    type Val;

    /// Look up the value of this socket option on the given socket.
    fn get(&self, fd: RawFd) -> Result<Self::Val>;
}

/// Represents a socket option that can be set.
pub trait SetSockOpt : Clone {
    type Val;

    /// Set the value of this socket option on the given socket.
    fn set(&self, fd: RawFd, val: &Self::Val) -> Result<()>;
}

/// Get the current value for the requested socket option
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockopt.html)
pub fn getsockopt<O: GetSockOpt>(fd: RawFd, opt: O) -> Result<O::Val> {
    opt.get(fd)
}

/// Sets the value for the requested socket option
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/setsockopt.html)
///
/// # Examples
///
/// ```
/// use nix::sys::socket::setsockopt;
/// use nix::sys::socket::sockopt::KeepAlive;
/// use std::net::TcpListener;
/// use std::os::unix::io::AsRawFd;
///
/// let listener = TcpListener::bind("0.0.0.0:0").unwrap();
/// let fd = listener.as_raw_fd();
/// let res = setsockopt(fd, KeepAlive, &true);
/// assert!(res.is_ok());
/// ```
pub fn setsockopt<O: SetSockOpt>(fd: RawFd, opt: O, val: &O::Val) -> Result<()> {
    opt.set(fd, val)
}

/// Get the address of the peer connected to the socket `fd`.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getpeername.html)
pub fn getpeername(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let mut addr = mem::MaybeUninit::uninit();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = libc::getpeername(
            fd,
            addr.as_mut_ptr() as *mut libc::sockaddr,
            &mut len
        );

        Errno::result(ret)?;

        sockaddr_storage_to_addr(&addr.assume_init(), len as usize)
    }
}

/// Get the current address to which the socket `fd` is bound.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/getsockname.html)
pub fn getsockname(fd: RawFd) -> Result<SockAddr> {
    unsafe {
        let mut addr = mem::MaybeUninit::uninit();
        let mut len = mem::size_of::<sockaddr_storage>() as socklen_t;

        let ret = libc::getsockname(
            fd,
            addr.as_mut_ptr() as *mut libc::sockaddr,
            &mut len
        );

        Errno::result(ret)?;

        sockaddr_storage_to_addr(&addr.assume_init(), len as usize)
    }
}

/// Return the appropriate `SockAddr` type from a `sockaddr_storage` of a
/// certain size.
///
/// In C this would usually be done by casting.  The `len` argument
/// should be the number of bytes in the `sockaddr_storage` that are actually
/// allocated and valid.  It must be at least as large as all the useful parts
/// of the structure.  Note that in the case of a `sockaddr_un`, `len` need not
/// include the terminating null.
pub fn sockaddr_storage_to_addr(
    addr: &sockaddr_storage,
    len: usize) -> Result<SockAddr> {

    assert!(len <= mem::size_of::<sockaddr_storage>());
    if len < mem::size_of_val(&addr.ss_family) {
        return Err(Errno::ENOTCONN);
    }

    match c_int::from(addr.ss_family) {
        libc::AF_INET => {
            assert!(len as usize >= mem::size_of::<sockaddr_in>());
            let sin = unsafe {
                *(addr as *const sockaddr_storage as *const sockaddr_in)
            };
            Ok(SockAddr::Inet(InetAddr::V4(sin)))
        }
        libc::AF_INET6 => {
            assert!(len as usize >= mem::size_of::<sockaddr_in6>());
            let sin6 = unsafe {
                *(addr as *const _ as *const sockaddr_in6)
            };
            Ok(SockAddr::Inet(InetAddr::V6(sin6)))
        }
        libc::AF_UNIX => {
            let pathlen = len - offset_of!(sockaddr_un, sun_path);
            unsafe {
                let sun = *(addr as *const _ as *const sockaddr_un);
                Ok(SockAddr::Unix(UnixAddr::from_raw_parts(sun, pathlen)))
            }
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        libc::AF_PACKET => {
            use libc::sockaddr_ll;
            // Don't assert anything about the size.
            // Apparently the Linux kernel can return smaller sizes when
            // the value in the last element of sockaddr_ll (`sll_addr`) is
            // smaller than the declared size of that field
            let sll = unsafe {
                *(addr as *const _ as *const sockaddr_ll)
            };
            Ok(SockAddr::Link(LinkAddr(sll)))
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        libc::AF_NETLINK => {
            use libc::sockaddr_nl;
            let snl = unsafe {
                *(addr as *const _ as *const sockaddr_nl)
            };
            Ok(SockAddr::Netlink(NetlinkAddr(snl)))
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        libc::AF_ALG => {
            use libc::sockaddr_alg;
            let salg = unsafe {
                *(addr as *const _ as *const sockaddr_alg)
            };
            Ok(SockAddr::Alg(AlgAddr(salg)))
        }
        #[cfg(any(target_os = "android", target_os = "linux"))]
        libc::AF_VSOCK => {
            use libc::sockaddr_vm;
            let svm = unsafe {
                *(addr as *const _ as *const sockaddr_vm)
            };
            Ok(SockAddr::Vsock(VsockAddr(svm)))
        }
        af => panic!("unexpected address family {}", af),
    }
}


#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Shutdown {
    /// Further receptions will be disallowed.
    Read,
    /// Further  transmissions will be disallowed.
    Write,
    /// Further receptions and transmissions will be disallowed.
    Both,
}

/// Shut down part of a full-duplex connection.
///
/// [Further reading](https://pubs.opengroup.org/onlinepubs/9699919799/functions/shutdown.html)
pub fn shutdown(df: RawFd, how: Shutdown) -> Result<()> {
    unsafe {
        use libc::shutdown;

        let how = match how {
            Shutdown::Read  => libc::SHUT_RD,
            Shutdown::Write => libc::SHUT_WR,
            Shutdown::Both  => libc::SHUT_RDWR,
        };

        Errno::result(shutdown(df, how)).map(drop)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn can_use_cmsg_space() {
        let _ = cmsg_space!(u8);
    }
}
