//! `io_lifetimes` types for Windows assuming that Fd is Socket.
//!
//! We can make this assumption since `rustix` supports only `rustix::net` on
//! Windows.

pub use io_lifetimes::{BorrowedSocket as BorrowedFd, OwnedSocket as OwnedFd};
#[cfg(feature = "std")]
pub use std::os::windows::io::RawSocket as RawFd;
pub(crate) use windows_sys::Win32::Networking::WinSock::SOCKET as LibcFd;

// Re-export the `Socket` traits so that users can implement them.
pub use io_lifetimes::AsSocket;

/// A version of [`AsRawFd`] for use with Winsock2 API.
///
/// [`AsRawFd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.AsRawFd.html
pub trait AsRawFd {
    /// A version of [`as_raw_fd`] for use with Winsock2 API.
    ///
    /// [`as_raw_fd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html#tymethod.as_raw_fd
    fn as_raw_fd(&self) -> RawFd;
}
#[cfg(feature = "std")]
impl<T: std::os::windows::io::AsRawSocket> AsRawFd for T {
    #[inline]
    fn as_raw_fd(&self) -> RawFd {
        self.as_raw_socket()
    }
}

/// A version of [`IntoRawFd`] for use with Winsock2 API.
///
/// [`IntoRawFd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.IntoRawFd.html
pub trait IntoRawFd {
    /// A version of [`into_raw_fd`] for use with Winsock2 API.
    ///
    /// [`into_raw_fd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html#tymethod.into_raw_fd
    fn into_raw_fd(self) -> RawFd;
}
#[cfg(feature = "std")]
impl<T: std::os::windows::io::IntoRawSocket> IntoRawFd for T {
    #[inline]
    fn into_raw_fd(self) -> RawFd {
        self.into_raw_socket()
    }
}

/// A version of [`FromRawFd`] for use with Winsock2 API.
///
/// [`FromRawFd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html
pub trait FromRawFd {
    /// A version of [`from_raw_fd`] for use with Winsock2 API.
    ///
    /// # Safety
    ///
    /// See the [safety requirements] for [`from_raw_fd`].
    ///
    /// [`from_raw_fd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html#tymethod.from_raw_fd
    /// [safety requirements]: https://doc.rust-lang.org/stable/std/os/fd/trait.FromRawFd.html#safety
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self;
}
#[cfg(feature = "std")]
impl<T: std::os::windows::io::FromRawSocket> FromRawFd for T {
    #[inline]
    unsafe fn from_raw_fd(raw_fd: RawFd) -> Self {
        Self::from_raw_socket(raw_fd)
    }
}

/// A version of [`AsFd`] for use with Winsock2 API.
///
/// [`AsFd`]: https://doc.rust-lang.org/stable/std/os/fd/trait.AsFd.html
pub trait AsFd {
    /// An `as_fd` function for Winsock2, where a `Fd` is a `Socket`.
    fn as_fd(&self) -> BorrowedFd;
}
impl<T: AsSocket> AsFd for T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd {
        self.as_socket()
    }
}
