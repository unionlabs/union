//! Parks the runtime.
//!
//! A combination of the various resource driver park handles.

use crate::loom::sync::atomic::AtomicUsize;
use crate::loom::sync::{Arc, Condvar, Mutex};
use crate::loom::thread;
use crate::park::{Park, Unpark};
use crate::runtime::driver::Driver;
use crate::util::TryLock;

use std::sync::atomic::Ordering::SeqCst;
use std::time::Duration;

pub(crate) struct Parker {
    inner: Arc<Inner>,
}

pub(crate) struct Unparker {
    inner: Arc<Inner>,
}

struct Inner {
    /// Avoids entering the park if possible
    state: AtomicUsize,

    /// Used to coordinate access to the driver / condvar
    mutex: Mutex<()>,

    /// Condvar to block on if the driver is unavailable.
    condvar: Condvar,

    /// Resource (I/O, time, ...) driver
    shared: Arc<Shared>,
}

const EMPTY: usize = 0;
const PARKED_CONDVAR: usize = 1;
const PARKED_DRIVER: usize = 2;
const NOTIFIED: usize = 3;

/// Shared across multiple Parker handles
struct Shared {
    /// Shared driver. Only one thread at a time can use this
    driver: TryLock<Driver>,

    /// Unpark handle
    handle: <Driver as Park>::Unpark,
}

impl Parker {
    pub(crate) fn new(driver: Driver) -> Parker {
        let handle = driver.unpark();

        Parker {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(EMPTY),
                mutex: Mutex::new(()),
                condvar: Condvar::new(),
                shared: Arc::new(Shared {
                    driver: TryLock::new(driver),
                    handle,
                }),
            }),
        }
    }
}

impl Clone for Parker {
    fn clone(&self) -> Parker {
        Parker {
            inner: Arc::new(Inner {
                state: AtomicUsize::new(EMPTY),
                mutex: Mutex::new(()),
                condvar: Condvar::new(),
                shared: self.inner.shared.clone(),
            }),
        }
    }
}

impl Park for Parker {
    type Unpark = Unparker;
    type Error = ();

    fn unpark(&self) -> Unparker {
        Unparker {
            inner: self.inner.clone(),
        }
    }

    fn park(&mut self) -> Result<(), Self::Error> {
        self.inner.park();
        Ok(())
    }

    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error> {
        // Only parking with zero is supported...
        assert_eq!(duration, Duration::from_millis(0));

        if let Some(mut driver) = self.inner.shared.driver.try_lock() {
            driver.park_timeout(duration).map_err(|_| ())
        } else {
            Ok(())
        }
    }

    fn shutdown(&mut self) {
        self.inner.shutdown();
    }
}

impl Unpark for Unparker {
    fn unpark(&self) {
        self.inner.unpark();
    }
}

impl Inner {
    /// Parks the current thread for at most `dur`.
    fn park(&self) {
        for _ in 0..3 {
            // If we were previously notified then we consume this notification and
            // return quickly.
            if self
                .state
                .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
                .is_ok()
            {
                return;
            }

            thread::yield_now();
        }

        if let Some(mut driver) = self.shared.driver.try_lock() {
            self.park_driver(&mut driver);
        } else {
            self.park_condvar();
        }
    }

    fn park_condvar(&self) {
        // Otherwise we need to coordinate going to sleep
        let mut m = self.mutex.lock();

        match self
            .state
            .compare_exchange(EMPTY, PARKED_CONDVAR, SeqCst, SeqCst)
        {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read here, even though we know it will be `NOTIFIED`.
                // This is because `unpark` may have been called again since we read
                // `NOTIFIED` in the `compare_exchange` above. We must perform an
                // acquire operation that synchronizes with that `unpark` to observe
                // any writes it made before the call to unpark. To do that we must
                // read from the write it made to `state`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        loop {
            m = self.condvar.wait(m).unwrap();

            if self
                .state
                .compare_exchange(NOTIFIED, EMPTY, SeqCst, SeqCst)
                .is_ok()
            {
                // got a notification
                return;
            }

            // spurious wakeup, go back to sleep
        }
    }

    fn park_driver(&self, driver: &mut Driver) {
        match self
            .state
            .compare_exchange(EMPTY, PARKED_DRIVER, SeqCst, SeqCst)
        {
            Ok(_) => {}
            Err(NOTIFIED) => {
                // We must read here, even though we know it will be `NOTIFIED`.
                // This is because `unpark` may have been called again since we read
                // `NOTIFIED` in the `compare_exchange` above. We must perform an
                // acquire operation that synchronizes with that `unpark` to observe
                // any writes it made before the call to unpark. To do that we must
                // read from the write it made to `state`.
                let old = self.state.swap(EMPTY, SeqCst);
                debug_assert_eq!(old, NOTIFIED, "park state changed unexpectedly");

                return;
            }
            Err(actual) => panic!("inconsistent park state; actual = {}", actual),
        }

        // TODO: don't unwrap
        driver.park().unwrap();

        match self.state.swap(EMPTY, SeqCst) {
            NOTIFIED => {}      // got a notification, hurray!
            PARKED_DRIVER => {} // no notification, alas
            n => panic!("inconsistent park_timeout state: {}", n),
        }
    }

    fn unpark(&self) {
        // To ensure the unparked thread will observe any writes we made before
        // this call, we must perform a release operation that `park` can
        // synchronize with. To do that we must write `NOTIFIED` even if `state`
        // is already `NOTIFIED`. That is why this must be a swap rather than a
        // compare-and-swap that returns if it reads `NOTIFIED` on failure.
        match self.state.swap(NOTIFIED, SeqCst) {
            EMPTY => {}    // no one was waiting
            NOTIFIED => {} // already unparked
            PARKED_CONDVAR => self.unpark_condvar(),
            PARKED_DRIVER => self.unpark_driver(),
            actual => panic!("inconsistent state in unpark; actual = {}", actual),
        }
    }

    fn unpark_condvar(&self) {
        // There is a period between when the parked thread sets `state` to
        // `PARKED` (or last checked `state` in the case of a spurious wake
        // up) and when it actually waits on `cvar`. If we were to notify
        // during this period it would be ignored and then when the parked
        // thread went to sleep it would never wake up. Fortunately, it has
        // `lock` locked at this stage so we can acquire `lock` to wait until
        // it is ready to receive the notification.
        //
        // Releasing `lock` before the call to `notify_one` means that when the
        // parked thread wakes it doesn't get woken only to have to wait for us
        // to release `lock`.
        drop(self.mutex.lock());

        self.condvar.notify_one()
    }

    fn unpark_driver(&self) {
        self.shared.handle.unpark();
    }

    fn shutdown(&self) {
        if let Some(mut driver) = self.shared.driver.try_lock() {
            driver.shutdown();
        }

        self.condvar.notify_all();
    }
}
