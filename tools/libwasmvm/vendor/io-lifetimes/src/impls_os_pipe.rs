//! Implementations of io-lifetimes' traits for os_pipe's types. In the
//! future, we'll prefer to have crates provide their own impls; this is
//! just a temporary measure.

#[cfg(any(unix, target_os = "wasi"))]
use crate::{AsFd, BorrowedFd, FromFd, IntoFd, OwnedFd};
#[cfg(windows)]
use crate::{AsHandle, BorrowedHandle, FromHandle, IntoHandle, OwnedHandle};
#[cfg(unix)]
use std::os::unix::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(target_os = "wasi")]
use std::os::wasi::io::{AsRawFd, FromRawFd, IntoRawFd};
#[cfg(windows)]
use std::os::windows::io::{AsRawHandle, FromRawHandle, IntoRawHandle};

#[cfg(any(unix, target_os = "wasi"))]
impl AsFd for os_pipe::PipeReader {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for os_pipe::PipeReader {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl IntoFd for os_pipe::PipeReader {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl From<os_pipe::PipeReader> for OwnedFd {
    #[inline]
    fn from(owned: os_pipe::PipeReader) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for os_pipe::PipeReader {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<os_pipe::PipeReader> for OwnedHandle {
    #[inline]
    fn from(owned: os_pipe::PipeReader) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl FromFd for os_pipe::PipeReader {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl From<OwnedFd> for os_pipe::PipeReader {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromHandle for os_pipe::PipeReader {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<OwnedHandle> for os_pipe::PipeReader {
    #[inline]
    fn from(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl AsFd for os_pipe::PipeWriter {
    #[inline]
    fn as_fd(&self) -> BorrowedFd<'_> {
        unsafe { BorrowedFd::borrow_raw(self.as_raw_fd()) }
    }
}

#[cfg(windows)]
impl AsHandle for os_pipe::PipeWriter {
    #[inline]
    fn as_handle(&self) -> BorrowedHandle<'_> {
        unsafe { BorrowedHandle::borrow_raw(self.as_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl IntoFd for os_pipe::PipeWriter {
    #[inline]
    fn into_fd(self) -> OwnedFd {
        unsafe { OwnedFd::from_raw_fd(self.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl From<os_pipe::PipeWriter> for OwnedFd {
    #[inline]
    fn from(owned: os_pipe::PipeWriter) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl IntoHandle for os_pipe::PipeWriter {
    #[inline]
    fn into_handle(self) -> OwnedHandle {
        unsafe { OwnedHandle::from_raw_handle(self.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<os_pipe::PipeWriter> for OwnedHandle {
    #[inline]
    fn from(owned: os_pipe::PipeWriter) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl FromFd for os_pipe::PipeWriter {
    #[inline]
    fn from_fd(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(any(unix, target_os = "wasi"))]
impl From<OwnedFd> for os_pipe::PipeWriter {
    #[inline]
    fn from(owned: OwnedFd) -> Self {
        unsafe { Self::from_raw_fd(owned.into_raw_fd()) }
    }
}

#[cfg(windows)]
impl FromHandle for os_pipe::PipeWriter {
    #[inline]
    fn from_handle(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}

#[cfg(windows)]
impl From<OwnedHandle> for os_pipe::PipeWriter {
    #[inline]
    fn from(owned: OwnedHandle) -> Self {
        unsafe { Self::from_raw_handle(owned.into_raw_handle()) }
    }
}
