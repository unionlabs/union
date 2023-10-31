//! Linux `statx`.

use crate::fd::AsFd;
use crate::fs::AtFlags;
use crate::{backend, io, path};

pub use backend::fs::types::{Statx, StatxFlags, StatxTimestamp};

#[cfg(feature = "linux_4_11")]
use backend::fs::syscalls::statx as _statx;
#[cfg(not(feature = "linux_4_11"))]
use compat::statx as _statx;

/// `statx(dirfd, path, flags, mask, statxbuf)`
///
/// This function returns [`io::Errno::NOSYS`] if `statx` is not available on
/// the platform, such as Linux before 4.11. This also includes older Docker
/// versions where the actual syscall fails with different error codes; rustix
/// handles this and translates them into `NOSYS`.
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/statx.2.html
#[inline]
pub fn statx<P: path::Arg, Fd: AsFd>(
    dirfd: Fd,
    path: P,
    flags: AtFlags,
    mask: StatxFlags,
) -> io::Result<Statx> {
    path.into_with_c_str(|path| _statx(dirfd.as_fd(), path, flags, mask))
}

#[cfg(not(feature = "linux_4_11"))]
mod compat {
    use crate::fd::BorrowedFd;
    use crate::ffi::CStr;
    use crate::fs::AtFlags;
    use crate::{backend, io};
    use core::sync::atomic::{AtomicU8, Ordering};

    use backend::fs::types::{Statx, StatxFlags};

    // Linux kernel prior to 4.11 old versions of Docker don't support `statx`.
    // We store the availability in a global to avoid unnecessary syscalls.
    //
    // 0: Unknown
    // 1: Not available
    // 2: Available
    static STATX_STATE: AtomicU8 = AtomicU8::new(0);

    #[inline]
    pub fn statx(
        dirfd: BorrowedFd<'_>,
        path: &CStr,
        flags: AtFlags,
        mask: StatxFlags,
    ) -> io::Result<Statx> {
        match STATX_STATE.load(Ordering::Relaxed) {
            0 => statx_init(dirfd, path, flags, mask),
            1 => Err(io::Errno::NOSYS),
            _ => backend::fs::syscalls::statx(dirfd, path, flags, mask),
        }
    }

    /// The first `statx` call. We don't know if `statx` is available yet.
    fn statx_init(
        dirfd: BorrowedFd<'_>,
        path: &CStr,
        flags: AtFlags,
        mask: StatxFlags,
    ) -> io::Result<Statx> {
        match backend::fs::syscalls::statx(dirfd, path, flags, mask) {
            Err(io::Errno::NOSYS) => statx_error_nosys(),
            Err(io::Errno::PERM) => statx_error_perm(),
            result => {
                STATX_STATE.store(2, Ordering::Relaxed);
                result
            }
        }
    }

    /// The first `statx` call failed with `NOSYS` (or something we're treating
    /// like `NOSYS`).
    #[cold]
    fn statx_error_nosys() -> io::Result<Statx> {
        STATX_STATE.store(1, Ordering::Relaxed);
        Err(io::Errno::NOSYS)
    }

    /// The first `statx` call failed with `PERM`.
    #[cold]
    fn statx_error_perm() -> io::Result<Statx> {
        // Some old versions of Docker have `statx` fail with `PERM` when it
        // isn't recognized. Check whether `statx` really is available, and if
        // so, fail with `PERM`, and if not, treat it like `NOSYS`.
        if backend::fs::syscalls::is_statx_available() {
            STATX_STATE.store(2, Ordering::Relaxed);
            Err(io::Errno::PERM)
        } else {
            statx_error_nosys()
        }
    }
}
