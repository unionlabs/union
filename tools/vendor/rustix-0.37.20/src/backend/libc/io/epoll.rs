//! epoll support.
//!
//! This is an experiment, and it isn't yet clear whether epoll is the right
//! level of abstraction at which to introduce safety. But it works fairly well
//! in simple examples 🙂.
//!
//! # Examples
//!
//! ```no_run
//! # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
//! # #[cfg(feature = "net")]
//! # fn main() -> std::io::Result<()> {
//! use io_lifetimes::AsFd;
//! use rustix::io::{epoll, ioctl_fionbio, read, write};
//! use rustix::net::{
//!     accept, bind_v4, listen, socket, AddressFamily, Ipv4Addr, Protocol, SocketAddrV4,
//!     SocketType,
//! };
//! use std::collections::HashMap;
//! use std::os::unix::io::AsRawFd;
//!
//! // Create a socket and listen on it.
//! let listen_sock = socket(AddressFamily::INET, SocketType::STREAM, Protocol::default())?;
//! bind_v4(&listen_sock, &SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
//! listen(&listen_sock, 1)?;
//!
//! // Create an epoll object. Using `Owning` here means the epoll object will
//! // take ownership of the file descriptors registered with it.
//! let epoll = epoll::epoll_create(epoll::CreateFlags::CLOEXEC)?;
//!
//! // Register the socket with the epoll object.
//! epoll::epoll_add(&epoll, &listen_sock, 1, epoll::EventFlags::IN)?;
//!
//! // Keep track of the sockets we've opened.
//! let mut next_id = 2;
//! let mut sockets = HashMap::new();
//!
//! // Process events.
//! let mut event_list = epoll::EventVec::with_capacity(4);
//! loop {
//!     epoll::epoll_wait(&epoll, &mut event_list, -1)?;
//!     for (_event_flags, target) in &event_list {
//!         if target == 1 {
//!             // Accept a new connection, set it to non-blocking, and
//!             // register to be notified when it's ready to write to.
//!             let conn_sock = accept(&listen_sock)?;
//!             ioctl_fionbio(&conn_sock, true)?;
//!             epoll::epoll_add(
//!                 &epoll,
//!                 &conn_sock,
//!                 next_id,
//!                 epoll::EventFlags::OUT | epoll::EventFlags::ET,
//!             )?;
//!
//!             // Keep track of the socket.
//!             sockets.insert(next_id, conn_sock);
//!             next_id += 1;
//!         } else {
//!             // Write a message to the stream and then unregister it.
//!             let target = sockets.remove(&target).unwrap();
//!             write(&target, b"hello\n")?;
//!             let _ = epoll::epoll_del(&epoll, &target)?;
//!         }
//!     }
//! }
//! # }
//! # #[cfg(not(feature = "net"))]
//! # fn main() {}
//! ```

use super::super::c;
use super::super::conv::{ret, ret_owned_fd, ret_u32};
use crate::fd::{AsFd, AsRawFd, OwnedFd};
use crate::io;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::convert::TryInto;
use core::ptr::null_mut;

bitflags! {
    /// `EPOLL_*` for use with [`Epoll::new`].
    pub struct CreateFlags: c::c_int {
        /// `EPOLL_CLOEXEC`
        const CLOEXEC = c::EPOLL_CLOEXEC;
    }
}

bitflags! {
    /// `EPOLL*` for use with [`Epoll::add`].
    #[derive(Default)]
    pub struct EventFlags: u32 {
        /// `EPOLLIN`
        const IN = c::EPOLLIN as u32;

        /// `EPOLLOUT`
        const OUT = c::EPOLLOUT as u32;

        /// `EPOLLPRI`
        const PRI = c::EPOLLPRI as u32;

        /// `EPOLLERR`
        const ERR = c::EPOLLERR as u32;

        /// `EPOLLHUP`
        const HUP = c::EPOLLHUP as u32;

        /// `EPOLLRDNORM`
        const RDNORM = c::EPOLLRDNORM as u32;

        /// `EPOLLRDBAND`
        const RDBAND = c::EPOLLRDBAND as u32;

        /// `EPOLLWRNORM`
        const WRNORM = c::EPOLLWRNORM as u32;

        /// `EPOLLWRBAND`
        const WRBAND = c::EPOLLWRBAND as u32;

        /// `EPOLLMSG`
        const MSG = c::EPOLLMSG as u32;

        /// `EPOLLRDHUP`
        const RDHUP = c::EPOLLRDHUP as u32;

        /// `EPOLLET`
        const ET = c::EPOLLET as u32;

        /// `EPOLLONESHOT`
        const ONESHOT = c::EPOLLONESHOT as u32;

        /// `EPOLLWAKEUP`
        const WAKEUP = c::EPOLLWAKEUP as u32;

        /// `EPOLLEXCLUSIVE`
        #[cfg(not(target_os = "android"))]
        const EXCLUSIVE = c::EPOLLEXCLUSIVE as u32;
    }
}

/// `epoll_create1(flags)`—Creates a new `Epoll`.
///
/// Use the [`CreateFlags::CLOEXEC`] flag to prevent the resulting file
/// descriptor from being implicitly passed across `exec` boundaries.
#[inline]
#[doc(alias = "epoll_create1")]
pub fn epoll_create(flags: CreateFlags) -> io::Result<OwnedFd> {
    // SAFETY: We're calling `epoll_create1` via FFI and we know how it
    // behaves.
    unsafe { ret_owned_fd(c::epoll_create1(flags.bits())) }
}

/// `epoll_ctl(self, EPOLL_CTL_ADD, data, event)`—Adds an element to an
/// `Epoll`.
///
/// If `epoll_del` is not called on the I/O source passed into this function
/// before the I/O source is `close`d, then the `epoll` will act as if the I/O
/// source is still registered with it. This can lead to spurious events being
/// returned from `epoll_wait`. If a file descriptor is an
/// `Arc<dyn SystemResource>`, then `epoll` can be thought to maintain a
/// `Weak<dyn SystemResource>` to the file descriptor.
#[doc(alias = "epoll_ctl")]
pub fn epoll_add(
    epoll: impl AsFd,
    source: impl AsFd,
    data: u64,
    event_flags: EventFlags,
) -> io::Result<()> {
    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        let raw_fd = source.as_fd().as_raw_fd();
        ret(c::epoll_ctl(
            epoll.as_fd().as_raw_fd(),
            c::EPOLL_CTL_ADD,
            raw_fd,
            &mut c::epoll_event {
                events: event_flags.bits(),
                r#u64: data,
            },
        ))
    }
}

/// `epoll_ctl(self, EPOLL_CTL_MOD, target, event)`—Modifies an element in
/// this `Epoll`.
///
/// This sets the events of interest with `target` to `events`.
#[doc(alias = "epoll_ctl")]
pub fn epoll_mod(
    epoll: impl AsFd,
    source: impl AsFd,
    data: u64,
    event_flags: EventFlags,
) -> io::Result<()> {
    let raw_fd = source.as_fd().as_raw_fd();

    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        ret(c::epoll_ctl(
            epoll.as_fd().as_raw_fd(),
            c::EPOLL_CTL_MOD,
            raw_fd,
            &mut c::epoll_event {
                events: event_flags.bits(),
                r#u64: data,
            },
        ))
    }
}

/// `epoll_ctl(self, EPOLL_CTL_DEL, target, NULL)`—Removes an element in
/// this `Epoll`.
#[doc(alias = "epoll_ctl")]
pub fn epoll_del(epoll: impl AsFd, source: impl AsFd) -> io::Result<()> {
    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        let raw_fd = source.as_fd().as_raw_fd();
        ret(c::epoll_ctl(
            epoll.as_fd().as_raw_fd(),
            c::EPOLL_CTL_DEL,
            raw_fd,
            null_mut(),
        ))
    }
}

/// `epoll_wait(self, events, timeout)`—Waits for registered events of
/// interest.
///
/// For each event of interest, an element is written to `events`. On
/// success, this returns the number of written elements.
pub fn epoll_wait(
    epoll: impl AsFd,
    event_list: &mut EventVec,
    timeout: c::c_int,
) -> io::Result<()> {
    // SAFETY: We're calling `epoll_wait` via FFI and we know how it
    // behaves.
    unsafe {
        event_list.events.set_len(0);
        let nfds = ret_u32(c::epoll_wait(
            epoll.as_fd().as_raw_fd(),
            event_list.events.as_mut_ptr().cast::<c::epoll_event>(),
            event_list.events.capacity().try_into().unwrap_or(i32::MAX),
            timeout,
        ))?;
        event_list.events.set_len(nfds as usize);
    }

    Ok(())
}

/// An iterator over the `Event`s in an `EventVec`.
pub struct Iter<'a> {
    iter: core::slice::Iter<'a, Event>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = (EventFlags, u64);

    fn next(&mut self) -> Option<Self::Item> {
        // SAFETY: `self.context` is guaranteed to be valid because we hold
        // `'context` for it. And we know this event is associated with this
        // context because `wait` sets both.
        self.iter
            .next()
            .map(|event| (event.event_flags, event.data))
    }
}

/// A record of an event that occurred.
#[repr(C)]
#[cfg_attr(
    any(
        all(
            target_arch = "x86",
            not(target_env = "musl"),
            not(target_os = "android"),
        ),
        target_arch = "x86_64",
    ),
    repr(packed)
)]
struct Event {
    // Match the layout of `c::epoll_event`. We just use a `u64` instead of
    // the full union.
    event_flags: EventFlags,
    data: u64,
}

/// A vector of `Event`s, plus context for interpreting them.
pub struct EventVec {
    events: Vec<Event>,
}

impl EventVec {
    /// Constructs an `EventVec` with memory for `capacity` `Event`s.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            events: Vec::with_capacity(capacity),
        }
    }

    /// Returns the current `Event` capacity of this `EventVec`.
    #[inline]
    pub fn capacity(&self) -> usize {
        self.events.capacity()
    }

    /// Reserves enough memory for at least `additional` more `Event`s.
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.events.reserve(additional);
    }

    /// Reserves enough memory for exactly `additional` more `Event`s.
    #[inline]
    pub fn reserve_exact(&mut self, additional: usize) {
        self.events.reserve_exact(additional);
    }

    /// Clears all the `Events` out of this `EventVec`.
    #[inline]
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Shrinks the capacity of this `EventVec` as much as possible.
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.events.shrink_to_fit();
    }

    /// Returns an iterator over the `Event`s in this `EventVec`.
    #[inline]
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            iter: self.events.iter(),
        }
    }

    /// Returns the number of `Event`s logically contained in this `EventVec`.
    #[inline]
    pub fn len(&mut self) -> usize {
        self.events.len()
    }

    /// Tests whether this `EventVec` is logically empty.
    #[inline]
    pub fn is_empty(&mut self) -> bool {
        self.events.is_empty()
    }
}

impl<'a> IntoIterator for &'a EventVec {
    type IntoIter = Iter<'a>;
    type Item = (EventFlags, u64);

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
