//! Functions which operate on file descriptors which might be terminals.

use crate::backend;
use backend::fd::AsFd;
#[cfg(feature = "procfs")]
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
use {
    crate::ffi::CString, crate::io, crate::path::SMALL_PATH_BUFFER_SIZE, alloc::vec::Vec,
    backend::fd::BorrowedFd,
};

/// `isatty(fd)`—Tests whether a file descriptor refers to a terminal.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/isatty.html
/// [Linux]: https://man7.org/linux/man-pages/man3/isatty.3.html
#[inline]
pub fn isatty<Fd: AsFd>(fd: Fd) -> bool {
    backend::termios::syscalls::isatty(fd.as_fd())
}

/// `ttyname_r(fd)`
///
/// If `reuse` already has available capacity, reuse it if possible.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/ttyname.html
/// [Linux]: https://man7.org/linux/man-pages/man3/ttyname.3.html
#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[cfg(feature = "procfs")]
#[cfg_attr(doc_cfg, doc(cfg(feature = "procfs")))]
#[doc(alias = "ttyname_r")]
#[inline]
pub fn ttyname<Fd: AsFd, B: Into<Vec<u8>>>(dirfd: Fd, reuse: B) -> io::Result<CString> {
    _ttyname(dirfd.as_fd(), reuse.into())
}

#[cfg(not(any(target_os = "fuchsia", target_os = "wasi")))]
#[cfg(feature = "procfs")]
#[allow(unsafe_code)]
fn _ttyname(dirfd: BorrowedFd<'_>, mut buffer: Vec<u8>) -> io::Result<CString> {
    buffer.clear();
    buffer.reserve(SMALL_PATH_BUFFER_SIZE);

    loop {
        match backend::termios::syscalls::ttyname(dirfd, buffer.spare_capacity_mut()) {
            Err(io::Errno::RANGE) => {
                buffer.reserve(buffer.capacity() + 1); // use `Vec` reallocation strategy to grow capacity exponentially
            }
            Ok(len) => {
                // SAFETY: assume the backend returns the length of the string
                unsafe {
                    buffer.set_len(len);
                }

                // SAFETY:
                // - "ttyname_r stores this pathname in the buffer buf"
                // - [POSIX definition 3.271: Pathname]: "A string that is used to identify a
                //   file."
                // - [POSIX definition 3.375: String]: "A contiguous sequence of bytes
                //   terminated by and including the first null byte."
                //
                // Thus, there will be a single NUL byte at the end of the string.
                //
                // [POSIX definition 3.271: Pathname]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_271
                // [POSIX definition 3.375: String]: https://pubs.opengroup.org/onlinepubs/9699919799/basedefs/V1_chap03.html#tag_03_375
                unsafe {
                    return Ok(CString::from_vec_with_nul_unchecked(buffer));
                }
            }
            Err(errno) => return Err(errno),
        }
    }
}
