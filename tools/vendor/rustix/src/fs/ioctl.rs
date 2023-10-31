//! Filesystem-oriented `ioctl` functions.

#[cfg(linux_kernel)]
use {
    crate::fd::AsFd,
    crate::{backend, io},
};

/// `ioctl(fd, BLKSSZGET)`—Returns the logical block size of a block device.
///
/// This is mentioned in the [Linux `openat` manual page].
///
/// [Linux `openat` manual page]: https://man7.org/linux/man-pages/man2/openat.2.html
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "BLKSSZGET")]
pub fn ioctl_blksszget<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::fs::syscalls::ioctl_blksszget(fd.as_fd())
}

/// `ioctl(fd, BLKPBSZGET)`—Returns the physical block size of a block device.
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "BLKPBSZGET")]
pub fn ioctl_blkpbszget<Fd: AsFd>(fd: Fd) -> io::Result<u32> {
    backend::fs::syscalls::ioctl_blkpbszget(fd.as_fd())
}

/// `ioctl(fd, FICLONE, src_fd)`—Share data between open files.
///
/// This ioctl is not available on Sparc platforms
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/ioctl_ficlone.2.html
#[cfg(all(linux_kernel, not(any(target_arch = "sparc", target_arch = "sparc64"))))]
#[inline]
#[doc(alias = "FICLONE")]
pub fn ioctl_ficlone<Fd: AsFd, SrcFd: AsFd>(fd: Fd, src_fd: SrcFd) -> io::Result<()> {
    backend::fs::syscalls::ioctl_ficlone(fd.as_fd(), src_fd.as_fd())
}

/// `ioctl(fd, EXT4_IOC_RESIZE_FS, blocks)`—Resize ext4 filesystem on fd.
#[cfg(linux_kernel)]
#[inline]
#[doc(alias = "EXT4_IOC_RESIZE_FS")]
pub fn ext4_ioc_resize_fs<Fd: AsFd>(fd: Fd, blocks: u64) -> io::Result<()> {
    backend::fs::syscalls::ext4_ioc_resize_fs(fd.as_fd(), blocks)
}
