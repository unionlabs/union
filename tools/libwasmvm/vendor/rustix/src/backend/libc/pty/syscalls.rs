//! libc syscalls supporting `rustix::pty`.

use super::super::c;
use super::super::conv::{borrowed_fd, ret};
use crate::fd::BorrowedFd;
use crate::io;
#[cfg(not(target_os = "android"))]
use {super::super::conv::ret_owned_fd, crate::fd::OwnedFd, crate::pty::OpenptFlags};
#[cfg(any(apple, linux_like, target_os = "freebsd", target_os = "fuchsia"))]
use {
    crate::ffi::{CStr, CString},
    crate::path::SMALL_PATH_BUFFER_SIZE,
    alloc::borrow::ToOwned,
    alloc::vec::Vec,
};

#[cfg(not(linux_kernel))]
#[inline]
pub(crate) fn openpt(flags: OpenptFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::posix_openpt(flags.bits() as _)) }
}

#[cfg(any(apple, linux_like, target_os = "freebsd", target_os = "fuchsia"))]
#[inline]
pub(crate) fn ptsname(fd: BorrowedFd, mut buffer: Vec<u8>) -> io::Result<CString> {
    // This code would benefit from having a better way to read into
    // uninitialized memory, but that requires `unsafe`.
    buffer.clear();
    buffer.reserve(SMALL_PATH_BUFFER_SIZE);
    buffer.resize(buffer.capacity(), 0_u8);

    loop {
        // On platforms with `ptsname_r`, use it.
        #[cfg(any(target_os = "freebsd", linux_like, target_os = "fuchsia"))]
        let r =
            unsafe { libc::ptsname_r(borrowed_fd(fd), buffer.as_mut_ptr().cast(), buffer.len()) };

        // MacOS 10.13.4 has `ptsname_r`; use it if we have it, otherwise fall
        // back to calling the underlying ioctl directly.
        #[cfg(apple)]
        let r = unsafe {
            weak! { fn ptsname_r(c::c_int, *mut c::c_char, c::size_t) -> c::c_int }

            if let Some(libc_ptsname_r) = ptsname_r.get() {
                libc_ptsname_r(borrowed_fd(fd), buffer.as_mut_ptr().cast(), buffer.len())
            } else {
                // The size declared in the `TIOCPTYGNAME` macro in sys/ttycom.h is 128.
                let mut name: [u8; 128] = [0_u8; 128];
                match libc::ioctl(borrowed_fd(fd), libc::TIOCPTYGNAME as u64, &mut name) {
                    0 => {
                        let len = CStr::from_ptr(name.as_ptr().cast()).to_bytes().len();
                        std::ptr::copy_nonoverlapping(name.as_ptr(), buffer.as_mut_ptr(), len + 1);
                        0
                    }
                    _ => libc_errno::errno().0,
                }
            }
        };

        if r == 0 {
            return Ok(unsafe { CStr::from_ptr(buffer.as_ptr().cast()).to_owned() });
        }
        if r != libc::ERANGE {
            return Err(io::Errno::from_raw_os_error(r));
        }

        buffer.reserve(1); // use `Vec` reallocation strategy to grow capacity exponentially
        buffer.resize(buffer.capacity(), 0_u8);
    }
}

#[inline]
pub(crate) fn unlockpt(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::unlockpt(borrowed_fd(fd))) }
}

#[cfg(not(linux_kernel))]
#[inline]
pub(crate) fn grantpt(fd: BorrowedFd) -> io::Result<()> {
    unsafe { ret(c::grantpt(borrowed_fd(fd))) }
}

#[cfg(target_os = "linux")]
#[inline]
pub(crate) fn ioctl_tiocgptpeer(fd: BorrowedFd, flags: OpenptFlags) -> io::Result<OwnedFd> {
    unsafe { ret_owned_fd(c::ioctl(borrowed_fd(fd), c::TIOCGPTPEER, flags.bits())) }
}
