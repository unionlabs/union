use std::cell::UnsafeCell;
use std::fmt;
use std::ops::Deref;

/// `AtomicU32` providing an additional `load_unsync` function.
pub(crate) struct AtomicU32 {
    inner: UnsafeCell<std::sync::atomic::AtomicU32>,
}

unsafe impl Send for AtomicU32 {}
unsafe impl Sync for AtomicU32 {}

impl AtomicU32 {
    pub(crate) const fn new(val: u32) -> AtomicU32 {
        let inner = UnsafeCell::new(std::sync::atomic::AtomicU32::new(val));
        AtomicU32 { inner }
    }
}

impl Deref for AtomicU32 {
    type Target = std::sync::atomic::AtomicU32;

    fn deref(&self) -> &Self::Target {
        // safety: it is always safe to access `&self` fns on the inner value as
        // we never perform unsafe mutations.
        unsafe { &*self.inner.get() }
    }
}

impl fmt::Debug for AtomicU32 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(fmt)
    }
}
