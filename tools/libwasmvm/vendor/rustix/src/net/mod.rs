//! Network-related operations.
//!
//! On Windows, one must call [`wsa_startup`] in the process before calling any
//! of these APIs. [`wsa_cleanup`] may be used in the process if these APIs are
//! no longer needed.
//!
//! [`wsa_startup`]: https://docs.rs/rustix/*/x86_64-pc-windows-msvc/rustix/net/fn.wsa_startup.html
//! [`wsa_cleanup`]: https://docs.rs/rustix/*/x86_64-pc-windows-msvc/rustix/net/fn.wsa_cleanup.html

#[cfg(not(feature = "std"))]
mod addr;
#[cfg(not(feature = "std"))]
mod ip;
mod send_recv;
mod socket;
mod socket_addr_any;
#[cfg(not(any(windows, target_os = "wasi")))]
mod socketpair;
#[cfg(windows)]
mod wsa;

pub mod sockopt;

pub use send_recv::*;
pub use socket::*;
pub use socket_addr_any::{SocketAddrAny, SocketAddrStorage};
#[cfg(not(any(windows, target_os = "wasi")))]
pub use socketpair::socketpair;
#[cfg(feature = "std")]
pub use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
#[cfg(windows)]
pub use wsa::{wsa_cleanup, wsa_startup};
#[cfg(not(feature = "std"))]
pub use {
    addr::{SocketAddr, SocketAddrV4, SocketAddrV6},
    ip::{IpAddr, Ipv4Addr, Ipv6Addr, Ipv6MulticastScope},
};
