// Copyright 2015 The Rust Project Developers.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Utilities for creating and using sockets.
//!
//! The goal of this crate is to create and use a socket using advanced
//! configuration options (those that are not available in the types in the
//! standard library) without using any unsafe code.
//!
//! This crate provides as direct as possible access to the system's
//! functionality for sockets, this means little effort to provide
//! cross-platform utilities. It is up to the user to know how to use sockets
//! when using this crate. *If you don't know how to create a socket using
//! libc/system calls then this crate is not for you*. Most, if not all,
//! functions directly relate to the equivalent system call with no error
//! handling applied, so no handling errors such as [`EINTR`]. As a result using
//! this crate can be a little wordy, but it should give you maximal flexibility
//! over configuration of sockets.
//!
//! [`EINTR`]: std::io::ErrorKind::Interrupted
//!
//! # Examples
//!
//! ```no_run
//! # fn main() -> std::io::Result<()> {
//! use std::net::{SocketAddr, TcpListener};
//! use socket2::{Socket, Domain, Type};
//!
//! // Create a TCP listener bound to two addresses.
//! let socket = Socket::new(Domain::IPV6, Type::STREAM, None)?;
//!
//! socket.set_only_v6(false)?;
//! let address: SocketAddr = "[::1]:12345".parse().unwrap();
//! socket.bind(&address.into())?;
//! socket.listen(128)?;
//!
//! let listener: TcpListener = socket.into();
//! // ...
//! # drop(listener);
//! # Ok(()) }
//! ```
//!
//! ## Features
//!
//! This crate has a single feature `all`, which enables all functions even ones
//! that are not available on all OSs.

#![doc(html_root_url = "https://docs.rs/socket2/0.3")]
#![deny(missing_docs, missing_debug_implementations, rust_2018_idioms)]
// Show required OS/features on docs.rs.
#![cfg_attr(docsrs, feature(doc_cfg))]
// Disallow warnings when running tests.
#![cfg_attr(test, deny(warnings))]
// Disallow warnings in examples.
#![doc(test(attr(deny(warnings))))]

use std::fmt;
use std::mem::MaybeUninit;
use std::net::SocketAddr;
use std::ops::{Deref, DerefMut};
use std::time::Duration;

/// Macro to implement `fmt::Debug` for a type, printing the constant names
/// rather than a number.
///
/// Note this is used in the `sys` module and thus must be defined before
/// defining the modules.
macro_rules! impl_debug {
    (
        // Type name for which to implement `fmt::Debug`.
        $type: path,
        $(
            $(#[$target: meta])*
            // The flag(s) to check.
            // Need to specific the libc crate because Windows doesn't use
            // `libc` but `winapi`.
            $libc: ident :: $flag: ident
        ),+ $(,)*
    ) => {
        impl std::fmt::Debug for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let string = match self.0 {
                    $(
                        $(#[$target])*
                        $libc :: $flag => stringify!($flag),
                    )+
                    n => return write!(f, "{}", n),
                };
                f.write_str(string)
            }
        }
    };
}

/// Macro to convert from one network type to another.
macro_rules! from {
    ($from: ty, $for: ty) => {
        impl From<$from> for $for {
            fn from(socket: $from) -> $for {
                #[cfg(unix)]
                unsafe {
                    <$for>::from_raw_fd(socket.into_raw_fd())
                }
                #[cfg(windows)]
                unsafe {
                    <$for>::from_raw_socket(socket.into_raw_socket())
                }
            }
        }
    };
}

mod sockaddr;
mod socket;
mod sockref;

#[cfg_attr(unix, path = "sys/unix.rs")]
#[cfg_attr(windows, path = "sys/windows.rs")]
mod sys;

#[cfg(not(any(windows, unix)))]
compile_error!("Socket2 doesn't support the compile target");

use sys::c_int;

pub use sockaddr::SockAddr;
pub use socket::Socket;
pub use sockref::SockRef;

#[cfg(not(any(
    target_os = "haiku",
    target_os = "illumos",
    target_os = "netbsd",
    target_os = "redox",
    target_os = "solaris",
)))]
pub use socket::InterfaceIndexOrAddress;

/// Specification of the communication domain for a socket.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation. Convenience constants such
/// as [`Domain::IPV4`], [`Domain::IPV6`], etc, are provided to avoid reaching
/// into libc for various constants.
///
/// This type is freely interconvertible with C's `int` type, however, if a raw
/// value needs to be provided.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Domain(c_int);

impl Domain {
    /// Domain for IPv4 communication, corresponding to `AF_INET`.
    pub const IPV4: Domain = Domain(sys::AF_INET);

    /// Domain for IPv6 communication, corresponding to `AF_INET6`.
    pub const IPV6: Domain = Domain(sys::AF_INET6);

    /// Returns the correct domain for `address`.
    pub const fn for_address(address: SocketAddr) -> Domain {
        match address {
            SocketAddr::V4(_) => Domain::IPV4,
            SocketAddr::V6(_) => Domain::IPV6,
        }
    }
}

impl From<c_int> for Domain {
    fn from(d: c_int) -> Domain {
        Domain(d)
    }
}

impl From<Domain> for c_int {
    fn from(d: Domain) -> c_int {
        d.0
    }
}

/// Specification of communication semantics on a socket.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation. Convenience constants such
/// as [`Type::STREAM`], [`Type::DGRAM`], etc, are provided to avoid reaching
/// into libc for various constants.
///
/// This type is freely interconvertible with C's `int` type, however, if a raw
/// value needs to be provided.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Type(c_int);

impl Type {
    /// Type corresponding to `SOCK_STREAM`.
    ///
    /// Used for protocols such as TCP.
    pub const STREAM: Type = Type(sys::SOCK_STREAM);

    /// Type corresponding to `SOCK_DGRAM`.
    ///
    /// Used for protocols such as UDP.
    pub const DGRAM: Type = Type(sys::SOCK_DGRAM);

    /// Type corresponding to `SOCK_SEQPACKET`.
    #[cfg(feature = "all")]
    #[cfg_attr(docsrs, doc(cfg(feature = "all")))]
    pub const SEQPACKET: Type = Type(sys::SOCK_SEQPACKET);

    /// Type corresponding to `SOCK_RAW`.
    #[cfg(all(feature = "all", not(target_os = "redox")))]
    #[cfg_attr(docsrs, doc(cfg(all(feature = "all", not(target_os = "redox")))))]
    pub const RAW: Type = Type(sys::SOCK_RAW);
}

impl From<c_int> for Type {
    fn from(t: c_int) -> Type {
        Type(t)
    }
}

impl From<Type> for c_int {
    fn from(t: Type) -> c_int {
        t.0
    }
}

/// Protocol specification used for creating sockets via `Socket::new`.
///
/// This is a newtype wrapper around an integer which provides a nicer API in
/// addition to an injection point for documentation.
///
/// This type is freely interconvertible with C's `int` type, however, if a raw
/// value needs to be provided.
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Protocol(c_int);

impl Protocol {
    /// Protocol corresponding to `ICMPv4`.
    pub const ICMPV4: Protocol = Protocol(sys::IPPROTO_ICMP);

    /// Protocol corresponding to `ICMPv6`.
    pub const ICMPV6: Protocol = Protocol(sys::IPPROTO_ICMPV6);

    /// Protocol corresponding to `TCP`.
    pub const TCP: Protocol = Protocol(sys::IPPROTO_TCP);

    /// Protocol corresponding to `UDP`.
    pub const UDP: Protocol = Protocol(sys::IPPROTO_UDP);
}

impl From<c_int> for Protocol {
    fn from(p: c_int) -> Protocol {
        Protocol(p)
    }
}

impl From<Protocol> for c_int {
    fn from(p: Protocol) -> c_int {
        p.0
    }
}

/// Flags for incoming messages.
///
/// Flags provide additional information about incoming messages.
#[cfg(not(target_os = "redox"))]
#[cfg_attr(docsrs, doc(cfg(not(target_os = "redox"))))]
#[derive(Copy, Clone, Eq, PartialEq)]
pub struct RecvFlags(c_int);

#[cfg(not(target_os = "redox"))]
impl RecvFlags {
    /// Check if the message contains a truncated datagram.
    ///
    /// This flag is only used for datagram-based sockets,
    /// not for stream sockets.
    ///
    /// On Unix this corresponds to the `MSG_TRUNC` flag.
    /// On Windows this corresponds to the `WSAEMSGSIZE` error code.
    pub const fn is_truncated(self) -> bool {
        self.0 & sys::MSG_TRUNC != 0
    }
}

/// A version of [`IoSliceMut`] that allows the buffer to be uninitialised.
///
/// [`IoSliceMut`]: std::io::IoSliceMut
#[repr(transparent)]
pub struct MaybeUninitSlice<'a>(sys::MaybeUninitSlice<'a>);

impl<'a> fmt::Debug for MaybeUninitSlice<'a> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.0.as_slice(), fmt)
    }
}

impl<'a> MaybeUninitSlice<'a> {
    /// Creates a new `MaybeUninitSlice` wrapping a byte slice.
    ///
    /// # Panics
    ///
    /// Panics on Windows if the slice is larger than 4GB.
    pub fn new(buf: &'a mut [MaybeUninit<u8>]) -> MaybeUninitSlice<'a> {
        MaybeUninitSlice(sys::MaybeUninitSlice::new(buf))
    }
}

impl<'a> Deref for MaybeUninitSlice<'a> {
    type Target = [MaybeUninit<u8>];

    fn deref(&self) -> &[MaybeUninit<u8>] {
        self.0.as_slice()
    }
}

impl<'a> DerefMut for MaybeUninitSlice<'a> {
    fn deref_mut(&mut self) -> &mut [MaybeUninit<u8>] {
        self.0.as_mut_slice()
    }
}

/// Configures a socket's TCP keepalive parameters.
///
/// See [`Socket::set_tcp_keepalive`].
#[derive(Debug, Clone)]
pub struct TcpKeepalive {
    time: Option<Duration>,
    #[cfg(not(any(target_os = "redox", target_os = "solaris")))]
    interval: Option<Duration>,
    #[cfg(not(any(target_os = "redox", target_os = "solaris", target_os = "windows")))]
    retries: Option<u32>,
}

impl TcpKeepalive {
    /// Returns a new, empty set of TCP keepalive parameters.
    pub const fn new() -> TcpKeepalive {
        TcpKeepalive {
            time: None,
            #[cfg(not(any(target_os = "redox", target_os = "solaris")))]
            interval: None,
            #[cfg(not(any(target_os = "redox", target_os = "solaris", target_os = "windows")))]
            retries: None,
        }
    }

    /// Set the amount of time after which TCP keepalive probes will be sent on
    /// idle connections.
    ///
    /// This will set `TCP_KEEPALIVE` on macOS and iOS, and
    /// `TCP_KEEPIDLE` on all other Unix operating systems, except
    /// OpenBSD and Haiku which don't support any way to set this
    /// option. On Windows, this sets the value of the `tcp_keepalive`
    /// struct's `keepalivetime` field.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    pub const fn with_time(self, time: Duration) -> Self {
        Self {
            time: Some(time),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPINTVL` option. On Windows, this sets the
    /// value of the `tcp_keepalive` struct's `keepaliveinterval` field.
    ///
    /// Sets the time interval between TCP keepalive probes.
    ///
    /// Some platforms specify this value in seconds, so sub-second
    /// specifications may be omitted.
    #[cfg(all(
        feature = "all",
        any(
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
            target_os = "netbsd",
            target_vendor = "apple",
            windows,
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "linux",
                target_os = "netbsd",
                target_vendor = "apple",
                windows,
            )
        )))
    )]
    pub const fn with_interval(self, interval: Duration) -> Self {
        Self {
            interval: Some(interval),
            ..self
        }
    }

    /// Set the value of the `TCP_KEEPCNT` option.
    ///
    /// Set the maximum number of TCP keepalive probes that will be sent before
    /// dropping a connection, if TCP keepalive is enabled on this socket.
    #[cfg(all(
        feature = "all",
        any(
            doc,
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "fuchsia",
            target_os = "linux",
            target_os = "netbsd",
            target_vendor = "apple",
        )
    ))]
    #[cfg_attr(
        docsrs,
        doc(cfg(all(
            feature = "all",
            any(
                target_os = "freebsd",
                target_os = "fuchsia",
                target_os = "linux",
                target_os = "netbsd",
                target_vendor = "apple",
            )
        )))
    )]
    pub const fn with_retries(self, retries: u32) -> Self {
        Self {
            retries: Some(retries),
            ..self
        }
    }
}
