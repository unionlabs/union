//! inotify support for working with inotifies

use super::super::c;
use super::super::conv::{borrowed_fd, c_str, ret, ret_c_int, ret_owned_fd};
use crate::fd::{BorrowedFd, OwnedFd};
use crate::io;
use bitflags::bitflags;

bitflags! {
    /// `IN_*` for use with [`inotify_init`].
    ///
    /// [`inotify_init`]: crate::fs::inotify::inotify_init
    pub struct CreateFlags: c::c_int {
        /// `IN_CLOEXEC`
        const CLOEXEC = c::IN_CLOEXEC;
        /// `IN_NONBLOCK`
        const NONBLOCK = c::IN_NONBLOCK;
    }
}

bitflags! {
    /// `IN*` for use with [`inotify_add_watch`].
    ///
    /// [`inotify_add_watch`]: crate::fs::inotify::inotify_add_watch
    #[derive(Default)]
    pub struct WatchFlags: u32 {
        /// `IN_ACCESS`
        const ACCESS = c::IN_ACCESS;
        /// `IN_ATTRIB`
        const ATTRIB = c::IN_ATTRIB;
        /// `IN_CLOSE_NOWRITE`
        const CLOSE_NOWRITE = c::IN_CLOSE_NOWRITE;
        /// `IN_CLOSE_WRITE`
        const CLOSE_WRITE = c::IN_CLOSE_WRITE;
        /// `IN_CREATE `
        const CREATE = c::IN_CREATE;
        /// `IN_DELETE`
        const DELETE = c::IN_DELETE;
        /// `IN_DELETE_SELF`
        const DELETE_SELF = c::IN_DELETE_SELF;
        /// `IN_MODIFY`
        const MODIFY = c::IN_MODIFY;
        /// `IN_MOVE_SELF`
        const MOVE_SELF = c::IN_MOVE_SELF;
        /// `IN_MOVED_FROM`
        const MOVED_FROM = c::IN_MOVED_FROM;
        /// `IN_MOVED_TO`
        const MOVED_TO = c::IN_MOVED_TO;
        /// `IN_OPEN`
        const OPEN = c::IN_OPEN;

        /// `IN_CLOSE`
        const CLOSE = c::IN_CLOSE;
        /// `IN_MOVE`
        const MOVE = c::IN_MOVE;
        /// `IN_ALL_EVENTS`
        const ALL_EVENTS = c::IN_ALL_EVENTS;

        /// `IN_DONT_FOLLOW`
        const DONT_FOLLOW = c::IN_DONT_FOLLOW;
        /// `IN_EXCL_UNLINK`
        const EXCL_UNLINK = 1;
        /// `IN_MASK_ADD`
        const MASK_ADD = 1;
        /// `IN_MASK_CREATE`
        const MASK_CREATE = 1;
        /// `IN_ONESHOT`
        const ONESHOT = c::IN_ONESHOT;
        /// `IN_ONLYDIR`
        const ONLYDIR = c::IN_ONLYDIR;
    }
}

/// `inotify_init1(flags)`—Creates a new inotify object.
///
/// Use the [`CreateFlags::CLOEXEC`] flag to prevent the resulting file
/// descriptor from being implicitly passed across `exec` boundaries.
#[doc(alias = "inotify_init1")]
pub fn inotify_init(flags: CreateFlags) -> io::Result<OwnedFd> {
    // SAFETY: `inotify_init1` has no safety preconditions.
    unsafe { ret_owned_fd(c::inotify_init1(flags.bits())) }
}

/// `inotify_add_watch(self, path, flags)`—Adds a watch to inotify
///
/// This registers or updates a watch for the filesystem path `path`
/// and returns a watch descriptor corresponding to this watch.
///
/// Note: Due to the existence of hardlinks, providing two
/// different paths to this method may result in it returning
/// the same watch descriptor. An application should keep track of this
/// externally to avoid logic errors.
pub fn inotify_add_watch<P: crate::path::Arg>(
    inot: BorrowedFd<'_>,
    path: P,
    flags: WatchFlags,
) -> io::Result<i32> {
    let path = path.as_cow_c_str().unwrap();
    // SAFETY: The fd and path we are passing is guaranteed valid by the type
    // system.
    unsafe {
        ret_c_int(c::inotify_add_watch(
            borrowed_fd(inot),
            c_str(&path),
            flags.bits(),
        ))
    }
}

/// `inotify_rm_watch(self, wd)`—Removes a watch from this inotify
///
/// The watch descriptor provided should have previously been returned
/// by [`inotify_add_watch`] and not previously have been removed.
#[doc(alias = "inotify_rm_watch")]
pub fn inotify_remove_watch(inot: BorrowedFd<'_>, wd: i32) -> io::Result<()> {
    // Android's `inotify_rm_watch` takes `u32` despite that
    // `inotify_add_watch` expects a `i32`.
    #[cfg(target_os = "android")]
    let wd = wd as u32;
    // SAFETY: The fd is valid and closing an arbitrary wd is valid.
    unsafe { ret(c::inotify_rm_watch(borrowed_fd(inot), wd)) }
}
