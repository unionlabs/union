//! Bindings to kqueue (macOS, iOS, FreeBSD, NetBSD, OpenBSD, DragonFly BSD).

use std::io::{self, Read, Write};
use std::os::unix::io::{AsRawFd, RawFd};
use std::os::unix::net::UnixStream;
use std::ptr;
use std::time::Duration;

use crate::Event;

/// Interface to kqueue.
#[derive(Debug)]
pub struct Poller {
    /// File descriptor for the kqueue instance.
    kqueue_fd: RawFd,
    /// Read side of a pipe for consuming notifications.
    read_stream: UnixStream,
    /// Write side of a pipe for producing notifications.
    write_stream: UnixStream,
}

impl Poller {
    /// Creates a new poller.
    pub fn new() -> io::Result<Poller> {
        // Create a kqueue instance.
        let kqueue_fd = syscall!(kqueue())?;
        syscall!(fcntl(kqueue_fd, libc::F_SETFD, libc::FD_CLOEXEC))?;

        // Set up the notification pipe.
        let (read_stream, write_stream) = UnixStream::pair()?;
        read_stream.set_nonblocking(true)?;
        write_stream.set_nonblocking(true)?;

        let poller = Poller {
            kqueue_fd,
            read_stream,
            write_stream,
        };
        poller.add(
            poller.read_stream.as_raw_fd(),
            Event {
                key: crate::NOTIFY_KEY,
                readable: true,
                writable: false,
            },
        )?;

        log::trace!(
            "new: kqueue_fd={}, read_stream={:?}",
            kqueue_fd,
            poller.read_stream
        );
        Ok(poller)
    }

    /// Adds a new file descriptor.
    pub fn add(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        // File descriptors don't need to be added explicitly, so just modify the interest.
        self.modify(fd, ev)
    }

    /// Modifies an existing file descriptor.
    pub fn modify(&self, fd: RawFd, ev: Event) -> io::Result<()> {
        if fd != self.read_stream.as_raw_fd() {
            log::trace!("add: kqueue_fd={}, fd={}, ev={:?}", self.kqueue_fd, fd, ev);
        }

        let read_flags = if ev.readable {
            libc::EV_ADD | libc::EV_ONESHOT
        } else {
            libc::EV_DELETE
        };
        let write_flags = if ev.writable {
            libc::EV_ADD | libc::EV_ONESHOT
        } else {
            libc::EV_DELETE
        };

        // A list of changes for kqueue.
        let changelist = [
            libc::kevent {
                ident: fd as _,
                filter: libc::EVFILT_READ,
                flags: read_flags | libc::EV_RECEIPT,
                fflags: 0,
                data: 0,
                udata: ev.key as _,
            },
            libc::kevent {
                ident: fd as _,
                filter: libc::EVFILT_WRITE,
                flags: write_flags | libc::EV_RECEIPT,
                fflags: 0,
                data: 0,
                udata: ev.key as _,
            },
        ];

        // Apply changes.
        let mut eventlist = changelist;
        syscall!(kevent(
            self.kqueue_fd,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as _,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as _,
            ptr::null(),
        ))?;

        // Check for errors.
        for ev in &eventlist {
            // Explanation for ignoring EPIPE: https://github.com/tokio-rs/mio/issues/582
            if (ev.flags & libc::EV_ERROR) != 0
                && ev.data != 0
                && ev.data != libc::ENOENT as _
                && ev.data != libc::EPIPE as _
            {
                return Err(io::Error::from_raw_os_error(ev.data as _));
            }
        }

        Ok(())
    }

    /// Deletes a file descriptor.
    pub fn delete(&self, fd: RawFd) -> io::Result<()> {
        // Simply delete interest in the file descriptor.
        self.modify(fd, Event::none(0))
    }

    /// Waits for I/O events with an optional timeout.
    pub fn wait(&self, events: &mut Events, timeout: Option<Duration>) -> io::Result<()> {
        log::trace!("wait: kqueue_fd={}, timeout={:?}", self.kqueue_fd, timeout);

        // Convert the `Duration` to `libc::timespec`.
        let timeout = timeout.map(|t| libc::timespec {
            tv_sec: t.as_secs() as libc::time_t,
            tv_nsec: t.subsec_nanos() as libc::c_long,
        });

        // Wait for I/O events.
        let changelist = [];
        let eventlist = &mut events.list;
        let res = syscall!(kevent(
            self.kqueue_fd,
            changelist.as_ptr() as *const libc::kevent,
            changelist.len() as _,
            eventlist.as_mut_ptr() as *mut libc::kevent,
            eventlist.len() as _,
            match &timeout {
                None => ptr::null(),
                Some(t) => t,
            }
        ))?;
        events.len = res as usize;
        log::trace!("new events: kqueue_fd={}, res={}", self.kqueue_fd, res);

        // Clear the notification (if received) and re-register interest in it.
        while (&self.read_stream).read(&mut [0; 64]).is_ok() {}
        self.modify(
            self.read_stream.as_raw_fd(),
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
        log::trace!("notify: kqueue_fd={}", self.kqueue_fd);
        let _ = (&self.write_stream).write(&[1]);
        Ok(())
    }
}

impl Drop for Poller {
    fn drop(&mut self) {
        log::trace!("drop: kqueue_fd={}", self.kqueue_fd);
        let _ = self.delete(self.read_stream.as_raw_fd());
        let _ = syscall!(close(self.kqueue_fd));
    }
}

/// A list of reported I/O events.
pub struct Events {
    list: Box<[libc::kevent]>,
    len: usize,
}

unsafe impl Send for Events {}

impl Events {
    /// Creates an empty list.
    pub fn new() -> Events {
        let ev = libc::kevent {
            ident: 0 as _,
            filter: 0,
            flags: 0,
            fflags: 0,
            data: 0,
            udata: 0 as _,
        };
        let list = vec![ev; 1000].into_boxed_slice();
        let len = 0;
        Events { list, len }
    }

    /// Iterates over I/O events.
    pub fn iter(&self) -> impl Iterator<Item = Event> + '_ {
        // On some platforms, closing the read end of a pipe wakes up writers, but the
        // event is reported as EVFILT_READ with the EV_EOF flag.
        //
        // https://github.com/golang/go/commit/23aad448b1e3f7c3b4ba2af90120bde91ac865b4
        self.list[..self.len].iter().map(|ev| Event {
            key: ev.udata as usize,
            readable: ev.filter == libc::EVFILT_READ,
            writable: ev.filter == libc::EVFILT_WRITE
                || (ev.filter == libc::EVFILT_READ && (ev.flags & libc::EV_EOF) != 0),
        })
    }
}
