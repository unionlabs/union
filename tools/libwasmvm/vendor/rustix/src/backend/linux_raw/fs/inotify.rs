//! inotify support for working with inotifies

use super::super::c;
use crate::backend::fs::syscalls;
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io;
use bitflags::bitflags;

bitflags! {
    /// `IN_*` for use with [`inotify_init`].
    ///
    /// [`inotify_init`]: crate::fs::inotify::inotify_init
    pub struct CreateFlags: c::c_uint {
        /// `IN_CLOEXEC`
        const CLOEXEC = linux_raw_sys::general::IN_CLOEXEC;
        /// `IN_NONBLOCK`
        const NONBLOCK = linux_raw_sys::general::IN_NONBLOCK;
    }
}

bitflags! {
    /// `IN*` for use with [`inotify_add_watch`].
    ///
    /// [`inotify_add_watch`]: crate::fs::inotify::inotify_add_watch
    #[derive(Default)]
    pub struct WatchFlags: c::c_uint {
        /// `IN_ACCESS`
        const ACCESS = linux_raw_sys::general::IN_ACCESS;
        /// `IN_ATTRIB`
        const ATTRIB = linux_raw_sys::general::IN_ATTRIB;
        /// `IN_CLOSE_NOWRITE`
        const CLOSE_NOWRITE = linux_raw_sys::general::IN_CLOSE_NOWRITE;
        /// `IN_CLOSE_WRITE`
        const CLOSE_WRITE = linux_raw_sys::general::IN_CLOSE_WRITE;
        /// `IN_CREATE `
        const CREATE = linux_raw_sys::general::IN_CREATE;
        /// `IN_DELETE`
        const DELETE = linux_raw_sys::general::IN_DELETE;
        /// `IN_DELETE_SELF`
        const DELETE_SELF = linux_raw_sys::general::IN_DELETE_SELF;
        /// `IN_MODIFY`
        const MODIFY = linux_raw_sys::general::IN_MODIFY;
        /// `IN_MOVE_SELF`
        const MOVE_SELF = linux_raw_sys::general::IN_MOVE_SELF;
        /// `IN_MOVED_FROM`
        const MOVED_FROM = linux_raw_sys::general::IN_MOVED_FROM;
        /// `IN_MOVED_TO`
        const MOVED_TO = linux_raw_sys::general::IN_MOVED_TO;
        /// `IN_OPEN`
        const OPEN = linux_raw_sys::general::IN_OPEN;

        /// `IN_CLOSE`
        const CLOSE = linux_raw_sys::general::IN_CLOSE;
        /// `IN_MOVE`
        const MOVE = linux_raw_sys::general::IN_MOVE;
        /// `IN_ALL_EVENTS`
        const ALL_EVENTS = linux_raw_sys::general::IN_ALL_EVENTS;

        /// `IN_DONT_FOLLOW`
        const DONT_FOLLOW = linux_raw_sys::general::IN_DONT_FOLLOW;
        /// `IN_EXCL_UNLINK`
        const EXCL_UNLINK = linux_raw_sys::general::IN_EXCL_UNLINK;
        /// `IN_MASK_ADD`
        const MASK_ADD = linux_raw_sys::general::IN_MASK_ADD;
        /// `IN_MASK_CREATE`
        const MASK_CREATE = linux_raw_sys::general::IN_MASK_CREATE;
        /// `IN_ONESHOT`
        const ONESHOT = linux_raw_sys::general::IN_ONESHOT;
        /// `IN_ONLYDIR`
        const ONLYDIR = linux_raw_sys::general::IN_ONLYDIR;
    }
}

/// `inotify_init1(flags)`—Creates a new inotify object.
///
/// Use the [`CreateFlags::CLOEXEC`] flag to prevent the resulting file
/// descriptor from being implicitly passed across `exec` boundaries.
#[doc(alias = "inotify_init1")]
#[inline]
pub fn inotify_init(flags: CreateFlags) -> io::Result<OwnedFd> {
    syscalls::inotify_init1(flags)
}

/// `inotify_add_watch(self, path, flags)`—Adds a watch to inotify.
///
/// This registers or updates a watch for the filesystem path `path`
/// and returns a watch descriptor corresponding to this watch.
///
/// Note: Due to the existence of hardlinks, providing two
/// different paths to this method may result in it returning
/// the same watch descriptor. An application should keep track of this
/// externally to avoid logic errors.
#[inline]
pub fn inotify_add_watch<P: crate::path::Arg>(
    inot: BorrowedFd<'_>,
    path: P,
    flags: WatchFlags,
) -> io::Result<i32> {
    let path = path.as_cow_c_str().unwrap();
    syscalls::inotify_add_watch(inot, &path, flags)
}

/// `inotify_rm_watch(self, wd)`—Removes a watch from this inotify
///
/// The watch descriptor provided should have previously been returned
/// by [`inotify_add_watch`] and not previously have been removed.
#[doc(alias = "inotify_rm_watch")]
#[inline]
pub fn inotify_remove_watch(inot: BorrowedFd<'_>, wd: i32) -> io::Result<()> {
    syscalls::inotify_rm_watch(inot, wd)
}
