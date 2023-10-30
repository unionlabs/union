//! This module contains a type that can make `Send + !Sync` types `Sync` by
//! disallowing all immutable access to the value.
//!
//! A similar primitive is provided in the `sync_wrapper` crate.

pub(crate) struct SyncWrapper<T> {
    value: T,
}

// safety: The SyncWrapper being send allows you to send the inner value across
// thread boundaries.
unsafe impl<T: Send> Send for SyncWrapper<T> {}

// safety: An immutable reference to a SyncWrapper is useless, so moving such an
// immutable reference across threads is safe.
unsafe impl<T> Sync for SyncWrapper<T> {}

impl<T> SyncWrapper<T> {
    pub(crate) fn new(value: T) -> Self {
        Self { value }
    }

    pub(crate) fn into_inner(self) -> T {
        self.value
    }
}
