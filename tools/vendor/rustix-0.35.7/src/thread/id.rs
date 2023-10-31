use crate::backend;
use crate::process::Pid;

/// `gettid()`—Returns the thread ID.
///
/// This returns the OS thread ID, which is not necessarily the same as the
/// `rust::thread::Thread::id` or the pthread ID.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/gettid.2.html
#[inline]
#[must_use]
pub fn gettid() -> Pid {
    backend::thread::syscalls::gettid()
}
