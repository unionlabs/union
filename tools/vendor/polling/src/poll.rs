//! Bindings to poll (VxWorks, Fuchsia, other Unix systems).

use std::collections::HashMap;
use std::convert::TryInto;
use std::fmt::{self, Debug, Formatter};
use std::io;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::{Duration, Instant};

// std::os::unix doesn't exist on Fuchsia
use libc::c_int as RawFd;

use crate::Event;

/// Interface to poll.
#[derive(Debug)]
pub struct Poller {
    /// File descriptors to poll.
    fds: Mutex<Fds>,

    /// The file descriptor of the read half of the notify pipe. This is also stored as the first
    /// file descriptor in `fds.poll_fds`.
    notify_read: RawFd,
    /// The file descriptor of the write half of the notify pipe.
    ///
    /// Data is written to this to wake up the current instance of `wait`, which can occur when the
    /// user notifies it (in which case `notified` would have been set) or when an operation needs
    /// to occur (in which case `waiting_operations` would have been incremented).
    notify_write: RawFd,

    /// The number of operations (`add`, `modify` or `delete`) that are currently waiting on the
    /// mutex to become free. When this is nonzero, `wait` must be suspended until it reaches zero
    /// again.
    waiting_operations: AtomicUsize,
    /// Whether `wait` has been notified by the user.
    notified: AtomicBool,
    /// The condition variable that gets notified when `waiting_operations` reaches zero or
    /// `notified` becomes true.
    ///
    /// This is used with the `fds` mutex.
    operations_complete: Condvar,
}

/// The file descriptors to poll in a `Poller`.
#[derive(Debug)]
struct Fds {
    /// The list of `pollfds` taken by poll.
    ///
    /// The first file descriptor is always present and is used to notify the poller. It is also
    /// stored in `notify_read`.
    poll_fds: Vec<PollFd>,
    /// The map of each file descriptor to data associated with it. This does not include the file
    /// descriptors `notify_read` or `notify_write`.
    fd_data: HashMap<RawFd, FdData>,
}

/// Transparent wrapper around `libc::pollfd`, used to support `Debug` derives without adding the
/// `extra_traits` feature of `libc`.
#[repr(transparent)]
struct PollFd(libc::pollfd);

impl Debug for PollFd {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.debug_struct("pollfd")
            .field("fd", &self.0.fd)
            .field("events", &self.0.events)
            .field("revents", &self.0.revents)
            .finish()
    }
}

/// Data associated with a file descriptor in a poller.
#[derive(Debug)]
struct FdData {
    /// The index into `poll_fds` this file descriptor is.
    poll_fds_index: usize,
    /// The key of the `Event` associated with this file descriptor.
    key: usize,
}

impl Poller {
    /// Creates a new poller.
    pub fn new() -> io::Result<Poller> {
        // Create the notification pipe.
        let mut notify_pipe = [0; 2];
        syscall!(pipe(notify_pipe.as_mut_ptr()))?;

        // Put the reading side into non-blocking mode.
        let notify_read_flags = syscall!(fcntl(notify_pipe[0], libc::F_GETFL))?;
        syscall!(fcntl(
            notify_pipe[0],
            libc::F_SETFL,
            notify_read_flags | libc::O_NONBLOCK
        ))?;

        log::trace!(
            "new: notify_read={}, notify_write={}",
            notify_pipe[0],
            notify_pipe[1]
        );

        Ok(Self {
            fds: Mutex::new(Fds {
                poll_fds: vec![PollFd(libc::pollfd {
                    fd: notify_pipe[0],
                    events: libc::POLLRDNORM,
                    revents: 0,
                })],
                fd_data: HashMap::new(),
            }),
            notify_read: notify_pipe[0],
            notify_write: notify_pipe[1],
            waiting_operations: AtomicUsize::new(0),
            operations_complete: Condvar::new(),
            notified: AtomicBool::new(false),
        })
    }

    /// Adds a new file descriptor.
    pub fn add(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        if fd == self.notify_read || fd == self.notify_write {
            return Err(io::Error::from(io::ErrorKind::InvalidInput));
        }

        log::trace!(
            "add: notify_read={}, fd={}, ev={:?}",
            self.notify_read,
            fd,
            ev
        );

        self.modify_fds(|fds| {
            if fds.fd_data.contains_key(&fd) {
                return Err(io::Error::from(io::ErrorKind::AlreadyExists));
            }

            let poll_fds_index = fds.poll_fds.len();
            fds.fd_data.insert(
                fd,
                FdData {
                    poll_fds_index,
                    key: ev.key,
                },
            );

            fds.poll_fds.push(PollFd(libc::pollfd {
                fd,
                events: poll_events(ev),
                revents: 0,
            }));

            Ok(())
        })
    }

    /// Modifies an existing file descriptor.
    pub fn modify(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        log::trace!(
            "modify: notify_read={}, fd={}, ev={:?}",
            self.notify_read,
            fd,
            ev
        );

        self.modify_fds(|fds| {
            let data = fds.fd_data.get_mut(&fd).ok_or(io::ErrorKind::NotFound)?;
            data.key = ev.key;
            let poll_fds_index = data.poll_fds_index;
            fds.poll_fds[poll_fds_index].0.events = poll_events(ev);

            Ok(())
        })
    }

    /// Deletes a file descriptor.
    pub fn delete(&self, fd: RawFd) -> io::Result<()> {
        log::trace!("delete: notify_read={}, fd={}", self.notify_read, fd);

        self.modify_fds(|fds| {
            let data = fds.fd_data.remove(&fd).ok_or(io::ErrorKind::NotFound)?;
            fds.poll_fds.swap_remove(data.poll_fds_index);
            if let Some(swapped_pollfd) = fds.poll_fds.get(data.poll_fds_index) {
                fds.fd_data
                    .get_mut(&swapped_pollfd.0.fd)
                    .unwrap()
                    .poll_fds_index = data.poll_fds_index;
            }

            Ok(())
        })
    }

    /// Waits for I/O events with an optional timeout.
    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        log::trace!(
            "wait: notify_read={}, timeout={:?}",
            self.notify_read,
            timeout
        );

        let deadline = timeout.map(|t| Instant::now() + t);

        events.inner.clear();

        let mut fds = self.fds.lock().unwrap();

        loop {
            // Complete all current operations.
            loop {
                if self.notified.swap(false, Ordering::SeqCst) {
                    // `notify` will have sent a notification in case we were polling. We weren't,
                    // so remove it.
                    return self.pop_notification();
                } else if self.waiting_operations.load(Ordering::SeqCst) == 0 {
                    break;
                }

                fds = self.operations_complete.wait(fds).unwrap();
            }

            // Perform the poll.
            let num_events = poll(&mut fds.poll_fds, deadline)?;
            let notified = fds.poll_fds[0].0.revents != 0;
            let num_fd_events = if notified { num_events - 1 } else { num_events };
            log::trace!(
                "new events: notify_read={}, num={}",
                self.notify_read,
                num_events
            );

            // Read all notifications.
            if notified {
                while syscall!(read(self.notify_read, &mut [0; 64] as *mut _ as *mut _, 64)).is_ok()
                {
                }
            }

            // If the only event that occurred during polling was notification and it wasn't to
            // exit, another thread is trying to perform an operation on the fds. Continue the
            // loop.
            if !self.notified.swap(false, Ordering::SeqCst) && num_fd_events == 0 && notified {
                continue;
            }

            // Store the events if there were any.
            if num_fd_events > 0 {
                let fds = &mut *fds;

                events.inner.reserve(num_fd_events);
                for fd_data in fds.fd_data.values_mut() {
                    let PollFd(poll_fd) = &mut fds.poll_fds[fd_data.poll_fds_index];
                    if poll_fd.revents != 0 {
                        // Store event
                        events.inner.push(Event {
                            key: fd_data.key,
                            readable: poll_fd.revents & READ_REVENTS != 0,
                            writable: poll_fd.revents & WRITE_REVENTS != 0,
                        });
                        // Remove interest
                        poll_fd.events = 0;

                        if events.inner.len() == num_fd_events {
                            break;
                        }
                    }
                }
            }

            break;
        }

        Ok(())
    }

    /// Sends a notification to wake up the current or next `wait()` call.
    pub fn notify(&self) -> io::Result<()> {
        log::trace!("notify: notify_read={}", self.notify_read);

        if !self.notified.swap(true, Ordering::SeqCst) {
            self.notify_inner()?;
            self.operations_complete.notify_one();
        }

        Ok(())
    }

    /// Perform a modification on `fds`, interrupting the current caller of `wait` if it's running.
    fn modify_fds(&self, f: impl FnOnce(&mut Fds) -> io::Result<()>) -> io::Result<()> {
        self.waiting_operations.fetch_add(1, Ordering::SeqCst);

        // Wake up the current caller of `wait` if there is one.
        let sent_notification = self.notify_inner().is_ok();

        let mut fds = self.fds.lock().unwrap();

        // If there was no caller of `wait` our notification was not removed from the pipe.
        if sent_notification {
            let _ = self.pop_notification();
        }

        let res = f(&mut *fds);

        if self.waiting_operations.fetch_sub(1, Ordering::SeqCst) == 1 {
            self.operations_complete.notify_one();
        }

        res
    }

    /// Wake the current thread that is calling `wait`.
    fn notify_inner(&self) -> io::Result<()> {
        syscall!(write(self.notify_write, &0_u8 as *const _ as *const _, 1))?;
        Ok(())
    }

    /// Remove a notification created by `notify_inner`.
    fn pop_notification(&self) -> io::Result<()> {
        syscall!(read(self.notify_read, &mut [0; 1] as *mut _ as *mut _, 1))?;
        Ok(())
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        log::trace!("drop: notify_read={}", self.notify_read);
        let _ = syscall!(close(self.notify_read));
        let _ = syscall!(close(self.notify_write));
    }
}

/// Get the input poll events for the given event.
fn poll_events(ev: Event) -> libc::c_short {
    (if ev.readable {
        libc::POLLIN | libc::POLLPRI
    } else {
        0
    }) | (if ev.writable {
        libc::POLLOUT | libc::POLLWRBAND
    } else {
        0
    })
}

/// Returned poll events for reading.
const READ_REVENTS: libc::c_short = libc::POLLIN | libc::POLLPRI | libc::POLLHUP | libc::POLLERR;

/// Returned poll events for writing.
const WRITE_REVENTS: libc::c_short =
    libc::POLLOUT | libc::POLLWRBAND | libc::POLLHUP | libc::POLLERR;

/// A list of reported I/O events.
pub struct Events {
    inner: Vec<Event>,
}

impl Events {
    /// Creates an empty list.
    pub fn new() -> Events {
        Self { inner: Vec::new() }
    }

    /// Iterates over I/O events.
    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        self.inner.iter().copied()
    }
}

/// Helper function to call poll.
fn poll(fds: &mut [PollFd], deadline: Option<Instant>) -> io::Result<usize> {
    loop {
        // Convert the timeout to milliseconds.
        let timeout_ms = deadline
            .map(|deadline| {
                let timeout = deadline.saturating_duration_since(Instant::now());

                // Round up to a whole millisecond.
                let mut ms = timeout.as_millis().try_into().unwrap_or(std::u64::MAX);
                if Duration::from_millis(ms) < timeout {
                    ms = ms.saturating_add(1);
                }
                ms.try_into().unwrap_or(std::i32::MAX)
            })
            .unwrap_or(-1);

        match syscall!(poll(
            fds.as_mut_ptr() as *mut libc::pollfd,
            fds.len() as libc::nfds_t,
            timeout_ms,
        )) {
            Ok(num_events) => break Ok(num_events as usize),
            // poll returns EAGAIN if we can retry it.
            Err(e) if e.raw_os_error() == Some(libc::EAGAIN) => continue,
            Err(e) => return Err(e),
        }
    }
}
