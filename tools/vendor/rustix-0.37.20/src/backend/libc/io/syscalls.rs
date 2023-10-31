//! libc syscalls supporting `rustix::io`.

use super::super::c;
#[cfg(any(
    target_os = "android",
    all(target_os = "linux", not(target_env = "gnu")),
))]
use super::super::conv::syscall_ret_usize;
use super::super::conv::{borrowed_fd, ret, ret_c_int, ret_discarded_fd, ret_owned_fd, ret_usize};
use super::super::offset::{libc_pread, libc_pwrite};
#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
use super::super::offset::{libc_preadv, libc_pwritev};
#[cfg(all(target_os = "linux", target_env = "gnu"))]
use super::super::offset::{libc_preadv2, libc_pwritev2};
use crate::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
#[cfg(not(any(target_os = "aix", target_os = "wasi")))]
use crate::io::DupFlags;
#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
use crate::io::EventfdFlags;
#[cfg(not(any(apple, target_os = "aix", target_os = "haiku", target_os = "wasi")))]
use crate::io::PipeFlags;
use crate::io::{self, FdFlags, IoSlice, IoSliceMut, PollFd};
use core::cmp::min;
use core::convert::TryInto;
use core::mem::MaybeUninit;
#[cfg(all(feature = "fs", feature = "net"))]
use libc_errno::errno;
#[cfg(linux_kernel)]
use {
    super::super::conv::syscall_ret_owned_fd,
    crate::io::{IoSliceRaw, ReadWriteFlags, SpliceFlags},
    crate::utils::optional_as_mut_ptr,
};
#[cfg(bsd)]
use {crate::io::kqueue::Event, crate::utils::as_ptr, core::ptr::null};
#[cfg(solarish)]
use {crate::io::port::Event, crate::utils::as_mut_ptr, core::ptr::null_mut};

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
        ret_usize(libc_pread(
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

    unsafe {
        ret_usize(libc_pwrite(
            borrowed_fd(fd),
            buf.as_ptr().cast(),
            len,
            offset,
        ))
    }
}

pub(crate) fn readv(fd: BorrowedFd<'_>, bufs: &mut [IoSliceMut]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::readv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
        ))
    }
}

pub(crate) fn writev(fd: BorrowedFd<'_>, bufs: &[IoSlice]) -> io::Result<usize> {
    unsafe {
        ret_usize(c::writev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
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
        ret_usize(libc_preadv(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
        ))
    }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "solaris")))]
pub(crate) fn pwritev(fd: BorrowedFd<'_>, bufs: &[IoSlice], offset: u64) -> io::Result<usize> {
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        ret_usize(libc_pwritev(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
        ))
    }
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
    unsafe {
        ret_usize(libc_preadv2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
        ))
    }
}

/// At present, `libc` only has `preadv2` defined for glibc. On other
/// ABIs, use `c::syscall`.
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
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        syscall_ret_usize(c::syscall(
            c::SYS_preadv2,
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
        ))
    }
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
    unsafe {
        ret_usize(libc_pwritev2(
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
        ))
    }
}

/// At present, `libc` only has `pwritev2` defined for glibc. On other
/// ABIs, use `c::syscall`.
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
    // Silently cast; we'll get `EINVAL` if the value is negative.
    let offset = offset as i64;
    unsafe {
        syscall_ret_usize(c::syscall(
            c::SYS_pwritev2,
            borrowed_fd(fd),
            bufs.as_ptr().cast::<c::iovec>(),
            min(bufs.len(), max_iov()) as c::c_int,
            offset,
            flags.bits(),
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

#[cfg(bsd)]
const fn max_iov() -> usize {
    c::IOV_MAX as usize
}

#[cfg(any(linux_kernel, target_os = "emscripten", target_os = "nto"))]
const fn max_iov() -> usize {
    c::UIO_MAXIOV as usize
}

#[cfg(not(any(
    bsd,
    linux_kernel,
    target_os = "emscripten",
    target_os = "nto",
    target_os = "horizon",
)))]
const fn max_iov() -> usize {
    16 // The minimum value required by POSIX.
}

pub(crate) unsafe fn close(raw_fd: RawFd) {
    let _ = c::close(raw_fd as c::c_int);
}

#[cfg(linux_kernel)]
pub(crate) fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    unsafe { syscall_ret_owned_fd(c::syscall(c::SYS_eventfd2, initval, flags.bits())) }
}

#[cfg(any(target_os = "freebsd", target_os = "illumos"))]
pub(crate) fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::eventfd(initval, flags.bits())) }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) fn ioctl_blksszget(fd: BorrowedFd) -> io::Result<u32> {
    let mut result = MaybeUninit::<c::c_uint>::uninit();
    unsafe {
        ret(c::ioctl(borrowed_fd(fd), c::BLKSSZGET, result.as_mut_ptr()))?;
        Ok(result.assume_init() as u32)
    }
}

#[cfg(linux_kernel)]
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

// Sparc lacks `FICLONE`.
#[cfg(all(linux_kernel, not(any(target_arch = "sparc", target_arch = "sparc64"))))]
pub(crate) fn ioctl_ficlone(fd: BorrowedFd<'_>, src_fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe {
        ret(c::ioctl(
            borrowed_fd(fd),
            c::FICLONE as _,
            borrowed_fd(src_fd),
        ))
    }
}

#[cfg(linux_kernel)]
#[inline]
pub(crate) fn ext4_ioc_resize_fs(fd: BorrowedFd<'_>, blocks: u64) -> io::Result<()> {
    // TODO: Fix linux-raw-sys to define ioctl codes for sparc.
    #[cfg(any(target_arch = "sparc", target_arch = "sparc64"))]
    const EXT4_IOC_RESIZE_FS: u32 = 0x8008_6610;

    #[cfg(not(any(target_arch = "sparc", target_arch = "sparc64")))]
    use linux_raw_sys::ioctl::EXT4_IOC_RESIZE_FS;

    unsafe { ret(c::ioctl(borrowed_fd(fd), EXT4_IOC_RESIZE_FS as _, &blocks)) }
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
#[cfg(all(feature = "fs", feature = "net"))]
pub(crate) fn is_read_write(_fd: BorrowedFd<'_>) -> io::Result<(bool, bool)> {
    todo!("Implement is_read_write for WASI in terms of fd_fdstat_get");
}

pub(crate) fn fcntl_getfd(fd: BorrowedFd<'_>) -> io::Result<FdFlags> {
    unsafe { ret_c_int(c::fcntl(borrowed_fd(fd), c::F_GETFD)).map(FdFlags::from_bits_truncate) }
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
            flags.bits(),
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

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocexcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCEXCL as _)) }
}

#[cfg(not(any(target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub(crate) fn ioctl_tiocnxcl(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::ioctl(borrowed_fd(fd), c::TIOCNXCL as _)) }
}

#[cfg(bsd)]
pub(crate) fn kqueue() -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::kqueue()) }
}

#[cfg(bsd)]
pub(crate) unsafe fn kevent(
    kq: BorrowedFd<'_>,
    changelist: &[Event],
    eventlist: &mut [MaybeUninit<Event>],
    timeout: Option<&c::timespec>,
) -> io::Result<c::c_int> {
    ret_c_int(c::kevent(
        borrowed_fd(kq),
        changelist.as_ptr().cast(),
        changelist
            .len()
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        eventlist.as_mut_ptr().cast(),
        eventlist
            .len()
            .try_into()
            .map_err(|_| io::Errno::OVERFLOW)?,
        timeout.map_or(null(), as_ptr),
    ))
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

#[cfg(not(any(apple, target_os = "aix", target_os = "haiku", target_os = "wasi")))]
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

#[cfg(linux_kernel)]
#[inline]
pub fn splice(
    fd_in: BorrowedFd,
    off_in: Option<&mut u64>,
    fd_out: BorrowedFd,
    off_out: Option<&mut u64>,
    len: usize,
    flags: SpliceFlags,
) -> io::Result<usize> {
    let off_in = optional_as_mut_ptr(off_in).cast();
    let off_out = optional_as_mut_ptr(off_out).cast();

    unsafe {
        ret_usize(c::splice(
            borrowed_fd(fd_in),
            off_in,
            borrowed_fd(fd_out),
            off_out,
            len,
            flags.bits(),
        ))
    }
}

#[cfg(linux_kernel)]
#[inline]
pub unsafe fn vmsplice(
    fd: BorrowedFd,
    bufs: &[IoSliceRaw],
    flags: SpliceFlags,
) -> io::Result<usize> {
    ret_usize(c::vmsplice(
        borrowed_fd(fd),
        bufs.as_ptr().cast::<c::iovec>(),
        min(bufs.len(), max_iov()),
        flags.bits(),
    ))
}

#[cfg(solarish)]
pub(crate) fn port_create() -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::port_create()) }
}

#[cfg(solarish)]
pub(crate) unsafe fn port_associate(
    port: BorrowedFd<'_>,
    source: c::c_int,
    object: c::uintptr_t,
    events: c::c_int,
    user: *mut c::c_void,
) -> io::Result<()> {
    ret(c::port_associate(
        borrowed_fd(port),
        source,
        object,
        events,
        user,
    ))
}

#[cfg(solarish)]
pub(crate) unsafe fn port_dissociate(
    port: BorrowedFd<'_>,
    source: c::c_int,
    object: c::uintptr_t,
) -> io::Result<()> {
    ret(c::port_dissociate(borrowed_fd(port), source, object))
}

#[cfg(solarish)]
pub(crate) fn port_get(
    port: BorrowedFd<'_>,
    timeout: Option<&mut c::timespec>,
) -> io::Result<Event> {
    let mut event = MaybeUninit::<c::port_event>::uninit();
    let timeout = timeout.map_or(null_mut(), as_mut_ptr);

    unsafe {
        ret(c::port_get(borrowed_fd(port), event.as_mut_ptr(), timeout))?;
    }

    // If we're done, initialize the event and return it.
    Ok(Event(unsafe { event.assume_init() }))
}

#[cfg(solarish)]
pub(crate) fn port_getn(
    port: BorrowedFd<'_>,
    timeout: Option<&mut c::timespec>,
    events: &mut Vec<Event>,
    mut nget: u32,
) -> io::Result<()> {
    let timeout = timeout.map_or(null_mut(), as_mut_ptr);
    unsafe {
        ret(c::port_getn(
            borrowed_fd(port),
            events.as_mut_ptr().cast(),
            events.len().try_into().unwrap(),
            &mut nget,
            timeout,
        ))?;
    }

    // Update the vector length.
    unsafe {
        events.set_len(nget.try_into().unwrap());
    }

    Ok(())
}

#[cfg(solarish)]
pub(crate) fn port_send(
    port: BorrowedFd<'_>,
    events: c::c_int,
    userdata: *mut c::c_void,
) -> io::Result<()> {
    unsafe { ret(c::port_send(borrowed_fd(port), events, userdata)) }
}
