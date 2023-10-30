use rustix::fd::BorrowedFd;
use rustix::fs::{flock, FlockOperation};
use std::io::{self, Error, ErrorKind};
use std::os::unix::io::AsRawFd;

use super::{RwLockReadGuard, RwLockWriteGuard};

#[derive(Debug)]
pub struct RwLock<T: AsRawFd> {
    pub(crate) inner: T,
}

impl<T: AsRawFd> RwLock<T> {
    #[inline]
    pub fn new(inner: T) -> Self {
        RwLock { inner }
    }

    #[inline]
    pub fn write(&mut self) -> io::Result<RwLockWriteGuard<'_, T>> {
        flock(&self.as_fd(), FlockOperation::LockExclusive)?;
        Ok(RwLockWriteGuard::new(self))
    }

    #[inline]
    pub fn try_write(&mut self) -> Result<RwLockWriteGuard<'_, T>, Error> {
        flock(&self.as_fd(), FlockOperation::NonBlockingLockExclusive).map_err(|err| match err
            .kind()
        {
            ErrorKind::AlreadyExists => ErrorKind::WouldBlock.into(),
            _ => Error::from(err),
        })?;
        Ok(RwLockWriteGuard::new(self))
    }

    #[inline]
    pub fn read(&self) -> io::Result<RwLockReadGuard<'_, T>> {
        flock(&self.as_fd(), FlockOperation::LockShared)?;
        Ok(RwLockReadGuard::new(self))
    }

    #[inline]
    pub fn try_read(&self) -> Result<RwLockReadGuard<'_, T>, Error> {
        flock(&self.as_fd(), FlockOperation::NonBlockingLockShared).map_err(|err| {
            match err.kind() {
                ErrorKind::AlreadyExists => ErrorKind::WouldBlock.into(),
                _ => Error::from(err),
            }
        })?;
        Ok(RwLockReadGuard::new(self))
    }

    #[inline]
    pub fn into_inner(self) -> T
    where
        T: Sized,
    {
        self.inner
    }

    #[inline]
    pub(crate) fn as_fd(&self) -> BorrowedFd<'_> {
        // Safety: We assume that `self.inner`'s file descriptor is valid for
        // at least the lifetime of `self`.
        //
        // Once I/O safety is stablized in std, we can switch the public API to
        // use `AsFd` instead of `AsRawFd` and eliminate this `unsafe` block.
        unsafe { BorrowedFd::borrow_raw(self.inner.as_raw_fd()) }
    }
}
