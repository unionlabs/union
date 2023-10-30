//! epoll support.
//!
//! This is an experiment, and it isn't yet clear whether epoll is the right
//! level of abstraction at which to introduce safety. But it works fairly well
//! in simple examples 🙂.
//!
//! # Examples
//!
//! ```no_run
//! # #[cfg(feature = "net")]
//! # fn main() -> std::io::Result<()> {
//! use rustix::event::epoll;
//! use rustix::fd::AsFd;
//! use rustix::io::{ioctl_fionbio, read, write};
//! use rustix::net::{
//!     accept, bind_v4, listen, socket, AddressFamily, Ipv4Addr, SocketAddrV4, SocketType,
//! };
//! use std::collections::HashMap;
//! use std::os::unix::io::AsRawFd;
//!
//! // Create a socket and listen on it.
//! let listen_sock = socket(AddressFamily::INET, SocketType::STREAM, None)?;
//! bind_v4(&listen_sock, &SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
//! listen(&listen_sock, 1)?;
//!
//! // Create an epoll object. Using `Owning` here means the epoll object will
//! // take ownership of the file descriptors registered with it.
//! let epoll = epoll::create(epoll::CreateFlags::CLOEXEC)?;
//!
//! // Register the socket with the epoll object.
//! epoll::add(
//!     &epoll,
//!     &listen_sock,
//!     epoll::EventData::new_u64(1),
//!     epoll::EventFlags::IN,
//! )?;
//!
//! // Keep track of the sockets we've opened.
//! let mut next_id = epoll::EventData::new_u64(2);
//! let mut sockets = HashMap::new();
//!
//! // Process events.
//! let mut event_list = epoll::EventVec::with_capacity(4);
//! loop {
//!     epoll::wait(&epoll, &mut event_list, -1)?;
//!     for event in &event_list {
//!         let target = event.data;
//!         if target.u64() == 1 {
//!             // Accept a new connection, set it to non-blocking, and
//!             // register to be notified when it's ready to write to.
//!             let conn_sock = accept(&listen_sock)?;
//!             ioctl_fionbio(&conn_sock, true)?;
//!             epoll::add(
//!                 &epoll,
//!                 &conn_sock,
//!                 next_id,
//!                 epoll::EventFlags::OUT | epoll::EventFlags::ET,
//!             )?;
//!
//!             // Keep track of the socket.
//!             sockets.insert(next_id, conn_sock);
//!             next_id = epoll::EventData::new_u64(next_id.u64() + 1);
//!         } else {
//!             // Write a message to the stream and then unregister it.
//!             let target = sockets.remove(&target).unwrap();
//!             write(&target, b"hello\n")?;
//!             let _ = epoll::delete(&epoll, &target)?;
//!         }
//!     }
//! }
//! # }
//! # #[cfg(not(feature = "net"))]
//! # fn main() {}
//! ```

#![allow(unsafe_code)]

use crate::backend::c;
use crate::backend::event::syscalls;
use crate::fd::{AsFd, AsRawFd, OwnedFd};
use crate::io;
use alloc::vec::Vec;
use bitflags::bitflags;
use core::ffi::c_void;
use core::hash::{Hash, Hasher};
use core::slice;

bitflags! {
    /// `EPOLL_*` for use with [`new`].
    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct CreateFlags: c::c_uint {
        /// `EPOLL_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::EPOLL_CLOEXEC;
    }
}

bitflags! {
    /// `EPOLL*` for use with [`add`].
    #[repr(transparent)]
    #[derive(Default, Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct EventFlags: u32 {
        /// `EPOLLIN`
        const IN = linux_raw_sys::general::EPOLLIN as u32;

        /// `EPOLLOUT`
        const OUT = linux_raw_sys::general::EPOLLOUT as u32;

        /// `EPOLLPRI`
        const PRI = linux_raw_sys::general::EPOLLPRI as u32;

        /// `EPOLLERR`
        const ERR = linux_raw_sys::general::EPOLLERR as u32;

        /// `EPOLLHUP`
        const HUP = linux_raw_sys::general::EPOLLHUP as u32;

        /// `EPOLLRDNORM`
        const RDNORM = linux_raw_sys::general::EPOLLRDNORM as u32;

        /// `EPOLLRDBAND`
        const RDBAND = linux_raw_sys::general::EPOLLRDBAND as u32;

        /// `EPOLLWRNORM`
        const WRNORM = linux_raw_sys::general::EPOLLWRNORM as u32;

        /// `EPOLLWRBAND`
        const WRBAND = linux_raw_sys::general::EPOLLWRBAND as u32;

        /// `EPOLLMSG`
        const MSG = linux_raw_sys::general::EPOLLMSG as u32;

        /// `EPOLLRDHUP`
        const RDHUP = linux_raw_sys::general::EPOLLRDHUP as u32;

        /// `EPOLLET`
        const ET = linux_raw_sys::general::EPOLLET as u32;

        /// `EPOLLONESHOT`
        const ONESHOT = linux_raw_sys::general::EPOLLONESHOT as u32;

        /// `EPOLLWAKEUP`
        const WAKEUP = linux_raw_sys::general::EPOLLWAKEUP as u32;

        /// `EPOLLEXCLUSIVE`
        const EXCLUSIVE = linux_raw_sys::general::EPOLLEXCLUSIVE as u32;
    }
}

/// `epoll_create1(flags)`—Creates a new epoll object.
///
/// Use the [`CreateFlags::CLOEXEC`] flag to prevent the resulting file
/// descriptor from being implicitly passed across `exec` boundaries.
#[inline]
#[doc(alias = "epoll_create1")]
pub fn create(flags: CreateFlags) -> io::Result<OwnedFd> {
    syscalls::epoll_create(flags)
}

/// `epoll_ctl(self, EPOLL_CTL_ADD, data, event)`—Adds an element to an
/// epoll object.
///
/// This registers interest in any of the events set in `events` occurring
/// on the file descriptor associated with `data`.
///
/// If [`delete`] is not called on the I/O source passed into this function
/// before the I/O source is `close`d, then the `epoll` will act as if the I/O
/// source is still registered with it. This can lead to spurious events being
/// returned from [`wait`]. If a file descriptor is an
/// `Arc<dyn SystemResource>`, then `epoll` can be thought to maintain a
/// `Weak<dyn SystemResource>` to the file descriptor.
#[doc(alias = "epoll_ctl")]
#[inline]
pub fn add(
    epoll: impl AsFd,
    source: impl AsFd,
    data: EventData,
    event_flags: EventFlags,
) -> io::Result<()> {
    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        syscalls::epoll_add(
            epoll.as_fd(),
            source.as_fd().as_raw_fd(),
            &Event {
                flags: event_flags,
                data,
            },
        )
    }
}

/// `epoll_ctl(self, EPOLL_CTL_MOD, target, event)`—Modifies an element in
/// a given epoll object.
///
/// This sets the events of interest with `target` to `events`.
#[doc(alias = "epoll_ctl")]
#[inline]
pub fn modify(
    epoll: impl AsFd,
    source: impl AsFd,
    data: EventData,
    event_flags: EventFlags,
) -> io::Result<()> {
    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        let raw_fd = source.as_fd().as_raw_fd();
        syscalls::epoll_mod(
            epoll.as_fd(),
            raw_fd,
            &Event {
                flags: event_flags,
                data,
            },
        )
    }
}

/// `epoll_ctl(self, EPOLL_CTL_DEL, target, NULL)`—Removes an element in
/// a given epoll object.
#[doc(alias = "epoll_ctl")]
#[inline]
pub fn delete(epoll: impl AsFd, source: impl AsFd) -> io::Result<()> {
    // SAFETY: We're calling `epoll_ctl` via FFI and we know how it
    // behaves.
    unsafe {
        let raw_fd = source.as_fd().as_raw_fd();
        syscalls::epoll_del(epoll.as_fd(), raw_fd)
    }
}

/// `epoll_wait(self, events, timeout)`—Waits for registered events of
/// interest.
///
/// For each event of interest, an element is written to `events`. On
/// success, this returns the number of written elements.
#[inline]
pub fn wait(epoll: impl AsFd, event_list: &mut EventVec, timeout: c::c_int) -> io::Result<()> {
    // SAFETY: We're calling `epoll_wait` via FFI and we know how it
    // behaves.
    unsafe {
        event_list.events.set_len(0);
        let nfds = syscalls::epoll_wait(
            epoll.as_fd(),
            event_list.events[..].as_mut_ptr().cast(),
            event_list.events.capacity(),
            timeout,
        )?;
        event_list.events.set_len(nfds);
    }

    Ok(())
}

/// An iterator over the `Event`s in an `EventVec`.
pub struct Iter<'a> {
    /// Use `Copied` to copy the struct, since `Event` is `packed` on some
    /// platforms, and it's common for users to directly destructure it,
    /// which would lead to errors about forming references to packed fields.
    iter: core::iter::Copied<slice::Iter<'a, Event>>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = Event;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
    }
}

/// A record of an event that occurred.
#[repr(C)]
#[cfg_attr(target_arch = "x86_64", repr(packed))]
#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub struct Event {
    /// Which specific event(s) occurred.
    pub flags: EventFlags,
    /// User data.
    pub data: EventData,
}

/// Data assocated with an [`Event`]. This can either be a 64-bit integer value
/// or a pointer which preserves pointer provenance.
#[repr(C)]
#[derive(Copy, Clone)]
pub union EventData {
    /// A 64-bit integer value.
    as_u64: u64,

    /// A `*mut c_void` which preserves pointer provenance, extended to be
    /// 64-bit so that if we read the value as a `u64` union field, we don't
    /// get uninitialized memory.
    sixty_four_bit_pointer: SixtyFourBitPointer,
}

impl EventData {
    /// Construct a new value containing a `u64`.
    #[inline]
    pub const fn new_u64(value: u64) -> Self {
        Self { as_u64: value }
    }

    /// Construct a new value containing a `*mut c_void`.
    #[inline]
    pub const fn new_ptr(value: *mut c_void) -> Self {
        Self {
            sixty_four_bit_pointer: SixtyFourBitPointer {
                pointer: value,
                #[cfg(target_pointer_width = "32")]
                _padding: 0,
            },
        }
    }

    /// Return the value as a `u64`.
    ///
    /// If the stored value was a pointer, the pointer is zero-extended to
    /// a `u64`.
    #[inline]
    pub fn u64(self) -> u64 {
        unsafe { self.as_u64 }
    }

    /// Return the value as a `*mut c_void`.
    ///
    /// If the stored value was a `u64`, the least-significant bits of the
    /// `u64` are returned as a pointer value.
    #[inline]
    pub fn ptr(self) -> *mut c_void {
        unsafe { self.sixty_four_bit_pointer.pointer }
    }
}

impl PartialEq for EventData {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.u64() == other.u64()
    }
}

impl Eq for EventData {}

impl Hash for EventData {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.u64().hash(state)
    }
}

#[repr(C)]
#[derive(Copy, Clone)]
struct SixtyFourBitPointer {
    #[cfg(target_endian = "big")]
    #[cfg(target_pointer_width = "32")]
    _padding: u32,

    pointer: *mut c_void,

    #[cfg(target_endian = "little")]
    #[cfg(target_pointer_width = "32")]
    _padding: u32,
}

/// A vector of `Event`s, plus context for interpreting them.
pub struct EventVec {
    events: Vec<Event>,
}

impl EventVec {
    /// Constructs an `EventVec` from raw pointer, length, and capacity.
    ///
    /// # Safety
    ///
    /// This function calls [`Vec::from_raw_parts`] with its arguments.
    ///
    /// [`Vec::from_raw_parts`]: https://doc.rust-lang.org/stable/std/vec/struct.Vec.html#method.from_raw_parts
    #[inline]
    pub unsafe fn from_raw_parts(ptr: *mut Event, len: usize, capacity: usize) -> Self {
        Self {
            events: Vec::from_raw_parts(ptr, len, capacity),
        }
    }

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
            iter: self.events.iter().copied(),
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
    type Item = Event;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

#[test]
fn test_epoll_layouts() {
    check_renamed_type!(Event, epoll_event);
    check_renamed_type!(Event, epoll_event);
    check_renamed_struct_renamed_field!(Event, epoll_event, flags, events);
    check_renamed_struct_renamed_field!(Event, epoll_event, data, data);
}
