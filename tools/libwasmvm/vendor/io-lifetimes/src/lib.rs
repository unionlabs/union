//! Experimental new types and traits to replace the `Raw` family of types and
//! traits.
//!
//! This API has much conceptual similarity with the `Raw` API, but introduces
//! explicit concepts of ownership and borrowing:
//!
//! | `Raw` API  | This experimental API    |
//! | ---------- | ------------------------ |
//! | `Raw*`     | `Borrowed*` and `Owned*` |
//! | `AsRaw*`   | `As*`                    |
//! | `IntoRaw*` | `Into*`                  |
//! | `FromRaw*` | `From*`                  |
//!
//! This gives it several advantages:
//!
//!  - Less `unsafe` in user code!
//!
//!  - Easier to understand ownership.
//!
//!  - It avoids the inconsistency where `AsRawFd` and `IntoRawFd` return
//!    `RawFd` values that users ought to be able to trust, but aren't unsafe,
//!    so it's possible to fail to uphold this trust in purely safe Rust.
//!
//!  - It enables a number of safe and portable convenience features, such as
//!    [safe typed views] and [from+into conversions].
//!
//! [safe typed views]: AsFilelike::as_filelike_view
//! [from+into conversions]: FromFilelike::from_into_filelike

#![deny(missing_docs)]
// Work around <https://github.com/rust-lang/rust/issues/103306>.
#![cfg_attr(all(wasi_ext, target_os = "wasi"), feature(wasi_ext))]
// Currently supported platforms.
#![cfg(any(unix, windows, target_os = "wasi", target_os = "hermit"))]

mod portability;
mod traits;
#[cfg(not(io_safety_is_in_std))]
mod types;

#[cfg(not(io_safety_is_in_std))]
mod impls_std;

#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
pub use traits::AsFd;
#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
pub use traits::{AsHandle, AsSocket};
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[allow(deprecated)]
pub use traits::{FromFd, IntoFd};
#[cfg(windows)]
#[allow(deprecated)]
pub use traits::{FromHandle, FromSocket, IntoHandle, IntoSocket};

#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
pub use types::{BorrowedFd, OwnedFd};
#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
pub use types::{
    BorrowedHandle, BorrowedSocket, HandleOrInvalid, InvalidHandleError, NullHandleError,
    OwnedHandle, OwnedSocket,
};

#[cfg(io_safety_is_in_std)]
#[cfg(target_os = "hermit")]
pub use std::os::hermit::io::{AsFd, BorrowedFd, OwnedFd};
#[cfg(io_safety_is_in_std)]
#[cfg(unix)]
pub use std::os::unix::io::{AsFd, BorrowedFd, OwnedFd};
#[cfg(io_safety_is_in_std)]
#[cfg(target_os = "wasi")]
pub use std::os::wasi::io::{AsFd, BorrowedFd, OwnedFd};
#[cfg(io_safety_is_in_std)]
#[cfg(windows)]
pub use std::os::windows::io::{
    AsHandle, AsSocket, BorrowedHandle, BorrowedSocket, HandleOrInvalid, InvalidHandleError,
    NullHandleError, OwnedHandle, OwnedSocket,
};

// io-lifetimes defined `FromFd`/`IntoFd` traits instead of just using
// `From`/`Into` because that allowed it to implement them for foreign types,
// including std types like File and TcpStream, and popular third-party types.
//
// std just uses `From`/`Into`, because it defines those traits itself so it
// can implement them for std types itself, and std won't be implementing them
// for third-party types. However, this means that until `OwnedFd` et al are
// stabilized, there will be no impls for third-party traits.
//
// So we define `FromFd`/`IntoFd` traits, and implement them in terms of
// `From`/`Into`,
#[cfg(io_safety_is_in_std)]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[allow(deprecated)]
impl<T: From<OwnedFd>> FromFd for T {
    #[inline]
    fn from_fd(owned_fd: OwnedFd) -> Self {
        owned_fd.into()
    }
}
#[cfg(io_safety_is_in_std)]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[allow(deprecated)]
impl<T> IntoFd for T
where
    OwnedFd: From<T>,
{
    #[inline]
    fn into_fd(self) -> OwnedFd {
        self.into()
    }
}

#[cfg(io_safety_is_in_std)]
#[cfg(windows)]
#[allow(deprecated)]
impl<T: From<OwnedHandle>> FromHandle for T {
    #[inline]
    fn from_handle(owned_handle: OwnedHandle) -> Self {
        owned_handle.into()
    }
}
#[cfg(io_safety_is_in_std)]
#[cfg(windows)]
#[allow(deprecated)]
impl<T> IntoHandle for T
where
    OwnedHandle: From<T>,
{
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        self.into()
    }
}

#[cfg(io_safety_is_in_std)]
#[cfg(windows)]
#[allow(deprecated)]
impl<T: From<OwnedSocket>> FromSocket for T {
    #[inline]
    fn from_socket(owned_socket: OwnedSocket) -> Self {
        owned_socket.into()
    }
}
#[cfg(io_safety_is_in_std)]
#[cfg(windows)]
#[allow(deprecated)]
impl<T> IntoSocket for T
where
    OwnedSocket: From<T>,
{
    #[inline]
    fn into_socket(self) -> OwnedSocket {
        self.into()
    }
}

pub use portability::{
    AsFilelike, AsSocketlike, BorrowedFilelike, BorrowedSocketlike, FromFilelike, FromSocketlike,
    IntoFilelike, IntoSocketlike, OwnedFilelike, OwnedSocketlike,
};

#[cfg(feature = "close")]
pub mod example_ffi;
pub mod raw;
pub mod views;

// Ideally, we'd want crates to implement our traits themselves. But for now,
// while we're prototyping, we provide a few impls on foreign types.
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "async-std")]
mod impls_async_std;
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "fs-err")]
mod impls_fs_err;
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "mio")]
mod impls_mio;
#[cfg(not(target_os = "wasi"))]
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "os_pipe")]
mod impls_os_pipe;
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "socket2")]
mod impls_socket2;
#[cfg(not(io_safety_is_in_std))]
#[cfg(feature = "tokio")]
mod impls_tokio;
