//! Pseudoterminal operations.
//!
//! For the `openpty` and `login_tty` functions, see the
//! [rustix-openpty crate].
//!
//! [rustix-openpty crate]: https://crates.io/crates/rustix-openpty

use crate::backend::c;
use crate::fd::{AsFd, OwnedFd};
use crate::fs::OFlags;
use crate::{backend, io};
#[cfg(any(apple, linux_like, target_os = "freebsd", target_os = "fuchsia"))]
use {crate::ffi::CString, alloc::vec::Vec};

bitflags::bitflags! {
    /// `O_*` flags for use with [`openpt`] and [`ioctl_tiocgptpeer`].
    ///
    /// [`ioctl_tiocgtpeer`]: https://docs.rs/rustix/*/x86_64-unknown-linux-gnu/rustix/pty/fn.ioctl_tiocgtpeer.html
    pub struct OpenptFlags: u32 {
        /// `O_RDWR`
        const RDWR = c::O_RDWR as c::c_uint;

        /// `O_NOCTTY`
        #[cfg(not(target_os = "redox"))]
        const NOCTTY = c::O_NOCTTY as c::c_uint;

        /// `O_CLOEXEC`
        ///
        /// The standard `posix_openpt` function doesn't support `CLOEXEC`, but
        /// rustix supports it on Linux, and FreeBSD and NetBSD support it.
        #[cfg(any(linux_kernel, target_os = "freebsd", target_os = "netbsd"))]
        const CLOEXEC = c::O_CLOEXEC as c::c_uint;
    }
}

impl From<OpenptFlags> for OFlags {
    #[inline]
    fn from(flags: OpenptFlags) -> Self {
        // SAFETY: `OpenptFlags` is a subset of `OFlags`.
        #[allow(unsafe_code)]
        unsafe {
            Self::from_bits_unchecked(flags.bits() as _)
        }
    }
}

/// `posix_openpt(flags)`—Open a pseudoterminal device.
///
/// On Linux, an additional `CLOEXEC` flag value may be passed to request the
/// close-on-exec flag be set.
///
/// On Linux, if the system has no free pseudoterminals available, the
/// underlying system call fails with [`io::Errno::NOSPC`], however this rustix
/// function translates that to [`io::Errno::AGAIN`], so that the linux_raw and
/// libc backends have the same behavior.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [Apple]
///  - [FreeBSD]
///  - [DragonFly BSD]
///  - [NetBSD]
///  - [OpenBSD]
///  - [illumos]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/posix_openpt.html
/// [Linux]: https://man7.org/linux/man-pages/man3/posix_openpt.3.html
/// [Apple]: https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/posix_openpt.3.html
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?query=posix_openpt&sektion=2
/// [DragonFly BSD]: https://man.dragonflybsd.org/?command=posix_openpt&section=3
/// [NetBSD]: https://man.netbsd.org/posix_openpt.3
/// [OpenBSD]: https://man.openbsd.org/posix_openpt
/// [illumos]: https://illumos.org/man/3C/posix_openpt
#[inline]
#[doc(alias = "posix_openpt")]
pub fn openpt(flags: OpenptFlags) -> io::Result<OwnedFd> {
    // On Linux, open the device ourselves so that we can support `CLOEXEC`.
    #[cfg(linux_kernel)]
    {
        use crate::fs::{cwd, openat, Mode};
        match openat(cwd(), cstr!("/dev/ptmx"), flags.into(), Mode::empty()) {
            // Match libc `openat` behavior with `ENOSPC`.
            Err(io::Errno::NOSPC) => Err(io::Errno::AGAIN),
            otherwise => otherwise,
        }
    }

    // On all other platforms, use `openpt`.
    #[cfg(not(linux_kernel))]
    {
        backend::pty::syscalls::openpt(flags)
    }
}

/// `ptsname(fd)`—Return the name of a pseudoterminal.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [glibc]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/ptsname.html
/// [Linux]: https://man7.org/linux/man-pages/man3/ptsname.3.html
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Allocation.html#index-ptsname
#[inline]
#[doc(alias = "ptsname_r")]
#[cfg(any(apple, linux_like, target_os = "freebsd", target_os = "fuchsia"))]
pub fn ptsname<Fd: AsFd, B: Into<Vec<u8>>>(fd: Fd, reuse: B) -> io::Result<CString> {
    backend::pty::syscalls::ptsname(fd.as_fd(), reuse.into())
}

/// `unlockpt(fd)`—Unlock a pseudoterminal.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [glibc]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/unlockpt.html
/// [Linux]: https://man7.org/linux/man-pages/man3/unlockpt.3.html
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Allocation.html#index-unlockpt
#[inline]
pub fn unlockpt<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    backend::pty::syscalls::unlockpt(fd.as_fd())
}

/// `grantpt(fd)`—Grant access to the user side of a pseudoterminal.
///
/// On Linux, calling this function has no effect, as the kernel is expected to
/// grant the appropriate access. On all other platorms, this function has
/// unspecified behavior if the calling process has a `SIGCHLD` signal handler
/// installed.
///
/// # References
///  - [POSIX]
///  - [Linux]
///  - [glibc]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/grantpt.html
/// [Linux]: https://man7.org/linux/man-pages/man3/grantpt.3.html
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Allocation.html#index-grantpt
#[inline]
pub fn grantpt<Fd: AsFd>(fd: Fd) -> io::Result<()> {
    #[cfg(not(linux_kernel))]
    {
        backend::pty::syscalls::grantpt(fd.as_fd())
    }

    // On Linux, we assume the kernel has already granted the needed
    // permissions to the user side of the pseudoterminal.
    #[cfg(linux_kernel)]
    {
        let _ = fd;
        Ok(())
    }
}

/// `ioctl(fd, TIOCGPTPEER)`—Open the user side of a pseduoterminal.
///
/// This function is currently only implemented on Linux.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/ioctl_tty.2.html
#[cfg(target_os = "linux")]
#[inline]
pub fn ioctl_tiocgptpeer<Fd: AsFd>(fd: Fd, flags: OpenptFlags) -> io::Result<OwnedFd> {
    backend::pty::syscalls::ioctl_tiocgptpeer(fd.as_fd(), flags)
}
