//! Bindings to wepoll (Windows).

use std::convert::TryInto;
use std::io;
use std::os::windows::io::RawSocket;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

use wepoll_ffi as we;
use winapi::ctypes;

use crate::Event;

/// Calls a wepoll function and results in `io::Result`.
macro_rules! wepoll {
    ($fn:ident $args:tt) => {{
        let res = unsafe { we::$fn $args };
        if res == -1 {
            Err(std::io::Error::last_os_error())
        } else {
            Ok(res)
        }
    }};
}

/// Interface to wepoll.
#[derive(Debug)]
pub struct Poller {
    handle: we::HANDLE,
    notified: AtomicBool,
}

unsafe impl Send for Poller {}
unsafe impl Sync for Poller {}

impl Poller {
    /// Creates a new poller.
    pub fn new() -> io::Result<Poller> {
        let handle = unsafe { we::epoll_create1(0) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let notified = AtomicBool::new(false);
        log::trace!("new: handle={:?}", handle);
        Ok(Poller { handle, notified })
    }

    /// Adds a socket.
    pub fn add(&self, sock: RawSocket, ev: Event) -> io::Result<()> {
        log::trace!("add: handle={:?}, sock={}, ev={:?}", self.handle, sock, ev);
        self.ctl(we::EPOLL_CTL_ADD, sock, Some(ev))
    }

    /// Modifies a socket.
    pub fn modify(&self, sock: RawSocket, ev: Event) -> io::Result<()> {
        log::trace!(
            "modify: handle={:?}, sock={}, ev={:?}",
            self.handle,
            sock,
            ev
        );
        self.ctl(we::EPOLL_CTL_MOD, sock, Some(ev))
    }

    /// Deletes a socket.
    pub fn delete(&self, sock: RawSocket) -> io::Result<()> {
        log::trace!("remove: handle={:?}, sock={}", self.handle, sock);
        self.ctl(we::EPOLL_CTL_DEL, sock, None)
    }

    /// Waits for I/O events with an optional timeout.
    ///
    /// Returns the number of processed I/O events.
    ///
    /// If a notification occurs, this method will return but the notification event will not be
    /// included in the `events` list nor contribute to the returned count.
    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        log::trace!("wait: handle={:?}, timeout={:?}", self.handle, timeout);
        let deadline = timeout.map(|t| Instant::now() + t);

        loop {
            // Convert the timeout to milliseconds.
            let timeout_ms = match deadline.map(|d| d.saturating_duration_since(Instant::now())) {
                None => -1,
                Some(t) => {
                    // Round up to a whole millisecond.
                    let mut ms = t.as_millis().try_into().unwrap_or(std::u64::MAX);
                    if Duration::from_millis(ms) < t {
                        ms = ms.saturating_add(1);
                    }
                    ms.try_into().unwrap_or(std::i32::MAX)
                }
            };

            // Wait for I/O events.
            events.len = wepoll!(epoll_wait(
                self.handle,
                events.list.as_mut_ptr(),
                events.list.len() as ctypes::c_int,
                timeout_ms,
            ))? as usize;
            log::trace!("new events: handle={:?}, len={}", self.handle, events.len);

            // Break if there was a notification or at least one event, or if deadline is reached.
            if self.notified.swap(false, Ordering::SeqCst) || events.len > 0 || timeout_ms == 0 {
                break;
            }
        }

        Ok(())
    }

    /// Sends a notification to wake up the current or next `wait()` call.
    pub fn notify(&self) -> io::Result<()> {
        log::trace!("notify: handle={:?}", self.handle);

        if self
            .notified
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_ok()
        {
            unsafe {
                // This call errors if a notification has already been posted, but that's okay - we
                // can just ignore the error.
                //
                // The original wepoll does not support notifications triggered this way, which is
                // why wepoll-sys includes a small patch to support them.
                winapi::um::ioapiset::PostQueuedCompletionStatus(
                    self.handle as winapi::um::winnt::HANDLE,
                    0,
                    0,
                    ptr::null_mut(),
                );
            }
        }
        Ok(())
    }

    /// Passes arguments to `epoll_ctl`.
    fn ctl(&self, op: u32, sock: RawSocket, ev: Option<Event>) -> io::Result<()> {
        let mut ev = ev.map(|ev| {
            let mut flags = we::EPOLLONESHOT;
            if ev.readable {
                flags |= READ_FLAGS;
            }
            if ev.writable {
                flags |= WRITE_FLAGS;
            }
            we::epoll_event {
                events: flags as u32,
                data: we::epoll_data { u64_: ev.key as u64 },
            }
        });
        wepoll!(epoll_ctl(
            self.handle,
            op as ctypes::c_int,
            sock as we::SOCKET,
            ev.as_mut()
                .map(|ev| ev as *mut we::epoll_event)
                .unwrap_or(ptr::null_mut()),
        ))?;
        Ok(())
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        log::trace!("drop: handle={:?}", self.handle);
        unsafe {
            we::epoll_close(self.handle);
        }
    }
}

/// Wepoll flags for all possible readability events.
const READ_FLAGS: u32 = we::EPOLLIN | we::EPOLLRDHUP | we::EPOLLHUP | we::EPOLLERR | we::EPOLLPRI;

/// Wepoll flags for all possible writability events.
const WRITE_FLAGS: u32 = we::EPOLLOUT | we::EPOLLHUP | we::EPOLLERR;

/// A list of reported I/O events.
pub struct Events {
    list: Box<[we::epoll_event]>,
    len: usize,
}

unsafe impl Send for Events {}

impl Events {
    /// Creates an empty list.
    pub fn new() -> Events {
        let ev = we::epoll_event {
            events: 0,
            data: we::epoll_data { u64_: 0 },
        };
        Events {
            list: vec![ev; 1000].into_boxed_slice(),
            len: 0,
        }
    }

    /// Iterates over I/O events.
    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        self.list[..self.len].iter().map(|ev| Event {
            key: unsafe { ev.data.u64_ } as usize,
            readable: (ev.events & READ_FLAGS) != 0,
            writable: (ev.events & WRITE_FLAGS) != 0,
        })
    }
}
