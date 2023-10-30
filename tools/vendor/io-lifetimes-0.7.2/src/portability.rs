//! Portability abstractions over `Owned*` and `Borrowed*`.
//!
//! On Unix, "everything is a file descriptor". On Windows, file/pipe/process
//! handles are distinct from socket descriptors. This file provides a minimal
//! layer of portability over this difference.

use crate::views::{FilelikeView, FilelikeViewType, SocketlikeView, SocketlikeViewType};
#[cfg(any(unix, target_os = "wasi"))]
use crate::{AsFd, BorrowedFd, FromFd, IntoFd, OwnedFd};
#[cfg(windows)]
use crate::{
    AsHandle, AsSocket, BorrowedHandle, BorrowedSocket, FromHandle, FromSocket, IntoHandle,
    IntoSocket, OwnedHandle, OwnedSocket,
};

/// A reference to a filelike object.
///
/// This is a portability abstraction over Unix-like [`BorrowedFd`] and
/// Windows' `BorrowedHandle`.
#[cfg(any(unix, target_os = "wasi"))]
pub type BorrowedFilelike<'filelike> = BorrowedFd<'filelike>;

/// A reference to a filelike object.
///
/// This is a portability abstraction over Unix-like `BorrowedFd` and
/// Windows' [`BorrowedHandle`].
#[cfg(windows)]
pub type BorrowedFilelike<'filelike> = BorrowedHandle<'filelike>;

/// A reference to a socketlike object.
///
/// This is a portability abstraction over Unix-like [`BorrowedFd`] and
/// Windows' `BorrowedSocket`.
#[cfg(any(unix, target_os = "wasi"))]
pub type BorrowedSocketlike<'socketlike> = BorrowedFd<'socketlike>;

/// A reference to a socketlike object.
///
/// This is a portability abstraction over Unix-like `BorrowedFd` and
/// Windows' [`BorrowedSocket`].
#[cfg(windows)]
pub type BorrowedSocketlike<'socketlike> = BorrowedSocket<'socketlike>;

/// An owned filelike object.
///
/// This is a portability abstraction over Unix-like [`OwnedFd`] and
/// Windows' `OwnedHandle`.
#[cfg(any(unix, target_os = "wasi"))]
pub type OwnedFilelike = OwnedFd;

/// An owned filelike object.
///
/// This is a portability abstraction over Unix-like `OwnedFd` and
/// Windows' [`OwnedHandle`].
#[cfg(windows)]
pub type OwnedFilelike = OwnedHandle;

/// An owned socketlike object.
///
/// This is a portability abstraction over Unix-like [`OwnedFd`] and
/// Windows' `OwnedSocket`.
#[cfg(any(unix, target_os = "wasi"))]
pub type OwnedSocketlike = OwnedFd;

/// An owned socketlike object.
///
/// This is a portability abstraction over Unix-like `OwnedFd` and
/// Windows' [`OwnedSocket`].
#[cfg(windows)]
pub type OwnedSocketlike = OwnedSocket;

/// A portable trait to borrow a reference from an underlying filelike object.
///
/// This is a portability abstraction over Unix-like [`AsFd`] and Windows'
/// `AsHandle`. It also provides the `as_filelike_view` convenience function
/// providing typed views.
#[cfg(any(unix, target_os = "wasi"))]
pub trait AsFilelike: AsFd {
    /// Borrows the reference.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{AsFilelike, BorrowedFilelike};
    ///
    /// let mut f = File::open("foo.txt")?;
    /// let borrowed_filelike: BorrowedFilelike<'_> = f.as_filelike();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn as_filelike(&self) -> BorrowedFilelike<'_>;

    /// Return a borrowing view of a resource which dereferences to a `&Target`.
    ///
    /// Note that [`Read`] or [`Write`] require `&mut Target`, but in some cases,
    /// such as [`File`], `Read` and `Write` are implemented for `&Target` in
    /// addition to `Target`, and you can get a `&mut &Target` by doing `&*` on
    /// the resuting view, like this:
    ///
    /// ```rust,ignore
    /// let v = f.as_filelike_view::<std::fs::File>();
    /// (&*v).read(&mut buf).unwrap();
    /// ```
    ///
    /// [`File`]: std::fs::File
    fn as_filelike_view<Target: FilelikeViewType>(&self) -> FilelikeView<'_, Target>;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: AsFd> AsFilelike for T {
    #[inline]
    fn as_filelike(&self) -> BorrowedFilelike<'_> {
        self.as_fd()
    }

    #[inline]
    fn as_filelike_view<Target: FilelikeViewType>(&self) -> FilelikeView<'_, Target> {
        FilelikeView::new(self)
    }
}

/// A portable trait to borrow a reference from an underlying filelike object.
///
/// This is a portability abstraction over Unix-like `AsFd` and Windows'
/// [`AsHandle`]. It also provides the `as_filelike_view` convenience function
/// providing typed views.
#[cfg(windows)]
pub trait AsFilelike: AsHandle {
    /// Borrows the reference.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{AsFilelike, BorrowedFilelike};
    ///
    /// let mut f = File::open("foo.txt")?;
    /// let borrowed_filelike: BorrowedFilelike<'_> = f.as_filelike();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn as_filelike(&self) -> BorrowedFilelike<'_>;

    /// Return a borrowing view of a resource which dereferences to a `&Target`.
    ///
    /// Note that [`Read`] or [`Write`] require `&mut Target`, but in some cases,
    /// such as [`File`], `Read` and `Write` are implemented for `&Target` in
    /// addition to `Target`, and you can get a `&mut &Target` by doing `&*` on
    /// the resuting view, like this:
    ///
    /// ```rust,ignore
    /// let v = f.as_filelike_view::<std::fs::File>();
    /// (&*v).read(&mut buf).unwrap();
    /// ```
    ///
    /// [`File`]: std::fs::File
    fn as_filelike_view<Target: FilelikeViewType>(&self) -> FilelikeView<'_, Target>;
}

#[cfg(windows)]
impl<T: AsHandle> AsFilelike for T {
    #[inline]
    fn as_filelike(&self) -> BorrowedFilelike<'_> {
        self.as_handle()
    }

    #[inline]
    fn as_filelike_view<Target: FilelikeViewType>(&self) -> FilelikeView<'_, Target> {
        FilelikeView::new(self)
    }
}

/// A portable trait to borrow a reference from an underlying socketlike
/// object.
///
/// This is a portability abstraction over Unix-like [`AsFd`] and Windows'
/// `AsSocket`. It also provides the `as_socketlike_view` convenience
/// function providing typed views.
#[cfg(any(unix, target_os = "wasi"))]
pub trait AsSocketlike: AsFd {
    /// Borrows the reference.
    fn as_socketlike(&self) -> BorrowedSocketlike<'_>;

    /// Return a borrowing view of a resource which dereferences to a `&Target`.
    ///
    /// Note that [`Read`] or [`Write`] require `&mut Target`, but in some cases,
    /// such as [`TcpStream`], `Read` and `Write` are implemented for `&Target` in
    /// addition to `Target`, and you can get a `&mut &Target` by doing `&*` on
    /// the resuting view, like this:
    ///
    /// ```rust,ignore
    /// let v = s.as_socketlike_view::<std::net::TcpStream>();
    /// (&*v).read(&mut buf).unwrap();
    /// ```
    ///
    /// [`TcpStream`]: std::net::TcpStream
    fn as_socketlike_view<Target: SocketlikeViewType>(&self) -> SocketlikeView<'_, Target>;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: AsFd> AsSocketlike for T {
    #[inline]
    fn as_socketlike(&self) -> BorrowedSocketlike<'_> {
        self.as_fd()
    }

    #[inline]
    fn as_socketlike_view<Target: SocketlikeViewType>(&self) -> SocketlikeView<'_, Target> {
        SocketlikeView::new(self)
    }
}

/// A portable trait to borrow a reference from an underlying socketlike
/// object.
///
/// This is a portability abstraction over Unix-like `AsFd` and Windows'
/// [`AsSocket`]. It also provides the `as_socketlike_view` convenience
/// function providing typed views.
#[cfg(windows)]
pub trait AsSocketlike: AsSocket {
    /// Borrows the reference.
    fn as_socketlike(&self) -> BorrowedSocketlike;

    /// Return a borrowing view of a resource which dereferences to a `&Target`.
    ///
    /// Note that [`Read`] or [`Write`] require `&mut Target`, but in some cases,
    /// such as [`TcpStream`], `Read` and `Write` are implemented for `&Target` in
    /// addition to `Target`, and you can get a `&mut &Target` by doing `&*` on
    /// the resuting view, like this:
    ///
    /// ```rust,ignore
    /// let v = s.as_socketlike_view::<std::net::TcpStream>();
    /// (&*v).read(&mut buf).unwrap();
    /// ```
    ///
    /// [`TcpStream`]: std::net::TcpStream
    fn as_socketlike_view<Target: SocketlikeViewType>(&self) -> SocketlikeView<'_, Target>;
}

#[cfg(windows)]
impl<T: AsSocket> AsSocketlike for T {
    #[inline]
    fn as_socketlike(&self) -> BorrowedSocketlike<'_> {
        self.as_socket()
    }

    #[inline]
    fn as_socketlike_view<Target: SocketlikeViewType>(&self) -> SocketlikeView<'_, Target> {
        SocketlikeView::new(self)
    }
}

/// A portable trait to express the ability to consume an object and acquire
/// ownership of its filelike object.
///
/// This is a portability abstraction over Unix-like [`IntoFd`] and Windows'
/// `IntoHandle`.
#[cfg(any(unix, target_os = "wasi"))]
pub trait IntoFilelike: IntoFd {
    /// Consumes this object, returning the underlying filelike object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{IntoFilelike, OwnedFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_filelike: OwnedFilelike = f.into_filelike();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn into_filelike(self) -> OwnedFilelike;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: IntoFd> IntoFilelike for T {
    #[inline]
    fn into_filelike(self) -> OwnedFilelike {
        self.into_fd()
    }
}

/// A portable trait to express the ability to consume an object and acquire
/// ownership of its filelike object.
///
/// This is a portability abstraction over Unix-like `IntoFd` and Windows'
/// [`IntoHandle`].
#[cfg(windows)]
pub trait IntoFilelike: IntoHandle {
    /// Consumes this object, returning the underlying filelike object.
    fn into_filelike(self) -> OwnedFilelike;
}

#[cfg(windows)]
impl<T: IntoHandle> IntoFilelike for T {
    #[inline]
    fn into_filelike(self) -> OwnedFilelike {
        self.into_handle()
    }
}

/// A portable trait to express the ability to consume an object and acquire
/// ownership of its socketlike object.
///
/// This is a portability abstraction over Unix-like [`IntoFd`] and Windows'
/// `IntoSocket`.
#[cfg(any(unix, target_os = "wasi"))]
pub trait IntoSocketlike: IntoFd {
    /// Consumes this object, returning the underlying socketlike object.
    fn into_socketlike(self) -> OwnedSocketlike;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: IntoFd> IntoSocketlike for T {
    #[inline]
    fn into_socketlike(self) -> OwnedSocketlike {
        self.into_fd()
    }
}

/// A portable trait to express the ability to consume an object and acquire
/// ownership of its socketlike object.
///
/// This is a portability abstraction over Unix-like `IntoFd` and Windows'
/// [`IntoSocket`].
#[cfg(windows)]
pub trait IntoSocketlike: IntoSocket {
    /// Consumes this object, returning the underlying socketlike object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{IntoFilelike, OwnedFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_filelike: OwnedFilelike = f.into_filelike();
    /// # Ok::<(), io::Error>(())
    /// ```
    fn into_socketlike(self) -> OwnedSocketlike;
}

#[cfg(windows)]
impl<T: IntoSocket> IntoSocketlike for T {
    #[inline]
    fn into_socketlike(self) -> OwnedSocketlike {
        self.into_socket()
    }
}

/// A portable trait to express the ability to construct an object from a
/// filelike object.
///
/// This is a portability abstraction over Unix-like [`FromFd`] and Windows'
/// `FromHandle`. It also provides the `from_into_filelike` convenience
/// function providing simplified from+into conversions.
#[cfg(any(unix, target_os = "wasi"))]
pub trait FromFilelike: FromFd {
    /// Constructs a new instance of `Self` from the given filelike object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFilelike, IntoFilelike, OwnedFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_filelike: OwnedFilelike = f.into_filelike();
    /// let f = File::from_filelike(owned_filelike);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_filelike(owned: OwnedFilelike) -> Self;

    /// Constructs a new instance of `Self` from the given filelike object
    /// converted from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFilelike, IntoFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_filelike(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_into_filelike<Owned: IntoFilelike>(owned: Owned) -> Self;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: FromFd> FromFilelike for T {
    #[inline]
    fn from_filelike(owned: OwnedFilelike) -> Self {
        Self::from_fd(owned)
    }

    #[inline]
    fn from_into_filelike<Owned: IntoFilelike>(owned: Owned) -> Self {
        Self::from_filelike(owned.into_filelike())
    }
}

/// A portable trait to express the ability to construct an object from a
/// filelike object.
///
/// This is a portability abstraction over Unix-like `FromFd` and Windows'
/// [`FromHandle`]. It also provides the `from_into_filelike` convenience
/// function providing simplified from+into conversions.
#[cfg(windows)]
pub trait FromFilelike: FromHandle {
    /// Constructs a new instance of `Self` from the given filelike object.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFilelike, IntoFilelike, OwnedFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let owned_filelike: OwnedFilelike = f.into_filelike();
    /// let f = File::from_filelike(owned_filelike);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_filelike(owned: OwnedFilelike) -> Self;

    /// Constructs a new instance of `Self` from the given filelike object
    /// converted from `into_owned`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use std::fs::File;
    /// # use std::io;
    /// use io_lifetimes::{FromFilelike, IntoFilelike};
    ///
    /// let f = File::open("foo.txt")?;
    /// let f = File::from_into_filelike(f);
    /// # Ok::<(), io::Error>(())
    /// ```
    fn from_into_filelike<Owned: IntoFilelike>(owned: Owned) -> Self;
}

#[cfg(windows)]
impl<T: FromHandle> FromFilelike for T {
    #[inline]
    fn from_filelike(owned: OwnedFilelike) -> Self {
        Self::from_handle(owned)
    }

    #[inline]
    fn from_into_filelike<Owned: IntoFilelike>(owned: Owned) -> Self {
        Self::from_filelike(owned.into_filelike())
    }
}

/// A portable trait to express the ability to construct an object from a
/// socketlike object.
///
/// This is a portability abstraction over Unix-like [`FromFd`] and Windows'
/// `FromSocket`. It also provides the `from_into_socketlike` convenience
/// function providing simplified from+into conversions.
#[cfg(any(unix, target_os = "wasi"))]
pub trait FromSocketlike: FromFd {
    /// Constructs a new instance of `Self` from the given socketlike object.
    fn from_socketlike(owned: OwnedSocketlike) -> Self;

    /// Constructs a new instance of `Self` from the given socketlike object
    /// converted from `into_owned`.
    fn from_into_socketlike<Owned: IntoSocketlike>(owned: Owned) -> Self;
}

#[cfg(any(unix, target_os = "wasi"))]
impl<T: FromFd> FromSocketlike for T {
    #[inline]
    fn from_socketlike(owned: OwnedSocketlike) -> Self {
        Self::from_fd(owned)
    }

    #[inline]
    fn from_into_socketlike<Owned: IntoSocketlike>(owned: Owned) -> Self {
        Self::from_socketlike(owned.into_socketlike())
    }
}

/// A portable trait to express the ability to construct an object from a
/// socketlike object.
///
/// This is a portability abstraction over Unix-like `FromFd` and Windows'
/// [`FromSocket`]. It also provides the `from_into_socketlike` convenience
/// function providing simplified from+into conversions.
#[cfg(windows)]
pub trait FromSocketlike: FromSocket {
    /// Constructs a new instance of `Self` from the given socketlike object.
    fn from_socketlike(owned: OwnedSocketlike) -> Self;

    /// Constructs a new instance of `Self` from the given socketlike object
    /// converted from `into_owned`.
    fn from_into_socketlike<Owned: IntoSocketlike>(owned: Owned) -> Self;
}

#[cfg(windows)]
impl<T: FromSocket> FromSocketlike for T {
    #[inline]
    fn from_socketlike(owned: OwnedSocketlike) -> Self {
        Self::from_socket(owned)
    }

    #[inline]
    fn from_into_socketlike<Owned: IntoSocketlike>(owned: Owned) -> Self {
        Self::from_socketlike(owned.into_socketlike())
    }
}
