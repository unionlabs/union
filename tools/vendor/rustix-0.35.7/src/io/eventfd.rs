use crate::backend;
use crate::io::{self, OwnedFd};

pub use backend::io::types::EventfdFlags;

/// `eventfd(initval, flags)`—Creates a file descriptor for event
/// notification.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/eventfd.2.html
#[inline]
pub fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    backend::io::syscalls::eventfd(initval, flags)
}
