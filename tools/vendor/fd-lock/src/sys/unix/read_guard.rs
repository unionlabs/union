use rustix::fs::{flock, FlockOperation};
use std::ops;
use std::os::unix::io::AsRawFd;

use super::RwLock;

#[derive(Debug)]
pub struct RwLockReadGuard<'lock, T: AsRawFd> {
    lock: &'lock RwLock<T>,
}

impl<'lock, T: AsRawFd> RwLockReadGuard<'lock, T> {
    pub(crate) fn new(lock: &'lock RwLock<T>) -> Self {
        Self { lock }
    }
}

impl<T: AsRawFd> ops::Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.lock.inner
    }
}

impl<T: AsRawFd> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {
        let _ = flock(&self.lock.as_fd(), FlockOperation::Unlock).ok();
    }
}
