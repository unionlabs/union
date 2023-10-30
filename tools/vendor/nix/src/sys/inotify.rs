//! Monitoring API for filesystem events.
//!
//! Inotify is a Linux-only API to monitor filesystems events.
//!
//! For more documentation, please read [inotify(7)](https://man7.org/linux/man-pages/man7/inotify.7.html).
//!
//! # Examples
//!
//! Monitor all events happening in directory "test":
//! ```no_run
//! # use nix::sys::inotify::{AddWatchFlags,InitFlags,Inotify};
//! #
//! // We create a new inotify instance.
//! let instance = Inotify::init(InitFlags::empty()).unwrap();
//!
//! // We add a new watch on directory "test" for all events.
//! let wd = instance.add_watch("test", AddWatchFlags::IN_ALL_EVENTS).unwrap();
//!
//! loop {
//!     // We read from our inotify instance for events.
//!     let events = instance.read_events().unwrap();
//!     println!("Events: {:?}", events);
//! }
//! ```

use libc::{
    c_char,
    c_int,
};
use std::ffi::{OsString,OsStr,CStr};
use std::os::unix::ffi::OsStrExt;
use std::mem::{MaybeUninit, size_of};
use std::os::unix::io::{RawFd,AsRawFd,FromRawFd};
use std::ptr;
use crate::unistd::read;
use crate::Result;
use crate::NixPath;
use crate::errno::Errno;

libc_bitflags! {
    /// Configuration options for [`inotify_add_watch`](fn.inotify_add_watch.html).
    pub struct AddWatchFlags: u32 {
        IN_ACCESS;
        IN_MODIFY;
        IN_ATTRIB;
        IN_CLOSE_WRITE;
        IN_CLOSE_NOWRITE;
        IN_OPEN;
        IN_MOVED_FROM;
        IN_MOVED_TO;
        IN_CREATE;
        IN_DELETE;
        IN_DELETE_SELF;
        IN_MOVE_SELF;

        IN_UNMOUNT;
        IN_Q_OVERFLOW;
        IN_IGNORED;

        IN_CLOSE;
        IN_MOVE;

        IN_ONLYDIR;
        IN_DONT_FOLLOW;

        IN_ISDIR;
        IN_ONESHOT;
        IN_ALL_EVENTS;
    }
}

libc_bitflags! {
    /// Configuration options for [`inotify_init1`](fn.inotify_init1.html).
    pub struct InitFlags: c_int {
        IN_CLOEXEC;
        IN_NONBLOCK;
    }
}

/// An inotify instance. This is also a file descriptor, you can feed it to
/// other interfaces consuming file descriptors, epoll for example.
#[derive(Debug, Clone, Copy)]
pub struct Inotify {
    fd: RawFd
}

/// This object is returned when you create a new watch on an inotify instance.
/// It is then returned as part of an event once triggered. It allows you to
/// know which watch triggered which event.
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Ord, PartialOrd)]
pub struct WatchDescriptor {
    wd: i32
}

/// A single inotify event.
///
/// For more documentation see, [inotify(7)](https://man7.org/linux/man-pages/man7/inotify.7.html).
#[derive(Debug)]
pub struct InotifyEvent {
    /// Watch descriptor. This field corresponds to the watch descriptor you
    /// were issued when calling add_watch. It allows you to know which watch
    /// this event comes from.
    pub wd: WatchDescriptor,
    /// Event mask. This field is a bitfield describing the exact event that
    /// occured.
    pub mask: AddWatchFlags,
    /// This cookie is a number that allows you to connect related events. For
    /// now only IN_MOVED_FROM and IN_MOVED_TO can be connected.
    pub cookie: u32,
    /// Filename. This field exists only if the event was triggered for a file
    /// inside the watched directory.
    pub name: Option<OsString>
}

impl Inotify {
    /// Initialize a new inotify instance.
    ///
    /// Returns a Result containing an inotify instance.
    ///
    /// For more information see, [inotify_init(2)](https://man7.org/linux/man-pages/man2/inotify_init.2.html).
    pub fn init(flags: InitFlags) -> Result<Inotify> {
        let res = Errno::result(unsafe {
            libc::inotify_init1(flags.bits())
        });

        res.map(|fd| Inotify { fd })
    }

    /// Adds a new watch on the target file or directory.
    ///
    /// Returns a watch descriptor. This is not a File Descriptor!
    ///
    /// For more information see, [inotify_add_watch(2)](https://man7.org/linux/man-pages/man2/inotify_add_watch.2.html).
    pub fn add_watch<P: ?Sized + NixPath>(self,
                                          path: &P,
                                          mask: AddWatchFlags)
                                            -> Result<WatchDescriptor>
    {
        let res = path.with_nix_path(|cstr| {
            unsafe {
                libc::inotify_add_watch(self.fd, cstr.as_ptr(), mask.bits())
            }
        })?;

        Errno::result(res).map(|wd| WatchDescriptor { wd })
    }

    /// Removes an existing watch using the watch descriptor returned by
    /// inotify_add_watch.
    ///
    /// Returns an EINVAL error if the watch descriptor is invalid.
    ///
    /// For more information see, [inotify_rm_watch(2)](https://man7.org/linux/man-pages/man2/inotify_rm_watch.2.html).
    #[cfg(target_os = "linux")]
    pub fn rm_watch(self, wd: WatchDescriptor) -> Result<()> {
        let res = unsafe { libc::inotify_rm_watch(self.fd, wd.wd) };

        Errno::result(res).map(drop)
    }

    #[cfg(target_os = "android")]
    pub fn rm_watch(self, wd: WatchDescriptor) -> Result<()> {
        let res = unsafe { libc::inotify_rm_watch(self.fd, wd.wd as u32) };

        Errno::result(res).map(drop)
    }

    /// Reads a collection of events from the inotify file descriptor. This call
    /// can either be blocking or non blocking depending on whether IN_NONBLOCK
    /// was set at initialization.
    ///
    /// Returns as many events as available. If the call was non blocking and no
    /// events could be read then the EAGAIN error is returned.
    pub fn read_events(self) -> Result<Vec<InotifyEvent>> {
        let header_size = size_of::<libc::inotify_event>();
        const BUFSIZ: usize = 4096;
        let mut buffer = [0u8; BUFSIZ];
        let mut events = Vec::new();
        let mut offset = 0;

        let nread = read(self.fd, &mut buffer)?;

        while (nread - offset) >= header_size {
            let event = unsafe {
                let mut event = MaybeUninit::<libc::inotify_event>::uninit();
                ptr::copy_nonoverlapping(
                    buffer.as_ptr().add(offset),
                    event.as_mut_ptr() as *mut u8,
                    (BUFSIZ - offset).min(header_size)
                );
                event.assume_init()
            };

            let name = match event.len {
                0 => None,
                _ => {
                    let ptr = unsafe {
                        buffer
                            .as_ptr()
                            .add(offset + header_size)
                            as *const c_char
                    };
                    let cstr = unsafe { CStr::from_ptr(ptr) };

                    Some(OsStr::from_bytes(cstr.to_bytes()).to_owned())
                }
            };

            events.push(InotifyEvent {
                wd: WatchDescriptor { wd: event.wd },
                mask: AddWatchFlags::from_bits_truncate(event.mask),
                cookie: event.cookie,
                name
            });

            offset += header_size + event.len as usize;
        }

        Ok(events)
    }
}

impl AsRawFd for Inotify {
    fn as_raw_fd(&self) -> RawFd {
        self.fd
    }
}

impl FromRawFd for Inotify {
    unsafe fn from_raw_fd(fd: RawFd) -> Self {
        Inotify { fd }
    }
}
