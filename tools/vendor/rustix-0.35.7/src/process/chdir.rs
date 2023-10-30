use crate::ffi::CString;
use crate::path::SMALL_PATH_BUFFER_SIZE;
use crate::{backend, io, path};
use alloc::vec::Vec;
#[cfg(not(target_os = "fuchsia"))]
use backend::fd::AsFd;

/// `chdir(path)`—Change the current working directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/chdir.html
/// [Linux]: https://man7.org/linux/man-pages/man2/chdir.2.html
#[inline]
pub fn chdir<P: path::Arg>(path: P) -> io::Result<()> {
    path.into_with_c_str(backend::process::syscalls::chdir)
}

/// `fchdir(fd)`—Change the current working directory.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/fchdir.html
/// [Linux]: https://man7.org/linux/man-pages/man2/fchdir.2.html
#[cfg(not(target_os = "fuchsia"))]
#[inline]
pub fn fchdir<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::process::syscalls::fchdir(fd.as_fd())
}

/// `getcwd()`—Return the current working directory.
///
/// If `reuse` is non-empty, reuse its buffer to store the result if possible.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/getcwd.html
/// [Linux]: https://man7.org/linux/man-pages/man3/getcwd.3.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn getcwd<B: Into<Vec<u8>>>(reuse: B) -> io::Result<CString> {
    _getcwd(reuse.into())
}

fn _getcwd(mut buffer: Vec<u8>) -> io::Result<CString> {
    // This code would benefit from having a better way to read into
    // uninitialized memory, but that requires `unsafe`.
    buffer.clear();
    buffer.reserve(SMALL_PATH_BUFFER_SIZE);
    buffer.resize(buffer.capacity(), 0_u8);

    loop {
        match backend::process::syscalls::getcwd(&mut buffer) {
            Err(io::Errno::RANGE) => {
                buffer.reserve(1); // use `Vec` reallocation strategy to grow capacity exponentially
                buffer.resize(buffer.capacity(), 0_u8);
            }
            Ok(_) => {
                let len = buffer.iter().position(|x| *x == b'\0').unwrap();
                buffer.resize(len, 0_u8);
                return Ok(CString::new(buffer).unwrap());
            }
            Err(errno) => return Err(errno),
        }
    }
}
