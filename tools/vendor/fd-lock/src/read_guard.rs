use std::ops;

use crate::sys;

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] and [`try_read`] methods on
/// [`RwLock`].
///
/// [`read`]: crate::RwLock::read
/// [`try_read`]: crate::RwLock::try_read
/// [`RwLock`]: crate::RwLock
///
/// # Panics
///
/// Dropping this type may panic if the lock fails to unlock.
#[must_use = "if unused the RwLock will immediately unlock"]
#[derive(Debug)]
pub struct RwLockReadGuard<'lock, T: sys::AsRaw> {
    guard: sys::RwLockReadGuard<'lock, T>,
}

impl<'lock, T: sys::AsRaw> RwLockReadGuard<'lock, T> {
    pub(crate) fn new(guard: sys::RwLockReadGuard<'lock, T>) -> Self {
        Self { guard }
    }
}

impl<T: sys::AsRaw> ops::Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.guard.deref()
    }
}

/// Release the lock.
impl<T: sys::AsRaw> Drop for RwLockReadGuard<'_, T> {
    #[inline]
    fn drop(&mut self) {}
}
