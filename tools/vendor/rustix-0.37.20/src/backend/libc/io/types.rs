use super::super::c;
use bitflags::bitflags;
#[cfg(linux_kernel)]
use core::marker::PhantomData;

bitflags! {
    /// `FD_*` constants for use with [`fcntl_getfd`] and [`fcntl_setfd`].
    ///
    /// [`fcntl_getfd`]: crate::io::fcntl_getfd
    /// [`fcntl_setfd`]: crate::io::fcntl_setfd
    pub struct FdFlags: c::c_int {
        /// `FD_CLOEXEC`
        const CLOEXEC = c::FD_CLOEXEC;
    }
}

#[cfg(linux_kernel)]
bitflags! {
    /// `RWF_*` constants for use with [`preadv2`] and [`pwritev2`].
    ///
    /// [`preadv2`]: crate::io::preadv2
    /// [`pwritev2`]: crate::io::pwritev
    pub struct ReadWriteFlags: c::c_int {
        /// `RWF_DSYNC` (since Linux 4.7)
        const DSYNC = linux_raw_sys::general::RWF_DSYNC as c::c_int;
        /// `RWF_HIPRI` (since Linux 4.6)
        const HIPRI = linux_raw_sys::general::RWF_HIPRI as c::c_int;
        /// `RWF_SYNC` (since Linux 4.7)
        const SYNC = linux_raw_sys::general::RWF_SYNC as c::c_int;
        /// `RWF_NOWAIT` (since Linux 4.14)
        const NOWAIT = linux_raw_sys::general::RWF_NOWAIT as c::c_int;
        /// `RWF_APPEND` (since Linux 4.16)
        const APPEND = linux_raw_sys::general::RWF_APPEND as c::c_int;
    }
}

#[cfg(linux_kernel)]
bitflags! {
    /// `SPLICE_F_*` constants for use with [`splice`] and [`vmsplice`].
    pub struct SpliceFlags: c::c_uint {
        /// `SPLICE_F_MOVE`
        const MOVE = c::SPLICE_F_MOVE;
        /// `SPLICE_F_NONBLOCK`
        const NONBLOCK = c::SPLICE_F_NONBLOCK;
        /// `SPLICE_F_MORE`
        const MORE = c::SPLICE_F_MORE;
        /// `SPLICE_F_GIFT`
        const GIFT = c::SPLICE_F_GIFT;
    }
}

#[cfg(not(target_os = "wasi"))]
bitflags! {
    /// `O_*` constants for use with [`dup2`].
    ///
    /// [`dup2`]: crate::io::dup2
    pub struct DupFlags: c::c_int {
        /// `O_CLOEXEC`
        #[cfg(not(any(
            apple,
            target_os = "aix",
            target_os = "android",
            target_os = "redox",
        )))] // Android 5.0 has dup3, but libc doesn't have bindings
        const CLOEXEC = c::O_CLOEXEC;
    }
}

#[cfg(not(any(apple, target_os = "wasi")))]
bitflags! {
    /// `O_*` constants for use with [`pipe_with`].
    ///
    /// [`pipe_with`]: crate::io::pipe_with
    pub struct PipeFlags: c::c_int {
        /// `O_CLOEXEC`
        const CLOEXEC = c::O_CLOEXEC;
        /// `O_DIRECT`
        #[cfg(not(any(
            solarish,
            target_os = "haiku",
            target_os = "openbsd",
            target_os = "redox",
        )))]
        const DIRECT = c::O_DIRECT;
        /// `O_NONBLOCK`
        const NONBLOCK = c::O_NONBLOCK;
    }
}

#[cfg(any(linux_kernel, target_os = "freebsd", target_os = "illumos"))]
bitflags! {
    /// `EFD_*` flags for use with [`eventfd`].
    ///
    /// [`eventfd`]: crate::io::eventfd
    pub struct EventfdFlags: c::c_int {
        /// `EFD_CLOEXEC`
        const CLOEXEC = c::EFD_CLOEXEC;
        /// `EFD_NONBLOCK`
        const NONBLOCK = c::EFD_NONBLOCK;
        /// `EFD_SEMAPHORE`
        const SEMAPHORE = c::EFD_SEMAPHORE;
    }
}

/// `PIPE_BUF`—The maximum size of a write to a pipe guaranteed to be atomic.
#[cfg(not(any(solarish, target_os = "haiku", target_os = "redox", target_os = "wasi")))]
pub const PIPE_BUF: usize = c::PIPE_BUF;

#[cfg(not(any(windows, target_os = "redox")))]
pub(crate) const AT_FDCWD: c::c_int = c::AT_FDCWD;
#[cfg(not(windows))]
pub(crate) const STDIN_FILENO: c::c_int = c::STDIN_FILENO;
#[cfg(not(windows))]
pub(crate) const STDOUT_FILENO: c::c_int = c::STDOUT_FILENO;
#[cfg(not(windows))]
pub(crate) const STDERR_FILENO: c::c_int = c::STDERR_FILENO;

/// A buffer type used with `vmsplice`.
/// It is guaranteed to be ABI compatible with the iovec type on Unix platforms
/// and `WSABUF` on Windows. Unlike `IoSlice` and `IoSliceMut` it is
/// semantically like a raw pointer, and therefore can be shared or mutated as
/// needed.
#[cfg(linux_kernel)]
#[repr(transparent)]
pub struct IoSliceRaw<'a> {
    _buf: c::iovec,
    _lifetime: PhantomData<&'a ()>,
}

#[cfg(linux_kernel)]
impl<'a> IoSliceRaw<'a> {
    /// Creates a new `IoSlice` wrapping a byte slice.
    pub fn from_slice(buf: &'a [u8]) -> Self {
        IoSliceRaw {
            _buf: c::iovec {
                iov_base: buf.as_ptr() as *mut u8 as *mut c::c_void,
                iov_len: buf.len() as _,
            },
            _lifetime: PhantomData,
        }
    }

    /// Creates a new `IoSlice` wrapping a mutable byte slice.
    pub fn from_slice_mut(buf: &'a mut [u8]) -> Self {
        IoSliceRaw {
            _buf: c::iovec {
                iov_base: buf.as_mut_ptr() as *mut c::c_void,
                iov_len: buf.len() as _,
            },
            _lifetime: PhantomData,
        }
    }
}
