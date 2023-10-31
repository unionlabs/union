use std::cell::UnsafeCell;
use std::fmt;
use std::ops::Deref;

/// `AtomicU8` providing an additional `load_unsync` function.
pub(crate) struct AtomicU8 {
    inner: UnsafeCell<std::sync::atomic::AtomicU8>,
}

unsafe impl Send for AtomicU8 {}
unsafe impl Sync for AtomicU8 {}

impl AtomicU8 {
    pub(crate) const fn new(val: u8) -> AtomicU8 {
        let inner = UnsafeCell::new(std::sync::atomic::AtomicU8::new(val));
        AtomicU8 { inner }
    }
}

impl Deref for AtomicU8 {
    type Target = std::sync::atomic::AtomicU8;

    fn deref(&self) -> &Self::Target {
        // safety: it is always safe to access `&self` fns on the inner value as
        // we never perform unsafe mutations.
        unsafe { &*self.inner.get() }
    }
}

impl fmt::Debug for AtomicU8 {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.deref().fmt(fmt)
    }
}
