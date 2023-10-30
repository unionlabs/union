use crate::sync::rwlock::owned_read_guard::OwnedRwLockReadGuard;
use crate::sync::rwlock::owned_write_guard_mapped::OwnedRwLockMappedWriteGuard;
use crate::sync::rwlock::RwLock;
use std::fmt;
use std::marker::PhantomData;
use std::mem::{self, ManuallyDrop};
use std::ops;
use std::sync::Arc;

/// Owned RAII structure used to release the exclusive write access of a lock when
/// dropped.
///
/// This structure is created by the [`write_owned`] method
/// on [`RwLock`].
///
/// [`write_owned`]: method@crate::sync::RwLock::write_owned
/// [`RwLock`]: struct@crate::sync::RwLock
pub struct OwnedRwLockWriteGuard<T: ?Sized> {
    #[cfg(all(tokio_unstable, feature = "tracing"))]
    pub(super) resource_span: tracing::Span,
    pub(super) permits_acquired: u32,
    // ManuallyDrop allows us to destructure into this field without running the destructor.
    pub(super) lock: ManuallyDrop<Arc<RwLock<T>>>,
    pub(super) data: *mut T,
    pub(super) _p: PhantomData<T>,
}

impl<T: ?Sized> OwnedRwLockWriteGuard<T> {
    /// Makes a new [`OwnedRwLockMappedWriteGuard`] for a component of the locked
    /// data.
    ///
    /// This operation cannot fail as the `OwnedRwLockWriteGuard` passed in
    /// already locked the data.
    ///
    /// This is an associated function that needs to be used as
    /// `OwnedRwLockWriteGuard::map(..)`. A method would interfere with methods
    /// of the same name on the contents of the locked data.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use tokio::sync::{RwLock, OwnedRwLockWriteGuard};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// struct Foo(u32);
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let lock = Arc::new(RwLock::new(Foo(1)));
    ///
    /// {
    ///     let lock = Arc::clone(&lock);
    ///     let mut mapped = OwnedRwLockWriteGuard::map(lock.write_owned().await, |f| &mut f.0);
    ///     *mapped = 2;
    /// }
    ///
    /// assert_eq!(Foo(2), *lock.read().await);
    /// # }
    /// ```
    #[inline]
    pub fn map<F, U: ?Sized>(mut this: Self, f: F) -> OwnedRwLockMappedWriteGuard<T, U>
    where
        F: FnOnce(&mut T) -> &mut U,
    {
        let data = f(&mut *this) as *mut U;
        let lock = unsafe { ManuallyDrop::take(&mut this.lock) };
        let permits_acquired = this.permits_acquired;
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        let resource_span = this.resource_span.clone();
        // NB: Forget to avoid drop impl from being called.
        mem::forget(this);

        OwnedRwLockMappedWriteGuard {
            permits_acquired,
            lock: ManuallyDrop::new(lock),
            data,
            _p: PhantomData,
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            resource_span,
        }
    }

    /// Attempts to make  a new [`OwnedRwLockMappedWriteGuard`] for a component
    /// of the locked data. The original guard is returned if the closure
    /// returns `None`.
    ///
    /// This operation cannot fail as the `OwnedRwLockWriteGuard` passed in
    /// already locked the data.
    ///
    /// This is an associated function that needs to be
    /// used as `OwnedRwLockWriteGuard::try_map(...)`. A method would interfere
    /// with methods of the same name on the contents of the locked data.
    ///
    /// [`RwLockMappedWriteGuard`]: struct@crate::sync::RwLockMappedWriteGuard
    ///
    /// # Examples
    ///
    /// ```
    /// use std::sync::Arc;
    /// use tokio::sync::{RwLock, OwnedRwLockWriteGuard};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// struct Foo(u32);
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let lock = Arc::new(RwLock::new(Foo(1)));
    ///
    /// {
    ///     let guard = Arc::clone(&lock).write_owned().await;
    ///     let mut guard = OwnedRwLockWriteGuard::try_map(guard, |f| Some(&mut f.0)).expect("should not fail");
    ///     *guard = 2;
    /// }
    ///
    /// assert_eq!(Foo(2), *lock.read().await);
    /// # }
    /// ```
    #[inline]
    pub fn try_map<F, U: ?Sized>(
        mut this: Self,
        f: F,
    ) -> Result<OwnedRwLockMappedWriteGuard<T, U>, Self>
    where
        F: FnOnce(&mut T) -> Option<&mut U>,
    {
        let data = match f(&mut *this) {
            Some(data) => data as *mut U,
            None => return Err(this),
        };
        let permits_acquired = this.permits_acquired;
        let lock = unsafe { ManuallyDrop::take(&mut this.lock) };
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        let resource_span = this.resource_span.clone();

        // NB: Forget to avoid drop impl from being called.
        mem::forget(this);

        Ok(OwnedRwLockMappedWriteGuard {
            permits_acquired,
            lock: ManuallyDrop::new(lock),
            data,
            _p: PhantomData,
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            resource_span,
        })
    }

    /// Converts this `OwnedRwLockWriteGuard` into an
    /// `OwnedRwLockMappedWriteGuard`. This method can be used to store a
    /// non-mapped guard in a struct field that expects a mapped guard.
    ///
    /// This is equivalent to calling `OwnedRwLockWriteGuard::map(guard, |me| me)`.
    #[inline]
    pub fn into_mapped(this: Self) -> OwnedRwLockMappedWriteGuard<T> {
        Self::map(this, |me| me)
    }

    /// Atomically downgrades a write lock into a read lock without allowing
    /// any writers to take exclusive access of the lock in the meantime.
    ///
    /// **Note:** This won't *necessarily* allow any additional readers to acquire
    /// locks, since [`RwLock`] is fair and it is possible that a writer is next
    /// in line.
    ///
    /// Returns an RAII guard which will drop this read access of the `RwLock`
    /// when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use tokio::sync::RwLock;
    /// # use std::sync::Arc;
    /// #
    /// # #[tokio::main]
    /// # async fn main() {
    /// let lock = Arc::new(RwLock::new(1));
    ///
    /// let n = lock.clone().write_owned().await;
    ///
    /// let cloned_lock = lock.clone();
    /// let handle = tokio::spawn(async move {
    ///     *cloned_lock.write_owned().await = 2;
    /// });
    ///
    /// let n = n.downgrade();
    /// assert_eq!(*n, 1, "downgrade is atomic");
    ///
    /// drop(n);
    /// handle.await.unwrap();
    /// assert_eq!(*lock.read().await, 2, "second writer obtained write lock");
    /// # }
    /// ```
    pub fn downgrade(mut self) -> OwnedRwLockReadGuard<T> {
        let lock = unsafe { ManuallyDrop::take(&mut self.lock) };
        let data = self.data;
        let to_release = (self.permits_acquired - 1) as usize;

        // Release all but one of the permits held by the write guard
        lock.s.release(to_release);
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        self.resource_span.in_scope(|| {
            tracing::trace!(
            target: "runtime::resource::state_update",
            write_locked = false,
            write_locked.op = "override",
            )
        });

        #[cfg(all(tokio_unstable, feature = "tracing"))]
        self.resource_span.in_scope(|| {
            tracing::trace!(
            target: "runtime::resource::state_update",
            current_readers = 1,
            current_readers.op = "add",
            )
        });

        #[cfg(all(tokio_unstable, feature = "tracing"))]
        let resource_span = self.resource_span.clone();
        // NB: Forget to avoid drop impl from being called.
        mem::forget(self);

        OwnedRwLockReadGuard {
            lock: ManuallyDrop::new(lock),
            data,
            _p: PhantomData,
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            resource_span,
        }
    }
}

impl<T: ?Sized> ops::Deref for OwnedRwLockWriteGuard<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<T: ?Sized> ops::DerefMut for OwnedRwLockWriteGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data }
    }
}

impl<T: ?Sized> fmt::Debug for OwnedRwLockWriteGuard<T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: ?Sized> fmt::Display for OwnedRwLockWriteGuard<T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<T: ?Sized> Drop for OwnedRwLockWriteGuard<T> {
    fn drop(&mut self) {
        self.lock.s.release(self.permits_acquired as usize);
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        self.resource_span.in_scope(|| {
            tracing::trace!(
            target: "runtime::resource::state_update",
            write_locked = false,
            write_locked.op = "override",
            )
        });
        unsafe { ManuallyDrop::drop(&mut self.lock) };
    }
}
