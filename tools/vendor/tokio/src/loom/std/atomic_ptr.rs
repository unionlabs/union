use std::fmt;
use std::ops::{Deref, DerefMut};

/// `AtomicPtr` providing an additional `load_unsync` function.
pub(crate) struct AtomicPtr<T> {
    inner: std::sync::atomic::AtomicPtr<T>,
}

impl<T> AtomicPtr<T> {
    pub(crate) fn new(ptr: *mut T) -> AtomicPtr<T> {
        let inner = std::sync::atomic::AtomicPtr::new(ptr);
        AtomicPtr { inner }
    }
}

impl<T> Deref for AtomicPtr<T> {
    type Target = std::sync::atomic::AtomicPtr<T>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> DerefMut for AtomicPtr<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<T> fmt::Debug for AtomicPtr<T> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(fmt)
    }
}
