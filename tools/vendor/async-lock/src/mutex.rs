use std::cell::UnsafeCell;
use std::fmt;
use std::future::Future;
use std::ops::{Deref, DerefMut};
use std::process;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Note: we cannot use `target_family = "wasm"` here because it requires Rust 1.54.
#[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
use std::time::{Duration, Instant};

use std::usize;

use event_listener::Event;

/// An async mutex.
///
/// The locking mechanism uses eventual fairness to ensure locking will be fair on average without
/// sacrificing performance. This is done by forcing a fair lock whenever a lock operation is
/// starved for longer than 0.5 milliseconds.
///
/// # Examples
///
/// ```
/// # futures_lite::future::block_on(async {
/// use async_lock::Mutex;
///
/// let m = Mutex::new(1);
///
/// let mut guard = m.lock().await;
/// *guard = 2;
///
/// assert!(m.try_lock().is_none());
/// drop(guard);
/// assert_eq!(*m.try_lock().unwrap(), 2);
/// # })
/// ```
pub struct Mutex<T: ?Sized> {
    /// Current state of the mutex.
    ///
    /// The least significant bit is set to 1 if the mutex is locked.
    /// The other bits hold the number of starved lock operations.
    state: AtomicUsize,

    /// Lock operations waiting for the mutex to be released.
    lock_ops: Event,

    /// The value inside the mutex.
    data: UnsafeCell<T>,
}

unsafe impl<T: Send + ?Sized> Send for Mutex<T> {}
unsafe impl<T: Send + ?Sized> Sync for Mutex<T> {}

impl<T> Mutex<T> {
    /// Creates a new async mutex.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// ```
    pub const fn new(data: T) -> Mutex<T> {
        Mutex {
            state: AtomicUsize::new(0),
            lock_ops: Event::new(),
            data: UnsafeCell::new(data),
        }
    }

    /// Consumes the mutex, returning the underlying data.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// assert_eq!(mutex.into_inner(), 10);
    /// ```
    pub fn into_inner(self) -> T {
        self.data.into_inner()
    }
}

impl<T: ?Sized> Mutex<T> {
    /// Acquires the mutex.
    ///
    /// Returns a guard that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// let guard = mutex.lock().await;
    /// assert_eq!(*guard, 10);
    /// # })
    /// ```
    #[inline]
    pub async fn lock(&self) -> MutexGuard<'_, T> {
        if let Some(guard) = self.try_lock() {
            return guard;
        }
        self.acquire_slow().await;
        MutexGuard(self)
    }

    /// Slow path for acquiring the mutex.
    #[cold]
    async fn acquire_slow(&self) {
        // Get the current time.
        #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
        let start = Instant::now();

        loop {
            // Start listening for events.
            let listener = self.lock_ops.listen();

            // Try locking if nobody is being starved.
            match self
                .state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
                .unwrap_or_else(|x| x)
            {
                // Lock acquired!
                0 => return,

                // Lock is held and nobody is starved.
                1 => {}

                // Somebody is starved.
                _ => break,
            }

            // Wait for a notification.
            listener.await;

            // Try locking if nobody is being starved.
            match self
                .state
                .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
                .unwrap_or_else(|x| x)
            {
                // Lock acquired!
                0 => return,

                // Lock is held and nobody is starved.
                1 => {}

                // Somebody is starved.
                _ => {
                    // Notify the first listener in line because we probably received a
                    // notification that was meant for a starved task.
                    self.lock_ops.notify(1);
                    break;
                }
            }

            // If waiting for too long, fall back to a fairer locking strategy that will prevent
            // newer lock operations from starving us forever.
            #[cfg(not(any(target_arch = "wasm32", target_arch = "wasm64")))]
            if start.elapsed() > Duration::from_micros(500) {
                break;
            }
        }

        // Increment the number of starved lock operations.
        if self.state.fetch_add(2, Ordering::Release) > usize::MAX / 2 {
            // In case of potential overflow, abort.
            process::abort();
        }

        // Decrement the counter when exiting this function.
        let _call = CallOnDrop(|| {
            self.state.fetch_sub(2, Ordering::Release);
        });

        loop {
            // Start listening for events.
            let listener = self.lock_ops.listen();

            // Try locking if nobody else is being starved.
            match self
                .state
                .compare_exchange(2, 2 | 1, Ordering::Acquire, Ordering::Acquire)
                .unwrap_or_else(|x| x)
            {
                // Lock acquired!
                2 => return,

                // Lock is held by someone.
                s if s % 2 == 1 => {}

                // Lock is available.
                _ => {
                    // Be fair: notify the first listener and then go wait in line.
                    self.lock_ops.notify(1);
                }
            }

            // Wait for a notification.
            listener.await;

            // Try acquiring the lock without waiting for others.
            if self.state.fetch_or(1, Ordering::Acquire) % 2 == 0 {
                return;
            }
        }
    }

    /// Attempts to acquire the mutex.
    ///
    /// If the mutex could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Mutex;
    ///
    /// let mutex = Mutex::new(10);
    /// if let Some(guard) = mutex.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        if self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            Some(MutexGuard(self))
        } else {
            None
        }
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// Since this call borrows the mutex mutably, no actual locking takes place -- the mutable
    /// borrow statically guarantees the mutex is not already acquired.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Mutex;
    ///
    /// let mut mutex = Mutex::new(0);
    /// *mutex.get_mut() = 10;
    /// assert_eq!(*mutex.lock().await, 10);
    /// # })
    /// ```
    pub fn get_mut(&mut self) -> &mut T {
        unsafe { &mut *self.data.get() }
    }
}

impl<T: ?Sized> Mutex<T> {
    async fn lock_arc_impl(self: Arc<Self>) -> MutexGuardArc<T> {
        if let Some(guard) = self.try_lock_arc() {
            return guard;
        }
        self.acquire_slow().await;
        MutexGuardArc(self)
    }

    /// Acquires the mutex and clones a reference to it.
    ///
    /// Returns an owned guard that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Mutex;
    /// use std::sync::Arc;
    ///
    /// let mutex = Arc::new(Mutex::new(10));
    /// let guard = mutex.lock_arc().await;
    /// assert_eq!(*guard, 10);
    /// # })
    /// ```
    #[inline]
    pub fn lock_arc(self: &Arc<Self>) -> impl Future<Output = MutexGuardArc<T>> {
        self.clone().lock_arc_impl()
    }

    /// Attempts to acquire the mutex and clone a reference to it.
    ///
    /// If the mutex could not be acquired at this time, then [`None`] is returned. Otherwise, an
    /// owned guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Mutex;
    /// use std::sync::Arc;
    ///
    /// let mutex = Arc::new(Mutex::new(10));
    /// if let Some(guard) = mutex.try_lock() {
    ///     assert_eq!(*guard, 10);
    /// }
    /// # ;
    /// ```
    #[inline]
    pub fn try_lock_arc(self: &Arc<Self>) -> Option<MutexGuardArc<T>> {
        if self
            .state
            .compare_exchange(0, 1, Ordering::Acquire, Ordering::Acquire)
            .is_ok()
        {
            Some(MutexGuardArc(self.clone()))
        } else {
            None
        }
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for Mutex<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct Locked;
        impl fmt::Debug for Locked {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("<locked>")
            }
        }

        match self.try_lock() {
            None => f.debug_struct("Mutex").field("data", &Locked).finish(),
            Some(guard) => f.debug_struct("Mutex").field("data", &&*guard).finish(),
        }
    }
}

impl<T> From<T> for Mutex<T> {
    fn from(val: T) -> Mutex<T> {
        Mutex::new(val)
    }
}

impl<T: Default + ?Sized> Default for Mutex<T> {
    fn default() -> Mutex<T> {
        Mutex::new(Default::default())
    }
}

/// A guard that releases the mutex when dropped.
pub struct MutexGuard<'a, T: ?Sized>(&'a Mutex<T>);

unsafe impl<T: Send + ?Sized> Send for MutexGuard<'_, T> {}
unsafe impl<T: Sync + ?Sized> Sync for MutexGuard<'_, T> {}

impl<'a, T: ?Sized> MutexGuard<'a, T> {
    /// Returns a reference to the mutex a guard came from.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::{Mutex, MutexGuard};
    ///
    /// let mutex = Mutex::new(10i32);
    /// let guard = mutex.lock().await;
    /// dbg!(MutexGuard::source(&guard));
    /// # })
    /// ```
    pub fn source(guard: &MutexGuard<'a, T>) -> &'a Mutex<T> {
        guard.0
    }
}

impl<T: ?Sized> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        // Remove the last bit and notify a waiting lock operation.
        self.0.state.fetch_sub(1, Ordering::Release);
        self.0.lock_ops.notify(1);
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for MutexGuard<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}

/// An owned guard that releases the mutex when dropped.
pub struct MutexGuardArc<T: ?Sized>(Arc<Mutex<T>>);

unsafe impl<T: Send + ?Sized> Send for MutexGuardArc<T> {}
unsafe impl<T: Sync + ?Sized> Sync for MutexGuardArc<T> {}

impl<T: ?Sized> MutexGuardArc<T> {
    /// Returns a reference to the mutex a guard came from.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::{Mutex, MutexGuardArc};
    /// use std::sync::Arc;
    ///
    /// let mutex = Arc::new(Mutex::new(10i32));
    /// let guard = mutex.lock_arc().await;
    /// dbg!(MutexGuardArc::source(&guard));
    /// # })
    /// ```
    pub fn source(guard: &MutexGuardArc<T>) -> &Arc<Mutex<T>> {
        &guard.0
    }
}

impl<T: ?Sized> Drop for MutexGuardArc<T> {
    fn drop(&mut self) {
        // Remove the last bit and notify a waiting lock operation.
        self.0.state.fetch_sub(1, Ordering::Release);
        self.0.lock_ops.notify(1);
    }
}

impl<T: fmt::Debug + ?Sized> fmt::Debug for MutexGuardArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&**self, f)
    }
}

impl<T: fmt::Display + ?Sized> fmt::Display for MutexGuardArc<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        (**self).fmt(f)
    }
}

impl<T: ?Sized> Deref for MutexGuardArc<T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.0.data.get() }
    }
}

impl<T: ?Sized> DerefMut for MutexGuardArc<T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.0.data.get() }
    }
}

/// Calls a function when dropped.
struct CallOnDrop<F: Fn()>(F);

impl<F: Fn()> Drop for CallOnDrop<F> {
    fn drop(&mut self) {
        (self.0)();
    }
}
