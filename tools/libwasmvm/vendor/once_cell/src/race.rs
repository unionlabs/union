//! Thread-safe, non-blocking, "first one wins" flavor of `OnceCell`.
//!
//! If two threads race to initialize a type from the `race` module, they
//! don't block, execute initialization function together, but only one of
//! them stores the result.
//!
//! This module does not require `std` feature.
//!
//! # Atomic orderings
//!
//! All types in this module use `Acquire` and `Release`
//! [atomic orderings](Ordering) for all their operations. While this is not
//! strictly necessary for types other than `OnceBox`, it is useful for users as
//! it allows them to be certain that after `get` or `get_or_init` returns on
//! one thread, any side-effects caused by the setter thread prior to them
//! calling `set` or `get_or_init` will be made visible to that thread; without
//! it, it's possible for it to appear as if they haven't happened yet from the
//! getter thread's perspective. This is an acceptable tradeoff to make since
//! `Acquire` and `Release` have very little performance overhead on most
//! architectures versus `Relaxed`.

#[cfg(feature = "critical-section")]
use atomic_polyfill as atomic;
#[cfg(not(feature = "critical-section"))]
use core::sync::atomic;

use atomic::{AtomicPtr, AtomicUsize, Ordering};
use core::cell::UnsafeCell;
use core::marker::PhantomData;
use core::num::NonZeroUsize;
use core::ptr;

/// A thread-safe cell which can be written to only once.
#[derive(Default, Debug)]
pub struct OnceNonZeroUsize {
    inner: AtomicUsize,
}

impl OnceNonZeroUsize {
    /// Creates a new empty cell.
    #[inline]
    pub const fn new() -> OnceNonZeroUsize {
        OnceNonZeroUsize { inner: AtomicUsize::new(0) }
    }

    /// Gets the underlying value.
    #[inline]
    pub fn get(&self) -> Option<NonZeroUsize> {
        let val = self.inner.load(Ordering::Acquire);
        NonZeroUsize::new(val)
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(())` if it was
    /// full.
    #[inline]
    pub fn set(&self, value: NonZeroUsize) -> Result<(), ()> {
        let exchange =
            self.inner.compare_exchange(0, value.get(), Ordering::AcqRel, Ordering::Acquire);
        match exchange {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell was
    /// empty.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_init<F>(&self, f: F) -> NonZeroUsize
    where
        F: FnOnce() -> NonZeroUsize,
    {
        enum Void {}
        match self.get_or_try_init(|| Ok::<NonZeroUsize, Void>(f())) {
            Ok(val) => val,
            Err(void) => match void {},
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<NonZeroUsize, E>
    where
        F: FnOnce() -> Result<NonZeroUsize, E>,
    {
        let val = self.inner.load(Ordering::Acquire);
        let res = match NonZeroUsize::new(val) {
            Some(it) => it,
            None => {
                let mut val = f()?.get();
                let exchange =
                    self.inner.compare_exchange(0, val, Ordering::AcqRel, Ordering::Acquire);
                if let Err(old) = exchange {
                    val = old;
                }
                unsafe { NonZeroUsize::new_unchecked(val) }
            }
        };
        Ok(res)
    }
}

/// A thread-safe cell which can be written to only once.
#[derive(Default, Debug)]
pub struct OnceBool {
    inner: OnceNonZeroUsize,
}

impl OnceBool {
    /// Creates a new empty cell.
    #[inline]
    pub const fn new() -> OnceBool {
        OnceBool { inner: OnceNonZeroUsize::new() }
    }

    /// Gets the underlying value.
    #[inline]
    pub fn get(&self) -> Option<bool> {
        self.inner.get().map(OnceBool::from_usize)
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(())` if it was
    /// full.
    #[inline]
    pub fn set(&self, value: bool) -> Result<(), ()> {
        self.inner.set(OnceBool::to_usize(value))
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell was
    /// empty.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_init<F>(&self, f: F) -> bool
    where
        F: FnOnce() -> bool,
    {
        OnceBool::from_usize(self.inner.get_or_init(|| OnceBool::to_usize(f())))
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<bool, E>
    where
        F: FnOnce() -> Result<bool, E>,
    {
        self.inner.get_or_try_init(|| f().map(OnceBool::to_usize)).map(OnceBool::from_usize)
    }

    #[inline]
    fn from_usize(value: NonZeroUsize) -> bool {
        value.get() == 1
    }

    #[inline]
    fn to_usize(value: bool) -> NonZeroUsize {
        unsafe { NonZeroUsize::new_unchecked(if value { 1 } else { 2 }) }
    }
}

/// A thread-safe cell which can be written to only once.
pub struct OnceRef<'a, T> {
    inner: AtomicPtr<T>,
    ghost: PhantomData<UnsafeCell<&'a T>>,
}

// TODO: Replace UnsafeCell with SyncUnsafeCell once stabilized
unsafe impl<'a, T: Sync> Sync for OnceRef<'a, T> {}

impl<'a, T> core::fmt::Debug for OnceRef<'a, T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "OnceRef({:?})", self.inner)
    }
}

impl<'a, T> Default for OnceRef<'a, T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, T> OnceRef<'a, T> {
    /// Creates a new empty cell.
    pub const fn new() -> OnceRef<'a, T> {
        OnceRef { inner: AtomicPtr::new(ptr::null_mut()), ghost: PhantomData }
    }

    /// Gets a reference to the underlying value.
    pub fn get(&self) -> Option<&'a T> {
        let ptr = self.inner.load(Ordering::Acquire);
        unsafe { ptr.as_ref() }
    }

    /// Sets the contents of this cell to `value`.
    ///
    /// Returns `Ok(())` if the cell was empty and `Err(value)` if it was
    /// full.
    pub fn set(&self, value: &'a T) -> Result<(), ()> {
        let ptr = value as *const T as *mut T;
        let exchange =
            self.inner.compare_exchange(ptr::null_mut(), ptr, Ordering::AcqRel, Ordering::Acquire);
        match exchange {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if the cell was
    /// empty.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_init<F>(&self, f: F) -> &'a T
    where
        F: FnOnce() -> &'a T,
    {
        enum Void {}
        match self.get_or_try_init(|| Ok::<&'a T, Void>(f())) {
            Ok(val) => val,
            Err(void) => match void {},
        }
    }

    /// Gets the contents of the cell, initializing it with `f` if
    /// the cell was empty. If the cell was empty and `f` failed, an
    /// error is returned.
    ///
    /// If several threads concurrently run `get_or_init`, more than one `f` can
    /// be called. However, all threads will return the same value, produced by
    /// some `f`.
    pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&'a T, E>
    where
        F: FnOnce() -> Result<&'a T, E>,
    {
        let mut ptr = self.inner.load(Ordering::Acquire);

        if ptr.is_null() {
            // TODO replace with `cast_mut` when MSRV reaches 1.65.0 (also in `set`)
            ptr = f()? as *const T as *mut T;
            let exchange = self.inner.compare_exchange(
                ptr::null_mut(),
                ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            if let Err(old) = exchange {
                ptr = old;
            }
        }

        Ok(unsafe { &*ptr })
    }

    /// ```compile_fail
    /// use once_cell::race::OnceRef;
    ///
    /// let mut l = OnceRef::new();
    ///
    /// {
    ///     let y = 2;
    ///     let mut r = OnceRef::new();
    ///     r.set(&y).unwrap();
    ///     core::mem::swap(&mut l, &mut r);
    /// }
    ///
    /// // l now contains a dangling reference to y
    /// eprintln!("uaf: {}", l.get().unwrap());
    /// ```
    fn _dummy() {}
}

#[cfg(feature = "alloc")]
pub use self::once_box::OnceBox;

#[cfg(feature = "alloc")]
mod once_box {
    use super::atomic::{AtomicPtr, Ordering};
    use core::{marker::PhantomData, ptr};

    use alloc::boxed::Box;

    /// A thread-safe cell which can be written to only once.
    pub struct OnceBox<T> {
        inner: AtomicPtr<T>,
        ghost: PhantomData<Option<Box<T>>>,
    }

    impl<T> core::fmt::Debug for OnceBox<T> {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "OnceBox({:?})", self.inner.load(Ordering::Relaxed))
        }
    }

    impl<T> Default for OnceBox<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> Drop for OnceBox<T> {
        fn drop(&mut self) {
            let ptr = *self.inner.get_mut();
            if !ptr.is_null() {
                drop(unsafe { Box::from_raw(ptr) })
            }
        }
    }

    impl<T> OnceBox<T> {
        /// Creates a new empty cell.
        pub const fn new() -> OnceBox<T> {
            OnceBox { inner: AtomicPtr::new(ptr::null_mut()), ghost: PhantomData }
        }

        /// Gets a reference to the underlying value.
        pub fn get(&self) -> Option<&T> {
            let ptr = self.inner.load(Ordering::Acquire);
            if ptr.is_null() {
                return None;
            }
            Some(unsafe { &*ptr })
        }

        /// Sets the contents of this cell to `value`.
        ///
        /// Returns `Ok(())` if the cell was empty and `Err(value)` if it was
        /// full.
        pub fn set(&self, value: Box<T>) -> Result<(), Box<T>> {
            let ptr = Box::into_raw(value);
            let exchange = self.inner.compare_exchange(
                ptr::null_mut(),
                ptr,
                Ordering::AcqRel,
                Ordering::Acquire,
            );
            if exchange.is_err() {
                let value = unsafe { Box::from_raw(ptr) };
                return Err(value);
            }
            Ok(())
        }

        /// Gets the contents of the cell, initializing it with `f` if the cell was
        /// empty.
        ///
        /// If several threads concurrently run `get_or_init`, more than one `f` can
        /// be called. However, all threads will return the same value, produced by
        /// some `f`.
        pub fn get_or_init<F>(&self, f: F) -> &T
        where
            F: FnOnce() -> Box<T>,
        {
            enum Void {}
            match self.get_or_try_init(|| Ok::<Box<T>, Void>(f())) {
                Ok(val) => val,
                Err(void) => match void {},
            }
        }

        /// Gets the contents of the cell, initializing it with `f` if
        /// the cell was empty. If the cell was empty and `f` failed, an
        /// error is returned.
        ///
        /// If several threads concurrently run `get_or_init`, more than one `f` can
        /// be called. However, all threads will return the same value, produced by
        /// some `f`.
        pub fn get_or_try_init<F, E>(&self, f: F) -> Result<&T, E>
        where
            F: FnOnce() -> Result<Box<T>, E>,
        {
            let mut ptr = self.inner.load(Ordering::Acquire);

            if ptr.is_null() {
                let val = f()?;
                ptr = Box::into_raw(val);
                let exchange = self.inner.compare_exchange(
                    ptr::null_mut(),
                    ptr,
                    Ordering::AcqRel,
                    Ordering::Acquire,
                );
                if let Err(old) = exchange {
                    drop(unsafe { Box::from_raw(ptr) });
                    ptr = old;
                }
            };
            Ok(unsafe { &*ptr })
        }
    }

    unsafe impl<T: Sync + Send> Sync for OnceBox<T> {}

    /// ```compile_fail
    /// struct S(*mut ());
    /// unsafe impl Sync for S {}
    ///
    /// fn share<T: Sync>(_: &T) {}
    /// share(&once_cell::race::OnceBox::<S>::new());
    /// ```
    fn _dummy() {}
}
