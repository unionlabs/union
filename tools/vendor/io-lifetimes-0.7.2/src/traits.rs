#[cfg(not(io_lifetimes_use_std))]
#[cfg(any(unix, target_os = "wasi"))]
use crate::BorrowedFd;
#[cfg(any(unix, target_os = "wasi"))]
use crate::OwnedFd;
#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
use crate::{BorrowedHandle, BorrowedSocket};
#[cfg(windows)]
use crate::{OwnedHandle, OwnedSocket};

/// A trait to borrow the file descriptor from an underlying object.
///
/// This is only available on unix platforms and must be imported in order to
/// call the method. Windows platforms have a corresponding `AsHandle` and
/// `AsSocket` set of traits.
#[cfg(not(io_lifetimes_use_std))]
#[cfg(any(unix, target_os = "wasi"))]
pub trait AsFd {
    /// Borrows the file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
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
#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
pub trait AsHandle {
    /// Borrows the handle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
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
#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
pub trait AsSocket {
    /// Borrows the socket.
    fn as_socket(&self) -> BorrowedSocket<'_>;
}

/// A trait to express the ability to consume an object and acquire ownership
/// of its file descriptor.
#[cfg(any(unix, target_os = "wasi"))]
pub trait IntoFd {
    /// Consumes this object, returning the underlying file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
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
pub trait IntoHandle {
    /// Consumes this object, returning the underlying handle.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
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
pub trait IntoSocket {
    /// Consumes this object, returning the underlying socket.
    fn into_socket(self) -> OwnedSocket;
}

/// A trait to express the ability to construct an object from a file
/// descriptor.
#[cfg(any(unix, target_os = "wasi"))]
pub trait FromFd {
    /// Constructs a new instance of `Self` from the given file descriptor.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFd, IntoFd, OwnedFd};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_fd: OwnedFd = f.into_fd();
    /// let f = File::from_fd(owned_fd);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_fd(owned: OwnedFd) -> Self;

    /// Constructs a new instance of `Self` from the given file descriptor
    /// converted from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFd, IntoFd};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_fd(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[inline]
    fn from_into_fd<Owned: IntoFd>(into_owned: Owned) -> Self
    where
        Self: Sized,
    {
        Self::from_fd(into_owned.into_fd())
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
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromHandle, IntoHandle, OwnedHandle};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_handle: OwnedHandle = f.into_handle();
    /// let f = File::from_handle(owned_handle);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_handle(owned: OwnedHandle) -> Self;

    /// Constructs a new instance of `Self` from the given handle converted
    /// from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #![cfg_attr(io_lifetimes_use_std, feature(io_safety))]
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromHandle, IntoHandle};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_handle(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    #[inline]
    fn from_into_handle<Owned: IntoHandle>(into_owned: Owned) -> Self
    where
        Self: Sized,
    {
        Self::from_handle(into_owned.into_handle())
    }
}

/// A trait to express the ability to construct an object from a socket.
#[cfg(windows)]
pub trait FromSocket {
    /// Constructs a new instance of `Self` from the given socket.
    fn from_socket(owned: OwnedSocket) -> Self;

    /// Constructs a new instance of `Self` from the given socket converted
    /// from `into_owned`.
    #[inline]
    fn from_into_socket<Owned: IntoSocket>(into_owned: Owned) -> Self
    where
        Self: Sized,
    {
        Self::from_socket(into_owned.into_socket())
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(any(unix, target_os = "wasi"))]
impl<T: AsFd> AsFd for &T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(any(unix, target_os = "wasi"))]
impl<T: AsFd> AsFd for &mut T {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        T::as_fd(self)
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
impl<T: AsHandle> AsHandle for &T {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        T::as_handle(self)
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
impl<T: AsHandle> AsHandle for &mut T {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        T::as_handle(self)
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
impl<T: AsSocket> AsSocket for &T {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        T::as_socket(self)
    }
}

#[cfg(not(io_lifetimes_use_std))]
#[cfg(windows)]
impl<T: AsSocket> AsSocket for &mut T {
    #[inline]
    fn as_socket(&self) -> BorrowedSocket<'_> {
        T::as_socket(self)
    }
}
