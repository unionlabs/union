//! Implementations of io-lifetimes' traits for fs_err's types. In the
//! future, we'll prefer to have crates provide their own impls; this is
//! just a temporary measure.

#[cfg(any(unix, target_os = "wasi"))]
use crate::{AsFd, BorrowedFd, IntoFd, OwnedFd};
#[cfg(windows)]
use crate::{AsHandle, BorrowedHandle, IntoHandle, OwnedHandle};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};

#[cfg(any(unix, target_os = "wasi"))]
impl AsFd for fs_err::File {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for fs_err::File {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl IntoFd for fs_err::File {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl From<fs_err::File> for OwnedFd {
    #[inline]
    fn from(owned: fs_err::File) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for fs_err::File {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<fs_err::File> for OwnedHandle {
    #[inline]
    fn from(owned: fs_err::File) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}
