use crate::sync::batch_semaphore::Semaphore;
use std::fmt;
use std::marker;
use std::mem;
use std::ops;

/// RAII structure used to release the shared read access of a lock when
/// dropped.
///
/// This structure is created by the [`read`] method on
/// [`RwLock`].
///
/// [`read`]: method@crate::sync::RwLock::read
/// [`RwLock`]: struct@crate::sync::RwLock
pub struct RwLockReadGuard<'a, T: ?Sized> {
    #[cfg(all(tokio_unstable, feature = "tracing"))]
    pub(super) resource_span: tracing::Span,
    pub(super) s: &'a Semaphore,
    pub(super) data: *const T,
    pub(super) marker: marker::PhantomData<&'a T>,
}

impl<'a, T: ?Sized> RwLockReadGuard<'a, T> {
    /// Makes a new `RwLockReadGuard` for a component of the locked data.
    ///
    /// This operation cannot fail as the `RwLockReadGuard` passed in already
    /// locked the data.
    ///
    /// This is an associated function that needs to be
    /// used as `RwLockReadGuard::map(...)`. A method would interfere with
    /// methods of the same name on the contents of the locked data.
    ///
    /// This is an asynchronous version of [`RwLockReadGuard::map`] from the
    /// [`parking_lot` crate].
    ///
    /// [`RwLockReadGuard::map`]: https://docs.rs/lock_api/latest/lock_api/struct.RwLockReadGuard.html#method.map
    /// [`parking_lot` crate]: https://crates.io/crates/parking_lot
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::{RwLock, RwLockReadGuard};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// struct Foo(u32);
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let lock = RwLock::new(Foo(1));
    ///
    /// let guard = lock.read().await;
    /// let guard = RwLockReadGuard::map(guard, |f| &f.0);
    ///
    /// assert_eq!(1, *guard);
    /// # }
    /// ```
    #[inline]
    pub fn map<F, U: ?Sized>(this: Self, f: F) -> RwLockReadGuard<'a, U>
    where
        F: FnOnce(&T) -> &U,
    {
        let data = f(&*this) as *const U;
        let s = this.s;
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        let resource_span = this.resource_span.clone();
        // NB: Forget to avoid drop impl from being called.
        mem::forget(this);

        RwLockReadGuard {
            s,
            data,
            marker: marker::PhantomData,
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            resource_span,
        }
    }

    /// Attempts to make a new [`RwLockReadGuard`] for a component of the
    /// locked data. The original guard is returned if the closure returns
    /// `None`.
    ///
    /// This operation cannot fail as the `RwLockReadGuard` passed in already
    /// locked the data.
    ///
    /// This is an associated function that needs to be used as
    /// `RwLockReadGuard::try_map(..)`. A method would interfere with methods of the
    /// same name on the contents of the locked data.
    ///
    /// This is an asynchronous version of [`RwLockReadGuard::try_map`] from the
    /// [`parking_lot` crate].
    ///
    /// [`RwLockReadGuard::try_map`]: https://docs.rs/lock_api/latest/lock_api/struct.RwLockReadGuard.html#method.try_map
    /// [`parking_lot` crate]: https://crates.io/crates/parking_lot
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::{RwLock, RwLockReadGuard};
    ///
    /// #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    /// struct Foo(u32);
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let lock = RwLock::new(Foo(1));
    ///
    /// let guard = lock.read().await;
    /// let guard = RwLockReadGuard::try_map(guard, |f| Some(&f.0)).expect("should not fail");
    ///
    /// assert_eq!(1, *guard);
    /// # }
    /// ```
    #[inline]
    pub fn try_map<F, U: ?Sized>(this: Self, f: F) -> Result<RwLockReadGuard<'a, U>, Self>
    where
        F: FnOnce(&T) -> Option<&U>,
    {
        let data = match f(&*this) {
            Some(data) => data as *const U,
            None => return Err(this),
        };
        let s = this.s;
        #[cfg(all(tokio_unstable, feature = "tracing"))]
        let resource_span = this.resource_span.clone();
        // NB: Forget to avoid drop impl from being called.
        mem::forget(this);

        Ok(RwLockReadGuard {
            s,
            data,
            marker: marker::PhantomData,
            #[cfg(all(tokio_unstable, feature = "tracing"))]
            resource_span,
        })
    }
}

impl<T: ?Sized> ops::Deref for RwLockReadGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.data }
    }
}

impl<'a, T: ?Sized> fmt::Debug for RwLockReadGuard<'a, T>
where
    T: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized> fmt::Display for RwLockReadGuard<'a, T>
where
    T: fmt::Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&**self, f)
    }
}

impl<'a, T: ?Sized> Drop for RwLockReadGuard<'a, T> {
    fn drop(&mut self) {
        self.s.release(1);

        #[cfg(all(tokio_unstable, feature = "tracing"))]
        self.resource_span.in_scope(|| {
            tracing::trace!(
            target: "runtime::resource::state_update",
            current_readers = 1,
            current_readers.op = "sub",
            )
        });
    }
}
