//! libc syscalls supporting `rustix::io`.

use crate::backend::conv::{
    borrowed_fd, ret, ret_c_int, ret_discarded_fd, ret_owned_fd, ret_usize,
};
use crate::backend::{c, MAX_IOV};
use crate::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
#[cfg(not(any(target_os = "aix", target_os = "wasi")))]
use crate::io::DupFlags;
#[cfg(linux_kernel)]
use crate::io::ReadWriteFlags;
use crate::io::{self, FdFlags, IoSlice, IoSliceMut};
use core::cmp::min;
use core::mem::MaybeUninit;
#[cfg(all(feature = "fs", feature = "net"))]
use libc_errno::errno;

pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::read(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            min(buf.len(), READ_LIMIT),
        ))
    }
}

pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::write(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            min(buf.len(), READ_LIMIT),
        ))
    }
}

pub(crate) fn pread(fd: BorrowedFd<'_>, buf: &mut [u8], offset: u64) -> io::Result<usize> {
    let len = min(buf.len(), READ_LIMIT);

    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;

    unsafe {
        ret_usize(c::pread(
            borrowed_fd(fd),
            buf.as_mut_ptr().cast(),
            len,
            offset,
        ))
    }
}

pub(crate) fn pwrite(fd: BorrowedFd<'_>, buf: &[u8], offset: u64) -> io::Result<usize> {
    let len = min(buf.len(), READ_LIMIT);

    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;

    unsafe { ret_usize(c::pwrite(borrowed_fd(fd), buf.as_ptr().cast(), len, offset)) }
}

pub(crate) fn readv(fd: BorrowedFd<'_>, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::readv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
        ))
    }
}

pub(crate) fn writev(fd: BorrowedFd<'_>, bufs: &[IoSlice]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::writev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
        ))
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
pub(crate) fn preadv(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut],
    offset: u64,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        ret_usize(c::preadv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
            offset,
        ))
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
pub(crate) fn pwritev(fd: BorrowedFd<'_>, bufs: &[IoSlice], offset: u64) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        ret_usize(c::pwritev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
            offset,
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn preadv2(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        ret_usize(c::preadv2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
            offset,
            bitflags_bits!(flags),
        ))
    }
}

#[cfg(linux_kernel)]
pub(crate) fn pwritev2(
    fd: BorrowedFd<'_>,
    bufs: &[IoSlice],
    offset: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        ret_usize(c::pwritev2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), MAX_IOV) as c::c_int,
            offset,
            bitflags_bits!(flags),
        ))
    }
}

// These functions are derived from Rust's library/std/src/sys/unix/fd.rs at
// revision 326ef470a8b379a180d6dc4bbef08990698a737a.

// The maximum read limit on most POSIX-like systems is `SSIZE_MAX`, with the
// manual page quoting that if the count of bytes to read is greater than
// `SSIZE_MAX` the result is “unspecified”.
//
// On macOS, however, apparently the 64-bit libc is either buggy or
// intentionally showing odd behavior by rejecting any read with a size larger
// than or equal to `INT_MAX`. To handle both of these the read size is capped
// on both platforms.
#[cfg(target_os = "macos")]
const READ_LIMIT: usize = c::c_int::MAX as usize - 1;
#[cfg(not(target_os = "macos"))]
const READ_LIMIT: usize = c::ssize_t::MAX as usize;

pub(crate) unsafe fn close(raw_fd: RawFd) {
    let _ = c::close(raw_fd as c::c_int);
}

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
#[cfg(all(feature = "fs", feature = "net"))]
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
                c::EAGAIN | c::EWOULDBLOCK | c::ENOTSOCK => (),
                c::EPIPE => write = false,
                err => return Err(io::Errno(err)),
            }
        }
    }
    Ok((read, write))
}

#[cfg(target_os = "wasi")]
#[cfg(all(feature = "fs", feature = "net"))]
pub(crate) fn is_read_write(_fd: BorrowedFd<'_>) -> io::Result<(bool, bool)> {
    todo!("Implement is_read_write for WASI in terms of fd_fdstat_get");
}

pub(crate) fn fcntl_getfd(fd: BorrowedFd<'_>) -> io::Result<FdFlags> {
    let flags = unsafe { ret_c_int(c::fcntl(borrowed_fd(fd), c::F_GETFD))? };
    Ok(FdFlags::from_bits_retain(bitcast!(flags)))
}

pub(crate) fn fcntl_setfd(fd: BorrowedFd<'_>, flags: FdFlags) -> io::Result<()> {
    unsafe { ret(c::fcntl(borrowed_fd(fd), c::F_SETFD, flags.bits())) }
}

#[cfg(not(target_os = "wasi"))]
pub(crate) fn fcntl_dupfd_cloexec(fd: BorrowedFd<'_>, min: RawFd) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::fcntl(borrowed_fd(fd), c::F_DUPFD_CLOEXEC, min)) }
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
    apple,
    target_os = "aix",
    target_os = "android",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
    target_os = "wasi",
)))]
pub(crate) fn dup3(fd: BorrowedFd<'_>, new: &mut OwnedFd, flags: DupFlags) -> io::Result<()> {
    unsafe {
        ret_discarded_fd(c::dup3(
            borrowed_fd(fd),
            borrowed_fd(new.as_fd()),
            bitflags_bits!(flags),
        ))
    }
}

#[cfg(any(
    apple,
    target_os = "android",
    target_os = "dragonfly",
    target_os = "haiku",
    target_os = "redox",
))]
pub(crate) fn dup3(fd: BorrowedFd<'_>, new: &mut OwnedFd, _flags: DupFlags) -> io::Result<()> {
    // Android 5.0 has `dup3`, but libc doesn't have bindings. Emulate it
    // using `dup2`. We don't need to worry about the difference between
    // `dup2` and `dup3` when the file descriptors are equal because we
    // have an `&mut OwnedFd` which means `fd` doesn't alias it.
    dup2(fd, new)
}

#[cfg(apple)]
pub(crate) fn ioctl_fioclex(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe {
        ret(c::ioctl(
            borrowed_fd(fd),
            c::FIOCLEX,
            core::ptr::null_mut::<u8>(),
        ))
    }
}
