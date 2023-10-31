use crate::{backend, io};
use backend::fd::AsFd;

/// `fcntl(fd, F_RDADVISE, radvisory { offset, len })`
///
/// # References
///  - [Apple]
///
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
#[inline]
pub fn fcntl_rdadvise<Fd: AsFd>(fd: Fd, offset: u64, len: u64) -> io::Result<()> {
    backend::fs::syscalls::fcntl_rdadvise(fd.as_fd(), offset, len)
}

/// `fcntl(fd, F_FULLFSYNC)`
///
/// # References
///  - [Apple]
///
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/fcntl.2.html
#[inline]
pub fn fcntl_fullfsync<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::fs::syscalls::fcntl_fullfsync(fd.as_fd())
}
