//! Bindings to epoll (Linux, Android).

use std::convert::TryInto;
use std::io;
use std::os::unix::io::RawFd;
use std::ptr;
use std::time::Duration;

use crate::Event;

/// Interface to epoll.
#[derive(Debug)]
pub struct Poller {
    /// File descriptor for the epoll instance.
    epoll_fd: RawFd,
    /// File descriptor for the eventfd that produces notifications.
    event_fd: RawFd,
    /// File descriptor for the timerfd that produces timeouts.
    timer_fd: Option<RawFd>,
}

impl Poller {
    /// Creates a new poller.
    pub fn new() -> io::Result<Poller> {
        // Create an epoll instance.
        //
        // Use `epoll_create1` with `EPOLL_CLOEXEC`.
        let epoll_fd = syscall!(syscall(
            libc::SYS_epoll_create1,
            libc::EPOLL_CLOEXEC as libc::c_int
        ))
        .map(|fd| fd as libc::c_int)
        .or_else(|e| {
            match e.raw_os_error() {
                Some(libc::ENOSYS) => {
                    // If `epoll_create1` is not implemented, use `epoll_create`
                    // and manually set `FD_CLOEXEC`.
                    let fd = syscall!(epoll_create(1024))?;

                    if let Ok(flags) = syscall!(fcntl(fd, libc::F_GETFD)) {
                        let _ = syscall!(fcntl(fd, libc::F_SETFD, flags | libc::FD_CLOEXEC));
                    }

                    Ok(fd)
                }
                _ => Err(e),
            }
        })?;

        // Set up eventfd and timerfd.
        let event_fd = syscall!(eventfd(0, libc::EFD_CLOEXEC | libc::EFD_NONBLOCK))?;
        let timer_fd = syscall!(syscall(
            libc::SYS_timerfd_create,
            libc::CLOCK_MONOTONIC as libc::c_int,
            (libc::TFD_CLOEXEC | libc::TFD_NONBLOCK) as libc::c_int,
        ))
        .map(|fd| fd as libc::c_int)
        .ok();

        let poller = Poller {
            epoll_fd,
            event_fd,
            timer_fd,
        };

        if let Some(timer_fd) = timer_fd {
            poller.add(timer_fd, Event::none(crate::NOTIFY_KEY))?;
        }

        poller.add(
            event_fd,
            Event {
                key: crate::NOTIFY_KEY,
                readable: true,
                writable: false,
            },
        )?;

        log::trace!(
            "new: epoll_fd={}, event_fd={}, timer_fd={:?}",
            epoll_fd,
            event_fd,
            timer_fd
        );
        Ok(poller)
    }

    /// Adds a new file descriptor.
    pub fn add(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        log::trace!("add: epoll_fd={}, fd={}, ev={:?}", self.epoll_fd, fd, ev);
        self.ctl(libc::EPOLL_CTL_ADD, fd, Some(ev))
    }

    /// Modifies an existing file descriptor.
    pub fn modify(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        log::trace!("modify: epoll_fd={}, fd={}, ev={:?}", self.epoll_fd, fd, ev);
        self.ctl(libc::EPOLL_CTL_MOD, fd, Some(ev))
    }

    /// Deletes a file descriptor.
    pub fn delete(&self, fd: RawFd) -> io::Result<()> {
        log::trace!("remove: epoll_fd={}, fd={}", self.epoll_fd, fd);
        self.ctl(libc::EPOLL_CTL_DEL, fd, None)
    }

    /// Waits for I/O events with an optional timeout.
    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        log::trace!("wait: epoll_fd={}, timeout={:?}", self.epoll_fd, timeout);

        if let Some(timer_fd) = self.timer_fd {
            // Configure the timeout using timerfd.
            let new_val = libc::itimerspec {
                it_interval: TS_ZERO,
                it_value: match timeout {
                    None => TS_ZERO,
                    Some(t) => libc::timespec {
                        tv_sec: t.as_secs() as libc::time_t,
                        tv_nsec: (t.subsec_nanos() as libc::c_long).into(),
                    },
                },
            };

            syscall!(syscall(
                libc::SYS_timerfd_settime,
                timer_fd as libc::c_int,
                0 as libc::c_int,
                &new_val as *const libc::itimerspec,
                ptr::null_mut() as *mut libc::itimerspec
            ))?;

            // Set interest in timerfd.
            self.modify(
                timer_fd,
                Event {
                    key: crate::NOTIFY_KEY,
                    readable: true,
                    writable: false,
                },
            )?;
        }

        // Timeout in milliseconds for epoll.
        let timeout_ms = match (self.timer_fd, timeout) {
            (_, Some(t)) if t == Duration::from_secs(0) => 0,
            (None, Some(t)) => {
                // Round up to a whole millisecond.
                let mut ms = t.as_millis().try_into().unwrap_or(std::i32::MAX);
                if Duration::from_millis(ms as u64) < t {
                    ms = ms.saturating_add(1);
                }
                ms
            }
            _ => -1,
        };

        // Wait for I/O events.
        let res = syscall!(epoll_wait(
            self.epoll_fd,
            events.list.as_mut_ptr() as *mut libc::epoll_event,
            events.list.len() as libc::c_int,
            timeout_ms as libc::c_int,
        ))?;
        events.len = res as usize;
        log::trace!("new events: epoll_fd={}, res={}", self.epoll_fd, res);

        // Clear the notification (if received) and re-register interest in it.
        let mut buf = [0u8; 8];
        let _ = syscall!(read(
            self.event_fd,
            buf.as_mut_ptr() as *mut libc::c_void,
            buf.len()
        ));
        self.modify(
            self.event_fd,
            Event {
                key: crate::NOTIFY_KEY,
                readable: true,
                writable: false,
            },
        )?;
        Ok(())
    }

    /// Sends a notification to wake up the current or next `wait()` call.
    pub fn notify(&self) -> io::Result<()> {
        log::trace!(
            "notify: epoll_fd={}, event_fd={}",
            self.epoll_fd,
            self.event_fd
        );

        let buf: [u8; 8] = 1u64.to_ne_bytes();
        let _ = syscall!(write(
            self.event_fd,
            buf.as_ptr() as *const libc::c_void,
            buf.len()
        ));
        Ok(())
    }

    /// Passes arguments to `epoll_ctl`.
    fn ctl(&self, op: libc::c_int, fd: RawFd, ev: Option<Event>) -> io::Result<()> {
        let mut ev = ev.map(|ev| {
            let mut flags = libc::EPOLLONESHOT;
            if ev.readable {
                flags |= read_flags();
            }
            if ev.writable {
                flags |= write_flags();
            }
            libc::epoll_event {
                events: flags as _,
                u64: ev.key as u64,
            }
        });
        syscall!(epoll_ctl(
            self.epoll_fd,
            op,
            fd,
            ev.as_mut()
                .map(|ev| ev as *mut libc::epoll_event)
                .unwrap_or(ptr::null_mut()),
        ))?;
        Ok(())
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        log::trace!(
            "drop: epoll_fd={}, event_fd={}, timer_fd={:?}",
            self.epoll_fd,
            self.event_fd,
            self.timer_fd
        );

        if let Some(timer_fd) = self.timer_fd {
            let _ = self.delete(timer_fd);
            let _ = syscall!(close(timer_fd));
        }
        let _ = self.delete(self.event_fd);
        let _ = syscall!(close(self.event_fd));
        let _ = syscall!(close(self.epoll_fd));
    }
}

/// `timespec` value that equals zero.
const TS_ZERO: libc::timespec = libc::timespec {
    tv_sec: 0,
    tv_nsec: 0,
};

/// Epoll flags for all possible readability events.
fn read_flags() -> libc::c_int {
    libc::EPOLLIN | libc::EPOLLRDHUP | libc::EPOLLHUP | libc::EPOLLERR | libc::EPOLLPRI
}

/// Epoll flags for all possible writability events.
fn write_flags() -> libc::c_int {
    libc::EPOLLOUT | libc::EPOLLHUP | libc::EPOLLERR
}

/// A list of reported I/O events.
pub struct Events {
    list: Box<[libc::epoll_event]>,
    len: usize,
}

unsafe impl Send for Events {}

impl Events {
    /// Creates an empty list.
    pub fn new() -> Events {
        let ev = libc::epoll_event { events: 0, u64: 0 };
        let list = vec![ev; 1000].into_boxed_slice();
        let len = 0;
        Events { list, len }
    }

    /// Iterates over I/O events.
    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        self.list[..self.len].iter().map(|ev| Event {
            key: ev.u64 as usize,
            readable: (ev.events as libc::c_int & read_flags()) != 0,
            writable: (ev.events as libc::c_int & write_flags()) != 0,
        })
    }
}
