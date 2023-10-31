use crate::{backend, io};

/// `GRND_*` constants for use with `getrandom`.
pub use backend::rand::types::GetRandomFlags;

/// `getrandom(buf, flags)`—Reads a sequence of random bytes.
///
/// This is a very low-level API which may be difficult to use correctly. Most
/// users should prefer to use [`getrandom`] or [`rand`] APIs instead.
///
/// [`getrandom`]: https://crates.io/crates/getrandom
/// [`rand`]: https://crates.io/crates/rand
///
/// # References
///  - [Linux]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/getrandom.2.html
#[inline]
pub fn getrandom(buf: &mut [u8], flags: GetRandomFlags) -> io::Result<usize> {
    backend::rand::syscalls::getrandom(buf, flags)
}
