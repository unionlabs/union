//! Portable interface to epoll, kqueue, event ports, and wepoll.
//!
//! Supported platforms:
//! - [epoll](https://en.wikipedia.org/wiki/Epoll): Linux, Android
//! - [kqueue](https://en.wikipedia.org/wiki/Kqueue): macOS, iOS, FreeBSD, NetBSD, OpenBSD,
//!   DragonFly BSD
//! - [event ports](https://illumos.org/man/port_create): illumos, Solaris
//! - [poll](https://en.wikipedia.org/wiki/Poll_(Unix)): VxWorks, Fuchsia, other Unix systems
//! - [wepoll](https://github.com/piscisaureus/wepoll): Windows
//!
//! Polling is done in oneshot mode, which means interest in I/O events needs to be re-enabled
//! after an event is delivered if we're interested in the next event of the same kind.
//!
//! Only one thread can be waiting for I/O events at a time.
//!
//! # Examples
//!
//! ```no_run
//! use polling::{Event, Poller};
//! use std::net::TcpListener;
//!
//! // Create a TCP listener.
//! let socket = TcpListener::bind("127.0.0.1:8000")?;
//! socket.set_nonblocking(true)?;
//! let key = 7; // Arbitrary key identifying the socket.
//!
//! // Create a poller and register interest in readability on the socket.
//! let poller = Poller::new()?;
//! poller.add(&socket, Event::readable(key))?;
//!
//! // The event loop.
//! let mut events = Vec::new();
//! loop {
//!     // Wait for at least one I/O event.
//!     events.clear();
//!     poller.wait(&mut events, None)?;
//!
//!     for ev in &events {
//!         if ev.key == key {
//!             // Perform a non-blocking accept operation.
//!             socket.accept()?;
//!             // Set interest in the next readability event.
//!             poller.modify(&socket, Event::readable(key))?;
//!         }
//!     }
//! }
//! # std::io::Result::Ok(())
//! ```

#![cfg(feature = "std")]
#![cfg_attr(not(feature = "std"), no_std)]
#![warn(missing_docs, missing_debug_implementations, rust_2018_idioms)]

use std::fmt;
use std::io;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use std::usize;

use cfg_if::cfg_if;

/// Calls a libc function and results in `io::Result`.
#[cfg(unix)]
macro_rules! syscall {
    ($fn:ident $args:tt) => {{
        let res = unsafe { libc::$fn $args };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

cfg_if! {
    if #[cfg(any(target_os = "linux", target_os = "android"))] {
        mod epoll;
        use epoll as sys;
    } else if #[cfg(any(
        target_os = "illumos",
        target_os = "solaris",
    ))] {
        mod port;
        use port as sys;
    } else if #[cfg(any(
        target_os = "macos",
        target_os = "ios",
        target_os = "freebsd",
        target_os = "netbsd",
        target_os = "openbsd",
        target_os = "dragonfly",
    ))] {
        mod kqueue;
        use kqueue as sys;
    } else if #[cfg(any(
        target_os = "vxworks",
        target_os = "fuchsia",
        unix,
    ))] {
        mod poll;
        use poll as sys;
    } else if #[cfg(target_os = "windows")] {
        mod wepoll;
        use wepoll as sys;
    } else {
        compile_error!("polling does not support this target OS");
    }
}

/// Key associated with notifications.
const NOTIFY_KEY: usize = std::usize::MAX;

/// Indicates that a file descriptor or socket can read or write without blocking.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Event {
    /// Key identifying the file descriptor or socket.
    pub key: usize,
    /// Can it do a read operation without blocking?
    pub readable: bool,
    /// Can it do a write operation without blocking?
    pub writable: bool,
}

impl Event {
    /// All kinds of events (readable and writable).
    ///
    /// Equivalent to: `Event { key, readable: true, writable: true }`
    pub fn all(key: usize) -> Event {
        Event {
            key,
            readable: true,
            writable: true,
        }
    }

    /// Only the readable event.
    ///
    /// Equivalent to: `Event { key, readable: true, writable: false }`
    pub fn readable(key: usize) -> Event {
        Event {
            key,
            readable: true,
            writable: false,
        }
    }

    /// Only the writable event.
    ///
    /// Equivalent to: `Event { key, readable: false, writable: true }`
    pub fn writable(key: usize) -> Event {
        Event {
            key,
            readable: false,
            writable: true,
        }
    }

    /// No events.
    ///
    /// Equivalent to: `Event { key, readable: false, writable: false }`
    pub fn none(key: usize) -> Event {
        Event {
            key,
            readable: false,
            writable: false,
        }
    }
}

/// Waits for I/O events.
pub struct Poller {
    poller: sys::Poller,
    events: Mutex<sys::Events>,
    notified: AtomicBool,
}

impl Poller {
    /// Creates a new poller.
    ///
    /// # Examples
    ///
    /// ```
    /// use polling::Poller;
    ///
    /// let poller = Poller::new()?;
    /// # std::io::Result::Ok(())
    /// ```
    pub fn new() -> io::Result<Poller> {
        Ok(Poller {
            poller: sys::Poller::new()?,
            events: Mutex::new(sys::Events::new()),
            notified: AtomicBool::new(false),
        })
    }

    /// Adds a file descriptor or socket to the poller.
    ///
    /// A file descriptor or socket is considered readable or writable when a read or write
    /// operation on it would not block. This doesn't mean the read or write operation will
    /// succeed, it only means the operation will return immediately.
    ///
    /// If interest is set in both readability and writability, the two kinds of events might be
    /// delivered either separately or together.
    ///
    /// For example, interest in `Event { key: 7, readable: true, writable: true }` might result in
    /// a single [`Event`] of the same form, or in two separate [`Event`]s:
    /// - `Event { key: 7, readable: true, writable: false }`
    /// - `Event { key: 7, readable: false, writable: true }`
    ///
    /// Note that interest in I/O events needs to be re-enabled using
    /// [`modify()`][`Poller::modify()`] again after an event is delivered if we're interested in
    /// the next event of the same kind.
    ///
    /// Don't forget to [`delete()`][`Poller::delete()`] the file descriptor or socket when it is
    /// no longer used!
    ///
    /// # Errors
    ///
    /// This method returns an error in the following situations:
    ///
    /// * If `key` equals `usize::MAX` because that key is reserved for internal use.
    /// * If an error is returned by the syscall.
    ///
    /// # Examples
    ///
    /// Set interest in all events:
    ///
    /// ```no_run
    /// use polling::{Event, Poller};
    ///
    /// let source = std::net::TcpListener::bind("127.0.0.1:0")?;
    /// source.set_nonblocking(true)?;
    /// let key = 7;
    ///
    /// let poller = Poller::new()?;
    /// poller.add(&source, Event::all(key))?;
    /// # std::io::Result::Ok(())
    /// ```
    pub fn add(&self, source: impl Source, interest: Event) -> io::Result<()> {
        if interest.key == NOTIFY_KEY {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the key is not allowed to be `usize::MAX`",
            ));
        }
        self.poller.add(source.raw(), interest)
    }

    /// Modifies the interest in a file descriptor or socket.
    ///
    /// This method has the same behavior as [`add()`][`Poller::add()`] except it modifies the
    /// interest of a previously added file descriptor or socket.
    ///
    /// To use this method with a file descriptor or socket, you must first add it using
    /// [`add()`][`Poller::add()`].
    ///
    /// Note that interest in I/O events needs to be re-enabled using
    /// [`modify()`][`Poller::modify()`] again after an event is delivered if we're interested in
    /// the next event of the same kind.
    ///
    /// # Errors
    ///
    /// This method returns an error in the following situations:
    ///
    /// * If `key` equals `usize::MAX` because that key is reserved for internal use.
    /// * If an error is returned by the syscall.
    ///
    /// # Examples
    ///
    /// To enable interest in all events:
    ///
    /// ```no_run
    /// # use polling::{Event, Poller};
    /// # let source = std::net::TcpListener::bind("127.0.0.1:0")?;
    /// # let key = 7;
    /// # let poller = Poller::new()?;
    /// # poller.add(&source, Event::none(key))?;
    /// poller.modify(&source, Event::all(key))?;
    /// # std::io::Result::Ok(())
    /// ```
    ///
    /// To enable interest in readable events and disable interest in writable events:
    ///
    /// ```no_run
    /// # use polling::{Event, Poller};
    /// # let source = std::net::TcpListener::bind("127.0.0.1:0")?;
    /// # let key = 7;
    /// # let poller = Poller::new()?;
    /// # poller.add(&source, Event::none(key))?;
    /// poller.modify(&source, Event::readable(key))?;
    /// # std::io::Result::Ok(())
    /// ```
    ///
    /// To disable interest in readable events and enable interest in writable events:
    ///
    /// ```no_run
    /// # use polling::{Event, Poller};
    /// # let poller = Poller::new()?;
    /// # let key = 7;
    /// # let source = std::net::TcpListener::bind("127.0.0.1:0")?;
    /// # poller.add(&source, Event::none(key))?;
    /// poller.modify(&source, Event::writable(key))?;
    /// # std::io::Result::Ok(())
    /// ```
    ///
    /// To disable interest in all events:
    ///
    /// ```no_run
    /// # use polling::{Event, Poller};
    /// # let source = std::net::TcpListener::bind("127.0.0.1:0")?;
    /// # let key = 7;
    /// # let poller = Poller::new()?;
    /// # poller.add(&source, Event::none(key))?;
    /// poller.modify(&source, Event::none(key))?;
    /// # std::io::Result::Ok(())
    /// ```
    pub fn modify(&self, source: impl Source, interest: Event) -> io::Result<()> {
        if interest.key == NOTIFY_KEY {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the key is not allowed to be `usize::MAX`",
            ));
        }
        self.poller.modify(source.raw(), interest)
    }

    /// Removes a file descriptor or socket from the poller.
    ///
    /// Unlike [`add()`][`Poller::add()`], this method only removes the file descriptor or
    /// socket from the poller without putting it back into blocking mode.
    ///
    /// # Examples
    ///
    /// ```
    /// use polling::{Event, Poller};
    /// use std::net::TcpListener;
    ///
    /// let socket = TcpListener::bind("127.0.0.1:0")?;
    /// socket.set_nonblocking(true)?;
    /// let key = 7;
    ///
    /// let poller = Poller::new()?;
    /// poller.add(&socket, Event::all(key))?;
    /// poller.delete(&socket)?;
    /// # std::io::Result::Ok(())
    /// ```
    pub fn delete(&self, source: impl Source) -> io::Result<()> {
        self.poller.delete(source.raw())
    }

    /// Waits for at least one I/O event and returns the number of new events.
    ///
    /// New events will be appended to `events`. If necessary, make sure to clear the [`Vec`]
    /// before calling [`wait()`][`Poller::wait()`]!
    ///
    /// This method will return with no new events if a notification is delivered by the
    /// [`notify()`] method, or the timeout is reached. Sometimes it may even return with no events
    /// spuriously.
    ///
    /// Only one thread can wait on I/O. If another thread is already in [`wait()`], concurrent
    /// calls to this method will return immediately with no new events.
    ///
    /// If the operating system is ready to deliver a large number of events at once, this method
    /// may decide to deliver them in smaller batches.
    ///
    /// [`notify()`]: `Poller::notify()`
    /// [`wait()`]: `Poller::wait()`
    ///
    /// # Examples
    ///
    /// ```
    /// use polling::{Event, Poller};
    /// use std::net::TcpListener;
    /// use std::time::Duration;
    ///
    /// let socket = TcpListener::bind("127.0.0.1:0")?;
    /// socket.set_nonblocking(true)?;
    /// let key = 7;
    ///
    /// let poller = Poller::new()?;
    /// poller.add(&socket, Event::all(key))?;
    ///
    /// let mut events = Vec::new();
    /// let n = poller.wait(&mut events, Some(Duration::from_secs(1)))?;
    /// # std::io::Result::Ok(())
    /// ```
    pub fn wait(&self, events: &mut Vec<Event>, timeout: Option<Duration>) -> io::Result<usize> {
        log::trace!("Poller::wait(_, {:?})", timeout);

        if let Ok(mut lock) = self.events.try_lock() {
            // Wait for I/O events.
            self.poller.wait(&mut lock, timeout)?;

            // Clear the notification, if any.
            self.notified.swap(false, Ordering::SeqCst);

            // Collect events.
            let len = events.len();
            events.extend(lock.iter().filter(|ev| ev.key != usize::MAX));
            Ok(events.len() - len)
        } else {
            log::trace!("wait: skipping because another thread is already waiting on I/O");
            Ok(0)
        }
    }

    /// Wakes up the current or the following invocation of [`wait()`].
    ///
    /// If no thread is calling [`wait()`] right now, this method will cause the following call
    /// to wake up immediately.
    ///
    /// [`wait()`]: `Poller::wait()`
    ///
    /// # Examples
    ///
    /// ```
    /// use polling::Poller;
    ///
    /// let poller = Poller::new()?;
    ///
    /// // Notify the poller.
    /// poller.notify()?;
    ///
    /// let mut events = Vec::new();
    /// poller.wait(&mut events, None)?; // wakes up immediately
    /// assert!(events.is_empty());
    /// # std::io::Result::Ok(())
    /// ```
    pub fn notify(&self) -> io::Result<()> {
        log::trace!("Poller::notify()");
        if self
            .notified
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            self.poller.notify()?;
        }
        Ok(())
    }
}

impl fmt::Debug for Poller {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.poller.fmt(f)
    }
}

cfg_if! {
    if #[cfg(unix)] {
        use std::os::unix::io::{AsRawFd, RawFd};

        /// A [`RawFd`] or a reference to a type implementing [`AsRawFd`].
        pub trait Source {
            /// Returns the [`RawFd`] for this I/O object.
            fn raw(&self) -> RawFd;
        }

        impl Source for RawFd {
            fn raw(&self) -> RawFd {
                *self
            }
        }

        impl<T: AsRawFd> Source for &T {
            fn raw(&self) -> RawFd {
                self.as_raw_fd()
            }
        }
    } else if #[cfg(windows)] {
        use std::os::windows::io::{AsRawSocket, RawSocket};

        /// A [`RawSocket`] or a reference to a type implementing [`AsRawSocket`].
        pub trait Source {
            /// Returns the [`RawSocket`] for this I/O object.
            fn raw(&self) -> RawSocket;
        }

        impl Source for RawSocket {
            fn raw(&self) -> RawSocket {
                *self
            }
        }

        impl<T: AsRawSocket> Source for &T {
            fn raw(&self) -> RawSocket {
                self.as_raw_socket()
            }
        }
    }
}
