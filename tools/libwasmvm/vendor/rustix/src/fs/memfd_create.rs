use crate::fd::OwnedFd;
use crate::{backend, io, path};

pub use backend::fs::types::MemfdFlags;

/// `memfd_create(path, flags)`
///
/// # References
///  - [Linux]
///  - [glibc]
///  - [FreeBSD]
///
/// [Linux]: https://man7.org/linux/man-pages/man2/memfd_create.2.html
/// [glibc]: https://www.gnu.org/software/libc/manual/html_node/Memory_002dmapped-I_002fO.html#index-memfd_005fcreate
/// [FreeBSD]: https://man.freebsd.org/cgi/man.cgi?memfd_create
#[inline]
pub fn memfd_create<P: path::Arg>(path: P, flags: MemfdFlags) -> io::Result<OwnedFd> {
    path.into_with_c_str(|path| backend::fs::syscalls::memfd_create(path, flags))
}
