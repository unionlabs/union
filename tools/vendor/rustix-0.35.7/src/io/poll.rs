use crate::{backend, io};

pub use backend::io::poll_fd::{PollFd, PollFlags};

/// `poll(self.fds, timeout)`
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [Winsock2]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/poll.html
/// [Linux]: https://man7.org/linux/man-pages/man2/poll.2.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/poll.2.html
/// [Winsock2]: https://docs.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-wsapoll
#[inline]
pub fn poll(fds: &mut [PollFd<'_>], timeout: i32) -> io::Result<usize> {
    backend::io::syscalls::poll(fds, timeout)
}
