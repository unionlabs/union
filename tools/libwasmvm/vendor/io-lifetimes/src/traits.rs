#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
use crate::BorrowedFd;
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
use crate::OwnedFd;
#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
use crate::{BorrowedHandle, BorrowedSocket};
#[cfg(windows)]
use crate::{OwnedHandle, OwnedSocket};

/// A trait to borrow the file descriptor from an underlying object.
///
/// This is only available on unix platforms and must be imported in order to
/// call the method. Windows platforms have a corresponding `AsHandle` and
/// `AsSocket` set of traits.
#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
pub trait AsFd {
    /// Borrows the file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{AsFd, BorrowedFd};
    ///
    /// let mut f = File::open("foo.txt")?;
    /// let borrowed_fd: BorrowedFd<'_> = f.as_fd();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn as_fd(&self) -> BorrowedFd<'_>;
}

/// A trait to borrow the handle from an underlying object.
#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
pub trait AsHandle {
    /// Borrows the handle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{AsHandle, BorrowedHandle};
    ///
    /// let mut f = File::open("foo.txt")?;
    /// let borrowed_handle: BorrowedHandle<'_> = f.as_handle();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn as_handle(&self) -> BorrowedHandle<'_>;
}

/// A trait to borrow the socket from an underlying object.
#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
pub trait AsSocket {
    /// Borrows the socket.
    fn as_socket(&self) -> BorrowedSocket<'_>;
}

/// A trait to express the ability to consume an object and acquire ownership
/// of its file descriptor.
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
#[deprecated(
    since = "1.0.0",
    note = "`IntoFd` is replaced by `From<...> for OwnedFd` or `Into<OwnedFd>`"
)]
pub trait IntoFd {
    /// Consumes this object, returning the underlying file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{IntoFd, OwnedFd};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_fd: OwnedFd = f.into_fd();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn into_fd(self) -> OwnedFd;
}

/// A trait to express the ability to consume an object and acquire ownership
/// of its handle.
#[cfg(windows)]
#[deprecated(
    since = "1.0.0",
    note = "`IntoHandle` is replaced by `From<...> for OwnedHandle` or `Into<OwnedHandle>`"
)]
pub trait IntoHandle {
    /// Consumes this object, returning the underlying handle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{IntoHandle, OwnedHandle};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_handle: OwnedHandle = f.into_handle();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn into_handle(self) -> OwnedHandle;
}

/// A trait to express the ability to consume an object and acquire ownership
/// of its socket.
#[cfg(windows)]
#[deprecated(
    since = "1.0.0",
    note = "`IntoSocket` is replaced by `From<...> for OwnedSocket` or `Into<OwnedSocket>`"
)]
pub trait IntoSocket {
    /// Consumes this object, returning the underlying socket.
    fn into_socket(self) -> OwnedSocket;
}

/// A trait to express the ability to construct an object from a file
/// descriptor.
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
pub trait FromFd {
    /// Constructs a new instance of `Self` from the given file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFd, IntoFd, OwnedFd};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_fd: OwnedFd = f.into_fd();
    /// let f = File::from_fd(owned_fd);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[deprecated(
        since = "1.0.0",
        note = "`FromFd::from_fd` is replaced by `From<OwnedFd>::from`"
    )]
    fn from_fd(owned: OwnedFd) -> Self;

    /// Constructs a new instance of `Self` from the given file descriptor
    /// converted from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFd, IntoFd};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_fd(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[inline]
    fn from_into_fd<Owned: Into<OwnedFd>>(into_owned: Owned) -> Self
    where
        Self: Sized + From<OwnedFd>,
    {
        Self::from(into_owned.into())
    }
}

/// A trait to express the ability to construct an object from a handle.
#[cfg(windows)]
pub trait FromHandle {
    /// Constructs a new instance of `Self` from the given handle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromHandle, IntoHandle, OwnedHandle};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_handle: OwnedHandle = f.into_handle();
    /// let f = File::from_handle(owned_handle);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[deprecated(
        since = "1.0.0",
        note = "`FromHandle::from_handle` is replaced by `From<OwnedHandle>::from`"
    )]
    fn from_handle(owned: OwnedHandle) -> Self;

    /// Constructs a new instance of `Self` from the given handle converted
    /// from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromHandle, IntoHandle};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_handle(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[inline]
    fn from_into_handle<Owned: Into<OwnedHandle>>(into_owned: Owned) -> Self
    where
        Self: Sized + From<OwnedHandle>,
    {
        Self::from(into_owned.into())
    }
}

/// A trait to express the ability to construct an object from a socket.
#[cfg(windows)]
pub trait FromSocket {
    /// Constructs a new instance of `Self` from the given socket.
    #[deprecated(
        since = "1.0.0",
        note = "`FromSocket::from_socket` is replaced by `From<OwnedSocket>::from`"
    )]
    fn from_socket(owned: OwnedSocket) -> Self;

    /// Constructs a new instance of `Self` from the given socket converted
    /// from `into_owned`.
    #[inline]
    fn from_into_socket<Owned: Into<OwnedSocket>>(into_owned: Owned) -> Self
    where
        Self: Sized + From<OwnedSocket>,
    {
        Self::from(into_owned.into())
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl<T: AsFd> AsFd for &T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(any(unix, target_os = "wasi", target_os = "hermit"))]
impl<T: AsFd> AsFd for &mut T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
impl<T: AsHandle> AsHandle for &T {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        T::as_handle(self)
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
impl<T: AsHandle> AsHandle for &mut T {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        T::as_handle(self)
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
impl<T: AsSocket> AsSocket for &T {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        T::as_socket(self)
    }
}

#[cfg(not(io_safety_is_in_std))]
#[cfg(windows)]
impl<T: AsSocket> AsSocket for &mut T {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        T::as_socket(self)
    }
}
