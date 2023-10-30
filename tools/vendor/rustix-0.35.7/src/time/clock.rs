use crate::{backend, io};

pub use backend::time::types::{Nsecs, Secs, Timespec};

/// `clockid_t`
#[cfg(any(not(target_os = "wasi")))]
pub use backend::time::types::{ClockId, DynamicClockId};

/// `clock_getres(id)`—Returns the resolution of a clock.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/clock_getres.html
/// [Linux]: https://man7.org/linux/man-pages/man2/clock_getres.2.html
#[cfg(any(not(any(target_os = "redox", target_os = "wasi"))))]
#[inline]
#[must_use]
pub fn clock_getres(id: ClockId) -> Timespec {
    backend::time::syscalls::clock_getres(id)
}

/// `clock_gettime(id)`—Returns the current value of a clock.
///
/// This function uses `ClockId` which only contains clocks which are known to
/// always be supported at runtime, allowing this function to be infallible.
/// For a greater set of clocks and dynamic clock support, see
/// [`clock_gettime_dynamic`].
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/clock_gettime.html
/// [Linux]: https://man7.org/linux/man-pages/man2/clock_gettime.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
#[must_use]
pub fn clock_gettime(id: ClockId) -> Timespec {
    backend::time::syscalls::clock_gettime(id)
}

/// Like [`clock_gettime`] but with support for dynamic clocks.
///
/// # References
///  - [POSIX]
///  - [Linux]
///
/// [POSIX]: https://pubs.opengroup.org/onlinepubs/9699919799/functions/clock_gettime.html
/// [Linux]: https://man7.org/linux/man-pages/man2/clock_gettime.2.html
#[cfg(not(target_os = "wasi"))]
#[inline]
pub fn clock_gettime_dynamic(id: DynamicClockId<'_>) -> io::Result<Timespec> {
    backend::time::syscalls::clock_gettime_dynamic(id)
}
