//! linux_raw syscalls supporting `rustix::io`.
//!
//! # Safety
//!
//! See the `rustix::backend` module documentation for details.
#![allow(unsafe_code)]
#![allow(clippy::undocumented_unsafe_blocks)]

use super::super::c;
#[cfg(target_pointer_width = "64")]
use super::super::conv::loff_t_from_u64;
use super::super::conv::{
    by_ref, c_int, c_uint, opt_mut, pass_usize, raw_fd, ret, ret_c_uint, ret_discarded_fd,
    ret_owned_fd, ret_usize, slice, slice_mut, zero,
};
#[cfg(target_pointer_width = "32")]
use super::super::conv::{hi, lo};
use crate::fd::{AsFd, BorrowedFd, OwnedFd, RawFd};
#[cfg(linux_kernel)]
use crate::io::SpliceFlags;
use crate::io::{
    self, epoll, DupFlags, EventfdFlags, FdFlags, IoSlice, IoSliceMut, IoSliceRaw, PipeFlags,
    PollFd, ReadWriteFlags,
};
#[cfg(all(feature = "fs", feature = "net"))]
use crate::net::{RecvFlags, SendFlags};
use core::cmp;
use core::mem::MaybeUninit;
#[cfg(target_os = "espidf")]
use linux_raw_sys::general::F_DUPFD;
use linux_raw_sys::general::{
    epoll_event, EPOLL_CTL_ADD, EPOLL_CTL_DEL, EPOLL_CTL_MOD, F_DUPFD_CLOEXEC, F_GETFD, F_SETFD,
    UIO_MAXIOV,
};
use linux_raw_sys::ioctl::{
    BLKPBSZGET, BLKSSZGET, EXT4_IOC_RESIZE_FS, FICLONE, FIONBIO, FIONREAD, TIOCEXCL, TIOCNXCL,
};
#[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
use {
    super::super::conv::{opt_ref, size_of},
    linux_raw_sys::general::{__kernel_timespec, kernel_sigset_t},
};

#[inline]
pub(crate) fn read(fd: BorrowedFd<'_>, buf: &mut [u8]) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);

    unsafe { ret_usize(syscall!(__NR_read, fd, buf_addr_mut, buf_len)) }
}

#[inline]
pub(crate) fn pread(fd: BorrowedFd<'_>, buf: &mut [u8], pos: u64) -> io::Result<usize> {
    let (buf_addr_mut, buf_len) = slice_mut(buf);

    // <https://github.com/torvalds/linux/blob/fcadab740480e0e0e9fa9bd272acd409884d431a/arch/arm64/kernel/sys32.c#L75>
    #[cfg(all(
        target_pointer_width = "32",
        any(target_arch = "arm", target_arch = "mips", target_arch = "power"),
    ))]
    unsafe {
        ret_usize(syscall!(
            __NR_pread64,
            fd,
            buf_addr_mut,
            buf_len,
            zero(),
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_arch = "arm", target_arch = "mips", target_arch = "power")),
    ))]
    unsafe {
        ret_usize(syscall!(
            __NR_pread64,
            fd,
            buf_addr_mut,
            buf_len,
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall!(
            __NR_pread64,
            fd,
            buf_addr_mut,
            buf_len,
            loff_t_from_u64(pos)
        ))
    }
}

#[inline]
pub(crate) fn readv(fd: BorrowedFd<'_>, bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    unsafe { ret_usize(syscall!(__NR_readv, fd, bufs_addr, bufs_len)) }
}

#[inline]
pub(crate) fn preadv(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut<'_>],
    pos: u64,
) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall!(
            __NR_preadv,
            fd,
            bufs_addr,
            bufs_len,
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall!(
            __NR_preadv,
            fd,
            bufs_addr,
            bufs_len,
            loff_t_from_u64(pos)
        ))
    }
}

#[inline]
pub(crate) fn preadv2(
    fd: BorrowedFd<'_>,
    bufs: &mut [IoSliceMut<'_>],
    pos: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall!(
            __NR_preadv2,
            fd,
            bufs_addr,
            bufs_len,
            hi(pos),
            lo(pos),
            flags
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall!(
            __NR_preadv2,
            fd,
            bufs_addr,
            bufs_len,
            loff_t_from_u64(pos),
            flags
        ))
    }
}

#[inline]
pub(crate) fn write(fd: BorrowedFd<'_>, buf: &[u8]) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    unsafe { ret_usize(syscall_readonly!(__NR_write, fd, buf_addr, buf_len)) }
}

#[inline]
pub(crate) fn pwrite(fd: BorrowedFd<'_>, buf: &[u8], pos: u64) -> io::Result<usize> {
    let (buf_addr, buf_len) = slice(buf);

    // <https://github.com/torvalds/linux/blob/fcadab740480e0e0e9fa9bd272acd409884d431a/arch/arm64/kernel/sys32.c#L81-L83>
    #[cfg(all(
        target_pointer_width = "32",
        any(target_arch = "arm", target_arch = "mips", target_arch = "power"),
    ))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwrite64,
            fd,
            buf_addr,
            buf_len,
            zero(),
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(all(
        target_pointer_width = "32",
        not(any(target_arch = "arm", target_arch = "mips", target_arch = "power")),
    ))]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwrite64,
            fd,
            buf_addr,
            buf_len,
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwrite64,
            fd,
            buf_addr,
            buf_len,
            loff_t_from_u64(pos)
        ))
    }
}

#[inline]
pub(crate) fn writev(fd: BorrowedFd<'_>, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    unsafe { ret_usize(syscall_readonly!(__NR_writev, fd, bufs_addr, bufs_len)) }
}

#[inline]
pub(crate) fn pwritev(fd: BorrowedFd<'_>, bufs: &[IoSlice<'_>], pos: u64) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwritev,
            fd,
            bufs_addr,
            bufs_len,
            hi(pos),
            lo(pos)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwritev,
            fd,
            bufs_addr,
            bufs_len,
            loff_t_from_u64(pos)
        ))
    }
}

#[inline]
pub(crate) fn pwritev2(
    fd: BorrowedFd<'_>,
    bufs: &[IoSlice<'_>],
    pos: u64,
    flags: ReadWriteFlags,
) -> io::Result<usize> {
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);

    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwritev2,
            fd,
            bufs_addr,
            bufs_len,
            hi(pos),
            lo(pos),
            flags
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_usize(syscall_readonly!(
            __NR_pwritev2,
            fd,
            bufs_addr,
            bufs_len,
            loff_t_from_u64(pos),
            flags
        ))
    }
}

/// The maximum number of buffers that can be passed into a vectored I/O system
/// call on the current platform.
const fn max_iov() -> usize {
    UIO_MAXIOV as usize
}

#[inline]
pub(crate) unsafe fn close(fd: RawFd) {
    // See the documentation for [`io::close`] for why errors are ignored.
    syscall_readonly!(__NR_close, raw_fd(fd)).decode_void();
}

#[inline]
pub(crate) fn eventfd(initval: u32, flags: EventfdFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_eventfd2, c_uint(initval), flags)) }
}

#[inline]
pub(crate) fn ioctl_fionread(fd: BorrowedFd<'_>) -> io::Result<u64> {
    unsafe {
        let mut result = MaybeUninit::<c::c_int>::uninit();
        ret(syscall!(__NR_ioctl, fd, c_uint(FIONREAD), &mut result))?;
        Ok(result.assume_init() as u64)
    }
}

#[inline]
pub(crate) fn ioctl_fionbio(fd: BorrowedFd<'_>, value: bool) -> io::Result<()> {
    unsafe {
        let data = c::c_int::from(value);
        ret(syscall_readonly!(
            __NR_ioctl,
            fd,
            c_uint(FIONBIO),
            by_ref(&data)
        ))
    }
}

#[inline]
pub(crate) fn ioctl_tiocexcl(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_ioctl, fd, c_uint(TIOCEXCL))) }
}

#[inline]
pub(crate) fn ioctl_tiocnxcl(fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_ioctl, fd, c_uint(TIOCNXCL))) }
}

#[inline]
pub(crate) fn ioctl_blksszget(fd: BorrowedFd) -> io::Result<u32> {
    let mut result = MaybeUninit::<c::c_uint>::uninit();
    unsafe {
        ret(syscall!(__NR_ioctl, fd, c_uint(BLKSSZGET), &mut result))?;
        Ok(result.assume_init() as u32)
    }
}

#[inline]
pub(crate) fn ioctl_blkpbszget(fd: BorrowedFd) -> io::Result<u32> {
    let mut result = MaybeUninit::<c::c_uint>::uninit();
    unsafe {
        ret(syscall!(__NR_ioctl, fd, c_uint(BLKPBSZGET), &mut result))?;
        Ok(result.assume_init() as u32)
    }
}

#[inline]
pub(crate) fn ioctl_ficlone(fd: BorrowedFd<'_>, src_fd: BorrowedFd<'_>) -> io::Result<()> {
    unsafe { ret(syscall_readonly!(__NR_ioctl, fd, c_uint(FICLONE), src_fd)) }
}

#[inline]
pub(crate) fn ext4_ioc_resize_fs(fd: BorrowedFd<'_>, blocks: u64) -> io::Result<()> {
    unsafe {
        ret(syscall_readonly!(
            __NR_ioctl,
            fd,
            c_uint(EXT4_IOC_RESIZE_FS),
            by_ref(&blocks)
        ))
    }
}

#[cfg(all(feature = "fs", feature = "net"))]
pub(crate) fn is_read_write(fd: BorrowedFd<'_>) -> io::Result<(bool, bool)> {
    let (mut read, mut write) = crate::fs::fd::_is_file_read_write(fd)?;
    let mut not_socket = false;
    if read {
        // Do a `recv` with `PEEK` and `DONTWAIT` for 1 byte. A 0 indicates
        // the read side is shut down; an `EWOULDBLOCK` indicates the read
        // side is still open.
        //
        // TODO: This code would benefit from having a better way to read into
        // uninitialized memory.
        let mut buf = [0];
        match super::super::net::syscalls::recv(fd, &mut buf, RecvFlags::PEEK | RecvFlags::DONTWAIT)
        {
            Ok(0) => read = false,
            Err(err) => {
                #[allow(unreachable_patterns)] // `EAGAIN` may equal `EWOULDBLOCK`
                match err {
                    io::Errno::AGAIN | io::Errno::WOULDBLOCK => (),
                    io::Errno::NOTSOCK => not_socket = true,
                    _ => return Err(err),
                }
            }
            Ok(_) => (),
        }
    }
    if write && !not_socket {
        // Do a `send` with `DONTWAIT` for 0 bytes. An `EPIPE` indicates
        // the write side is shut down.
        #[allow(unreachable_patterns)] // `EAGAIN` equals `EWOULDBLOCK`
        match super::super::net::syscalls::send(fd, &[], SendFlags::DONTWAIT) {
            // TODO or-patterns when we don't need 1.51
            Err(io::Errno::AGAIN) => (),
            Err(io::Errno::WOULDBLOCK) => (),
            Err(io::Errno::NOTSOCK) => (),
            Err(io::Errno::PIPE) => write = false,
            Err(err) => return Err(err),
            Ok(_) => (),
        }
    }
    Ok((read, write))
}

#[inline]
pub(crate) fn dup(fd: BorrowedFd<'_>) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_dup, fd)) }
}

#[inline]
pub(crate) fn dup2(fd: BorrowedFd<'_>, new: &mut OwnedFd) -> io::Result<()> {
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    {
        // We don't need to worry about the difference between `dup2` and
        // `dup3` when the file descriptors are equal because we have an
        // `&mut OwnedFd` which means `fd` doesn't alias it.
        dup3(fd, new, DupFlags::empty())
    }

    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        ret_discarded_fd(syscall_readonly!(__NR_dup2, fd, new.as_fd()))
    }
}

#[inline]
pub(crate) fn dup3(fd: BorrowedFd<'_>, new: &mut OwnedFd, flags: DupFlags) -> io::Result<()> {
    unsafe { ret_discarded_fd(syscall_readonly!(__NR_dup3, fd, new.as_fd(), flags)) }
}

#[inline]
pub(crate) fn fcntl_getfd(fd: BorrowedFd<'_>) -> io::Result<FdFlags> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_c_uint(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_GETFD)))
            .map(FdFlags::from_bits_truncate)
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_c_uint(syscall_readonly!(__NR_fcntl, fd, c_uint(F_GETFD)))
            .map(FdFlags::from_bits_truncate)
    }
}

#[inline]
pub(crate) fn fcntl_setfd(fd: BorrowedFd<'_>, flags: FdFlags) -> io::Result<()> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret(syscall_readonly!(__NR_fcntl64, fd, c_uint(F_SETFD), flags))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret(syscall_readonly!(__NR_fcntl, fd, c_uint(F_SETFD), flags))
    }
}

#[cfg(target_os = "espidf")]
#[inline]
pub(crate) fn fcntl_dupfd(fd: BorrowedFd<'_>, min: RawFd) -> io::Result<OwnedFd> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_fcntl64,
            fd,
            c_uint(F_DUPFD),
            raw_fd(min)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_fcntl,
            fd,
            c_uint(F_DUPFD),
            raw_fd(min)
        ))
    }
}

#[inline]
pub(crate) fn fcntl_dupfd_cloexec(fd: BorrowedFd<'_>, min: RawFd) -> io::Result<OwnedFd> {
    #[cfg(target_pointer_width = "32")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_fcntl64,
            fd,
            c_uint(F_DUPFD_CLOEXEC),
            raw_fd(min)
        ))
    }
    #[cfg(target_pointer_width = "64")]
    unsafe {
        ret_owned_fd(syscall_readonly!(
            __NR_fcntl,
            fd,
            c_uint(F_DUPFD_CLOEXEC),
            raw_fd(min)
        ))
    }
}

#[inline]
pub(crate) fn pipe_with(flags: PipeFlags) -> io::Result<(OwnedFd, OwnedFd)> {
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(syscall!(__NR_pipe2, &mut result, flags))?;
        let [p0, p1] = result.assume_init();
        Ok((p0, p1))
    }
}

#[inline]
pub(crate) fn pipe() -> io::Result<(OwnedFd, OwnedFd)> {
    // aarch64 and risc64 omit `__NR_pipe`. On mips, `__NR_pipe` uses a special
    // calling convention, but using it is not worth complicating our syscall
    // wrapping infrastructure at this time.
    #[cfg(any(
        target_arch = "aarch64",
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "riscv64",
    ))]
    {
        pipe_with(PipeFlags::empty())
    }
    #[cfg(not(any(
        target_arch = "aarch64",
        target_arch = "mips",
        target_arch = "mips64",
        target_arch = "riscv64",
    )))]
    unsafe {
        let mut result = MaybeUninit::<[OwnedFd; 2]>::uninit();
        ret(syscall!(__NR_pipe, &mut result))?;
        let [p0, p1] = result.assume_init();
        Ok((p0, p1))
    }
}

#[inline]
pub(crate) fn poll(fds: &mut [PollFd<'_>], timeout: c::c_int) -> io::Result<usize> {
    let (fds_addr_mut, fds_len) = slice_mut(fds);

    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    unsafe {
        let timeout = if timeout >= 0 {
            Some(__kernel_timespec {
                tv_sec: (timeout as i64) / 1000,
                tv_nsec: (timeout as i64) % 1000 * 1_000_000,
            })
        } else {
            None
        };
        ret_usize(syscall!(
            __NR_ppoll,
            fds_addr_mut,
            fds_len,
            opt_ref(timeout.as_ref()),
            zero(),
            size_of::<kernel_sigset_t, _>()
        ))
    }
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        ret_usize(syscall!(__NR_poll, fds_addr_mut, fds_len, c_int(timeout)))
    }
}

#[inline]
pub(crate) fn epoll_create(flags: epoll::CreateFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(syscall_readonly!(__NR_epoll_create1, flags)) }
}

#[inline]
pub(crate) unsafe fn epoll_add(
    epfd: BorrowedFd<'_>,
    fd: c::c_int,
    event: &epoll_event,
) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_ADD),
        raw_fd(fd),
        by_ref(event)
    ))
}

#[inline]
pub(crate) unsafe fn epoll_mod(
    epfd: BorrowedFd<'_>,
    fd: c::c_int,
    event: &epoll_event,
) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_MOD),
        raw_fd(fd),
        by_ref(event)
    ))
}

#[inline]
pub(crate) unsafe fn epoll_del(epfd: BorrowedFd<'_>, fd: c::c_int) -> io::Result<()> {
    ret(syscall_readonly!(
        __NR_epoll_ctl,
        epfd,
        c_uint(EPOLL_CTL_DEL),
        raw_fd(fd),
        zero()
    ))
}

#[inline]
pub(crate) fn epoll_wait(
    epfd: BorrowedFd<'_>,
    events: *mut epoll_event,
    num_events: usize,
    timeout: c::c_int,
) -> io::Result<usize> {
    #[cfg(not(any(target_arch = "aarch64", target_arch = "riscv64")))]
    unsafe {
        ret_usize(syscall!(
            __NR_epoll_wait,
            epfd,
            events,
            pass_usize(num_events),
            c_int(timeout)
        ))
    }
    #[cfg(any(target_arch = "aarch64", target_arch = "riscv64"))]
    unsafe {
        ret_usize(syscall!(
            __NR_epoll_pwait,
            epfd,
            events,
            pass_usize(num_events),
            c_int(timeout),
            zero()
        ))
    }
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
    unsafe {
        ret_usize(syscall!(
            __NR_splice,
            fd_in,
            opt_mut(off_in),
            fd_out,
            opt_mut(off_out),
            pass_usize(len),
            c_uint(flags.bits())
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
    let (bufs_addr, bufs_len) = slice(&bufs[..cmp::min(bufs.len(), max_iov())]);
    ret_usize(syscall!(
        __NR_vmsplice,
        fd,
        bufs_addr,
        bufs_len,
        c_uint(flags.bits())
    ))
}
