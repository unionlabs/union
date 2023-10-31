use crate::loom::cell::UnsafeCell;

use std::collections::VecDeque;
use std::marker::PhantomData;

/// This type is like VecDeque, except that it is not Sync and can be modified
/// through immutable references.
pub(crate) struct VecDequeCell<T> {
    inner: UnsafeCell<VecDeque<T>>,
    _not_sync: PhantomData<*const ()>,
}

// This is Send for the same reasons that RefCell<VecDeque<T>> is Send.
unsafe impl<T: Send> Send for VecDequeCell<T> {}

impl<T> VecDequeCell<T> {
    pub(crate) fn with_capacity(cap: usize) -> Self {
        Self {
            inner: UnsafeCell::new(VecDeque::with_capacity(cap)),
            _not_sync: PhantomData,
        }
    }

    /// Safety: This method may not be called recursively.
    #[inline]
    unsafe fn with_inner<F, R>(&self, f: F) -> R
    where
        F: FnOnce(&mut VecDeque<T>) -> R,
    {
        // safety: This type is not Sync, so concurrent calls of this method
        // cannot happen. Furthermore, the caller guarantees that the method is
        // not called recursively. Finally, this is the only place that can
        // create mutable references to the inner VecDeque. This ensures that
        // any mutable references created here are exclusive.
        self.inner.with_mut(|ptr| f(&mut *ptr))
    }

    pub(crate) fn pop_front(&self) -> Option<T> {
        unsafe { self.with_inner(VecDeque::pop_front) }
    }

    pub(crate) fn push_back(&self, item: T) {
        unsafe {
            self.with_inner(|inner| inner.push_back(item));
        }
    }

    /// Replaces the inner VecDeque with an empty VecDeque and return the current
    /// contents.
    pub(crate) fn take(&self) -> VecDeque<T> {
        unsafe { self.with_inner(|inner| std::mem::take(inner)) }
    }
}
