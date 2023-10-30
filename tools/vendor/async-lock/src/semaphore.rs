use std::future::Future;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use event_listener::Event;

/// A counter for limiting the number of concurrent operations.
#[derive(Debug)]
pub struct Semaphore {
    count: AtomicUsize,
    event: Event,
}

impl Semaphore {
    /// Creates a new semaphore with a limit of `n` concurrent operations.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Semaphore;
    ///
    /// let s = Semaphore::new(5);
    /// ```
    pub const fn new(n: usize) -> Semaphore {
        Semaphore {
            count: AtomicUsize::new(n),
            event: Event::new(),
        }
    }

    /// Attempts to get a permit for a concurrent operation.
    ///
    /// If the permit could not be acquired at this time, then [`None`] is returned. Otherwise, a
    /// guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Semaphore;
    ///
    /// let s = Semaphore::new(2);
    ///
    /// let g1 = s.try_acquire().unwrap();
    /// let g2 = s.try_acquire().unwrap();
    ///
    /// assert!(s.try_acquire().is_none());
    /// drop(g2);
    /// assert!(s.try_acquire().is_some());
    /// ```
    pub fn try_acquire(&self) -> Option<SemaphoreGuard<'_>> {
        let mut count = self.count.load(Ordering::Acquire);
        loop {
            if count == 0 {
                return None;
            }

            match self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(SemaphoreGuard(self)),
                Err(c) => count = c,
            }
        }
    }

    /// Waits for a permit for a concurrent operation.
    ///
    /// Returns a guard that releases the permit when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Semaphore;
    ///
    /// let s = Semaphore::new(2);
    /// let guard = s.acquire().await;
    /// # });
    /// ```
    pub async fn acquire(&self) -> SemaphoreGuard<'_> {
        let mut listener = None;

        loop {
            if let Some(guard) = self.try_acquire() {
                return guard;
            }

            match listener.take() {
                None => listener = Some(self.event.listen()),
                Some(l) => l.await,
            }
        }
    }
}

impl Semaphore {
    /// Attempts to get an owned permit for a concurrent operation.
    ///
    /// If the permit could not be acquired at this time, then [`None`] is returned. Otherwise, an
    /// owned guard is returned that releases the mutex when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// use async_lock::Semaphore;
    /// use std::sync::Arc;
    ///
    /// let s = Arc::new(Semaphore::new(2));
    ///
    /// let g1 = s.try_acquire_arc().unwrap();
    /// let g2 = s.try_acquire_arc().unwrap();
    ///
    /// assert!(s.try_acquire_arc().is_none());
    /// drop(g2);
    /// assert!(s.try_acquire_arc().is_some());
    /// ```
    pub fn try_acquire_arc(self: &Arc<Self>) -> Option<SemaphoreGuardArc> {
        let mut count = self.count.load(Ordering::Acquire);
        loop {
            if count == 0 {
                return None;
            }

            match self.count.compare_exchange_weak(
                count,
                count - 1,
                Ordering::AcqRel,
                Ordering::Acquire,
            ) {
                Ok(_) => return Some(SemaphoreGuardArc(self.clone())),
                Err(c) => count = c,
            }
        }
    }

    async fn acquire_arc_impl(self: Arc<Self>) -> SemaphoreGuardArc {
        let mut listener = None;

        loop {
            if let Some(guard) = self.try_acquire_arc() {
                return guard;
            }

            match listener.take() {
                None => listener = Some(self.event.listen()),
                Some(l) => l.await,
            }
        }
    }

    /// Waits for an owned permit for a concurrent operation.
    ///
    /// Returns a guard that releases the permit when dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # futures_lite::future::block_on(async {
    /// use async_lock::Semaphore;
    /// use std::sync::Arc;
    ///
    /// let s = Arc::new(Semaphore::new(2));
    /// let guard = s.acquire_arc().await;
    /// # });
    /// ```
    pub fn acquire_arc(self: &Arc<Self>) -> impl Future<Output = SemaphoreGuardArc> {
        self.clone().acquire_arc_impl()
    }
}

/// A guard that releases the acquired permit.
#[derive(Debug)]
pub struct SemaphoreGuard<'a>(&'a Semaphore);

impl Drop for SemaphoreGuard<'_> {
    fn drop(&mut self) {
        self.0.count.fetch_add(1, Ordering::AcqRel);
        self.0.event.notify(1);
    }
}

/// An owned guard that releases the acquired permit.
#[derive(Debug)]
pub struct SemaphoreGuardArc(Arc<Semaphore>);

impl Drop for SemaphoreGuardArc {
    fn drop(&mut self) {
        self.0.count.fetch_add(1, Ordering::AcqRel);
        self.0.event.notify(1);
    }
}
