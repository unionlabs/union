//! libc syscalls supporting `rustix::io`.

use super::super::c;
#[cfg(any(target_os = "android", target_os = "linux"))]
use super::super::conv::syscall_ret_owned_fd;
use super::super::conv::{
    borrowed_fd, ret, ret_c_int, ret_discarded_fd, ret_owned_fd, ret_ssize_t,
};
use super::super::offset::{libc_pread, libc_pwrite};
#[cfg(not(target_os = "redox"))]
use super::super::offset::{libc_preadv, libc_pwritev};
#[cfg(all(target_os = "linux", target_env = "gnu"))]
use super::super::offset::{libc_preadv2, libc_pwritev2};
use crate::fd::{AsFd, BorrowedFd, RawFd};
#[cfg(not(target_os = "wasi"))]
use crate::io::DupFlags;
#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "wasi")))]
use crate::io::PipeFlags;
use crate::io::{self, IoSlice, IoSliceMut, OwnedFd, PollFd};
#[cfg(any(target_os = "android", target_os = "linux"))]
use crate::io::{EventfdFlags, ReadWriteFlags};
use core::cmp::min;
use core::convert::TryInto;
use core::mem::MaybeUninit;
#[cfg(feature = "net")]
use libc_errno::errno;

pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let nread = unsafe {
        ret_ssize_t(c::read(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            min(buf.len(), READ_LIMIT),
        ))?
    };
    Ok(nread as usize)
}

pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let nwritten = unsafe {
        ret_ssize_t(c::write(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            min(buf.len(), READ_LIMIT),
        ))?
    };
    Ok(nwritten as usize)
}

pub(crate) fn pread(fd: BorrowedFd<'_>, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    let len = min(buf.len(), READ_LIMIT);

    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;

    let nread = unsafe {
        ret_ssize_t(libc_pread(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            len,
            offset,
        ))?
    };
    Ok(nread as usize)
}

pub(crate) fn pwrite(fd: BorrowedFd<'_>, buf: &[u8], offset: u64) -> io::Result<usize> {
    let len = min(buf.len(), READ_LIMIT);

    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;

    let nwritten = unsafe {
        ret_ssize_t(libc_pwrite(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            len,
            offset,
        ))?
    };
    Ok(nwritten as usize)
}

pub(crate) fn readv(fd: BorrowedFd<'_>, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
    let nread = unsafe {
        ret_ssize_t(c::readv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
        ))?
    };
    Ok(nread as usize)
}

pub(crate) fn writev(fd: BorrowedFd<'_>, bufs: &[IoSlice]) -> io::Result<usize> {
    let nwritten = unsafe {
        ret_ssize_t(c::writev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
        ))?
    };
    Ok(nwritten as usize)
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn preadv(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut],
    offset: u64,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    let nread = unsafe {
        ret_ssize_t(libc_preadv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
        ))?
    };
    Ok(nread as usize)
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn pwritev(fd: BorrowedFd<'_>, bufs: &[IoSlice], offset: u64) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    let nwritten = unsafe {
        ret_ssize_t(libc_pwritev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
        ))?
    };
    Ok(nwritten as usize)
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn preadv2(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    let nread = unsafe {
        ret_ssize_t(libc_preadv2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
        ))?
    };
    Ok(nread as usize)
}

/// At present, `libc` only has `preadv2` defined for glibc. On other
/// ABIs, `ReadWriteFlags` has no flags defined, and we use plain `preadv`.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
#[inline]
pub(crate) fn preadv2(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    assert!(flags.is_empty());
    preadv(fd, bufs, offset)
}

#[cfg(all(target_os = "linux", target_env = "gnu"))]
pub(crate) fn pwritev2(
    fd: BorrowedFd<'_>,
    bufs: &[IoSlice],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    let nwritten = unsafe {
        ret_ssize_t(libc_pwritev2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
        ))?
    };
    Ok(nwritten as usize)
}

/// At present, `libc` only has `pwritev2` defined for glibc. On other
/// ABIs, `ReadWriteFlags` has no flags defined, and we use plain `pwritev`.
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
#[inline]
pub(crate) fn pwritev2(
    fd: BorrowedFd<'_>,
    bufs: &[IoSlice],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    assert!(flags.is_empty());
    pwritev(fd, bufs, offset)
}

// These functions are derived from Rust's library/std/src/sys/unix/fd.rs at
// revision a77da2d454e6caa227a85b16410b95f93495e7e0.

// The maximum read limit on most POSIX-like systems is `SSIZE_MAX`, with the
// man page quoting that if the count of bytes to read is greater than
// `SSIZE_MAX` the result is "unspecified".
//
// On macOS, however, apparently the 64-bit libc is either buggy or
// intentionally showing odd behavior by rejecting any read with a size larger
// than or equal to `INT_MAX`. To handle both of these the read size is capped
// on both platforms.
#[cfg(target_os = "macos")]
const READ_LIMIT: usize = c::c_int::MAX as usize - 1;
#[cfg(not(target_os = "macos"))]
const READ_LIMIT: usize = c::ssize_t::MAX as usize;

#[cfg(any(
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
))]
const fn max_iov() -> usize {
    c::IOV_MAX as usize
}

#[cfg(any(target_os = "android", target_os = "emscripten", target_os = "linux"))]
const fn max_iov() -> usize {
    c::UIO_MAXIOV as usize
}

#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "emscripten",
    target_os = "freebsd",
    target_os = "ios",
    target_os = "linux",
    target_os = "macos",
    target_os = "netbsd",
    target_os = "openbsd",
)))]
const fn max_iov() -> usize {
    16 // The minimum value required by POSIX.
}

pub(crate) unsafe fn close(raw_fd: RawFd) {
    let _ = c::close(raw_fd as c::c_int);
}

#[cfg(any(target_os = "android", target_os = "linux"))]
pub(crate) fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    unsafe { syscall_ret_owned_fd(c::syscall(c::SYS_eventfd2, initval, flags.bits())) }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(crate) fn ioctl_blksszget(fd: BorrowedFd) -> io::Result<u32> {
    let mut result = MaybeUninit::<c::c_uint>::uninit();
    unsafe {
        ret(c::ioctl(borrowed_fd(fd), c::BLKSSZGET, result.as_mut_ptr()))?;
        Ok(result.assume_init() as u32)
    }
}

#[cfg(any(target_os = "android", target_os = "linux"))]
#[inline]
pub(crate) fn ioctl_blkpbszget(fd: BorrowedFd) -> io::Result<u32> {
    let mut result = MaybeUninit::<c::c_uint>::uninit();
    unsafe {
        ret(c::ioctl(
            borrowed_fd(fd),
            c::BLKPBSZGET,
            result.as_mut_ptr(),
        ))?;
        Ok(result.assume_init() as u32)
    }
}

#[cfg(not(target_os = "redox"))]
pub(crate) fn ioctl_fionread(fd: BorrowedFd<'_>) -> io::Result<u64> {
    let mut nread = MaybeUninit::<c::c_int>::uninit();
    unsafe {
        ret(c::ioctl(borrowed_fd(fd), c::FIONREAD, nread.as_mut_ptr()))?;
        // `FIONREAD` returns the number of bytes silently casted to a `c_int`,
        // even when this is lossy. The best we can do is convert it back to a
        // `u64` without sign-extending it back first.
        Ok(u64::from(nread.assume_init() as c::c_uint))
    }
}

pub(crate) fn ioctl_fionbio(fd: BorrowedFd<'_>, value: bool) -> io::Result<()> {
    unsafe {
        let data = value as c::c_int;
        ret(c::ioctl(borrowed_fd(fd), c::FIONBIO, &data))
    }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
#[cfg(feature = "net")]
pub(crate) fn is_read_write(fd: BorrowedFd<'_>) -> io::Result<(bool, bool)> {
    let (mut read, mut write) = crate::fs::fd::_is_file_read_write(fd)?;
    let mut not_socket = false;
    if read {
        // Do a `recv` with `PEEK` and `DONTWAIT` for 1 byte. A 0 indicates
        // the read side is shut down; an `EWOULDBLOCK` indicates the read
        // side is still open.
        match unsafe {
            c::recv(
                borrowed_fd(fd),
                MaybeUninit::<[u8; 1]>::uninit()
                    .as_mut_ptr()
                    .cast::<c::c_void>(),
                1,
                c::MSG_PEEK | c::MSG_DONTWAIT,
            )
        } {
            0 => read = false,
            -1 => {
                #[allow(unreachable_patterns)] // `EAGAIN` may equal `EWOULDBLOCK`
                match errno().0 {
                    c::EAGAIN | c::EWOULDBLOCK => (),
                    c::ENOTSOCK => not_socket = true,
                    err => return Err(io::Errno(err)),
                }
            }
            _ => (),
        }
    }
    if write && !not_socket {
        // Do a `send` with `DONTWAIT` for 0 bytes. An `EPIPE` indicates
        // the write side is shut down.
        if unsafe { c::send(borrowed_fd(fd), [].as_ptr(), 0, c::MSG_DONTWAIT) } == -1 {
            #[allow(unreachable_patterns)] // `EAGAIN` may equal `EWOULDBLOCK`
            match errno().0 {
                c::EAGAIN | c::EWOULDBLOCK => (),
                c::ENOTSOCK => (),
                c::EPIPE => write = false,
                err => return Err(io::Errno(err)),
            }
        }
    }
    Ok((read, write))
}

#[cfg(target_os = "wasi")]
#[cfg(feature = "net")]
pub(crate) fn is_read_write(_fd: BorrowedFd<'_>) -> io::Result<(bool, bool)> {
    todo!("Implement is_read_write for WASI in terms of fd_fdstat_get");
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn dup(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::dup(borrowed_fd(fd))) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn dup2(fd: BorrowedFd<'_>, new: &mut OwnedFd) -> io::Result<()> {
    unsafe { ret_discarded_fd(c::dup2(borrowed_fd(fd), borrowed_fd(new.as_fd()))) }
}

#[cfg(not(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(crate) fn dup3(fd: BorrowedFd<'_>, new: &mut OwnedFd, flags: DupFlags) -> io::Result<()> {
    unsafe {
        ret_discarded_fd(c::dup3(
            borrowed_fd(fd),
            borrowed_fd(new.as_fd()),
            flags.bits(),
        ))
    }
}

#[cfg(any(
    target_os = "android",
    target_os = "dragonfly",
    target_os = "ios",
    target_os = "macos",
    target_os = "redox",
))]
pub(crate) fn dup3(fd: BorrowedFd<'_>, new: &mut OwnedFd, _flags: DupFlags) -> io::Result<()> {
    // Android 5.0 has `dup3`, but libc doesn't have bindings. Emulate it
    // using `dup2`. We don't need to worry about the difference between
    // `dup2` and `dup3` when the file descriptors are equal because we
    // have an `&mut OwnedFd` which means `fd` doesn't alias it.
    dup2(fd, new)
}

#[cfg(any(target_os = "ios", target_os = "macos"))]
pub(crate) fn ioctl_fioclex(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::FIOCLEX)) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocexcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCEXCL as _)) }
}

#[cfg(not(any(target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocnxcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCNXCL as _)) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(c::pipe(result.as_mut_ptr().cast::<i32>()))?;
        let [p0, p1] = result.assume_init();
        Ok((p0, p1))
    }
}

#[cfg(not(any(target_os = "ios", target_os = "macos", target_os = "wasi")))]
pub(crate) fn pipe_with(flags: PipeFlags) -> io::Result<(OwnedFd, OwnedFd)> {
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(c::pipe2(result.as_mut_ptr().cast::<i32>(), flags.bits()))?;
        let [p0, p1] = result.assume_init();
        Ok((p0, p1))
    }
}

#[inline]
pub(crate) fn poll(fds: &mut [PollFd<'_>], timeout: c::c_int) -> io::Result<usize> {
    let nfds = fds
        .len()
        .try_into()
        .map_err(|_convert_err| io::Errno::INVAL)?;

    ret_c_int(unsafe { c::poll(fds.as_mut_ptr().cast(), nfds, timeout) })
        .map(|nready| nready as usize)
}
