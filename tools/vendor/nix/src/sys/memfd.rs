//! Interfaces for managing memory-backed files.

use std::os::unix::io::RawFd;
use crate::Result;
use crate::errno::Errno;
use std::ffi::CStr;

libc_bitflags!(
    /// Options that change the behavior of [`memfd_create`].
    pub struct MemFdCreateFlag: libc::c_uint {
        /// Set the close-on-exec ([`FD_CLOEXEC`]) flag on the new file descriptor.
        ///
        /// By default, the new file descriptor is set to remain open across an [`execve`]
        /// (the `FD_CLOEXEC` flag is initially disabled). This flag can be used to change
        /// this default. The file offset is set to the beginning of the file (see [`lseek`]).
        ///
        /// See also the description of the `O_CLOEXEC` flag in [`open(2)`].
        ///
        /// [`execve`]: crate::unistd::execve
        /// [`lseek`]: crate::unistd::lseek
        /// [`FD_CLOEXEC`]: crate::fcntl::FdFlag::FD_CLOEXEC
        /// [`open(2)`]: https://man7.org/linux/man-pages/man2/open.2.html
        MFD_CLOEXEC;
        /// Allow sealing operations on this file.
        ///
        /// See also the file sealing notes given in [`memfd_create(2)`].
        ///
        /// [`memfd_create(2)`]: https://man7.org/linux/man-pages/man2/memfd_create.2.html
        MFD_ALLOW_SEALING;
    }
);

/// Creates an anonymous file that lives in memory, and return a file-descriptor to it.
///
/// The file behaves like a regular file, and so can be modified, truncated, memory-mapped, and so on.
/// However, unlike a regular file, it lives in RAM and has a volatile backing storage.
///
/// For more information, see [`memfd_create(2)`].
///
/// [`memfd_create(2)`]: https://man7.org/linux/man-pages/man2/memfd_create.2.html
pub fn memfd_create(name: &CStr, flags: MemFdCreateFlag) -> Result<RawFd> {
    let res = unsafe {
        libc::syscall(libc::SYS_memfd_create, name.as_ptr(), flags.bits())
    };

    Errno::result(res).map(|r| r as RawFd)
}
