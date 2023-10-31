#![cfg_attr(not(feature = "sync"), allow(dead_code, unreachable_pub))]

//! A single-producer, multi-consumer channel that only retains the *last* sent
//! value.
//!
//! This channel is useful for watching for changes to a value from multiple
//! points in the code base, for example, changes to configuration values.
//!
//! # Usage
//!
//! [`channel`] returns a [`Sender`] / [`Receiver`] pair. These are the producer
//! and sender halves of the channel. The channel is created with an initial
//! value. The **latest** value stored in the channel is accessed with
//! [`Receiver::borrow()`]. Awaiting [`Receiver::changed()`] waits for a new
//! value to sent by the [`Sender`] half.
//!
//! # Examples
//!
//! ```
//! use tokio::sync::watch;
//!
//! # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
//!     let (tx, mut rx) = watch::channel("hello");
//!
//!     tokio::spawn(async move {
//!         while rx.changed().await.is_ok() {
//!             println!("received = {:?}", *rx.borrow());
//!         }
//!     });
//!
//!     tx.send("world")?;
//! # Ok(())
//! # }
//! ```
//!
//! # Closing
//!
//! [`Sender::is_closed`] and [`Sender::closed`] allow the producer to detect
//! when all [`Receiver`] handles have been dropped. This indicates that there
//! is no further interest in the values being produced and work can be stopped.
//!
//! # Thread safety
//!
//! Both [`Sender`] and [`Receiver`] are thread safe. They can be moved to other
//! threads and can be used in a concurrent environment. Clones of [`Receiver`]
//! handles may be moved to separate threads and also used concurrently.
//!
//! [`Sender`]: crate::sync::watch::Sender
//! [`Receiver`]: crate::sync::watch::Receiver
//! [`Receiver::changed()`]: crate::sync::watch::Receiver::changed
//! [`Receiver::borrow()`]: crate::sync::watch::Receiver::borrow
//! [`channel`]: crate::sync::watch::channel
//! [`Sender::is_closed`]: crate::sync::watch::Sender::is_closed
//! [`Sender::closed`]: crate::sync::watch::Sender::closed

use crate::sync::notify::Notify;

use crate::loom::sync::atomic::AtomicUsize;
use crate::loom::sync::atomic::Ordering::Relaxed;
use crate::loom::sync::{Arc, RwLock, RwLockReadGuard};
use std::mem;
use std::ops;
use std::panic;

/// Receives values from the associated [`Sender`](struct@Sender).
///
/// Instances are created by the [`channel`](fn@channel) function.
///
/// To turn this receiver into a `Stream`, you can use the [`WatchStream`]
/// wrapper.
///
/// [`WatchStream`]: https://docs.rs/tokio-stream/0.1/tokio_stream/wrappers/struct.WatchStream.html
#[derive(Debug)]
pub struct Receiver<T> {
    /// Pointer to the shared state
    shared: Arc<Shared<T>>,

    /// Last observed version
    version: Version,
}

/// Sends values to the associated [`Receiver`](struct@Receiver).
///
/// Instances are created by the [`channel`](fn@channel) function.
#[derive(Debug)]
pub struct Sender<T> {
    shared: Arc<Shared<T>>,
}

/// Returns a reference to the inner value.
///
/// Outstanding borrows hold a read lock on the inner value. This means that
/// long lived borrows could cause the produce half to block. It is recommended
/// to keep the borrow as short lived as possible. Additionally, if you are
/// running in an environment that allows `!Send` futures, you must ensure that
/// the returned `Ref` type is never held alive across an `.await` point.
///
/// The priority policy of the lock is dependent on the underlying lock
/// implementation, and this type does not guarantee that any particular policy
/// will be used. In particular, a producer which is waiting to acquire the lock
/// in `send` might or might not block concurrent calls to `borrow`, e.g.:
///
/// <details><summary>Potential deadlock example</summary>
///
/// ```text
/// // Task 1 (on thread A)    |  // Task 2 (on thread B)
/// let _ref1 = rx.borrow();   |
///                            |  // will block
///                            |  let _ = tx.send(());
/// // may deadlock            |
/// let _ref2 = rx.borrow();   |
/// ```
/// </details>
#[derive(Debug)]
pub struct Ref<'a, T> {
    inner: RwLockReadGuard<'a, T>,
    has_changed: bool,
}

impl<'a, T> Ref<'a, T> {
    /// Indicates if the borrowed value is considered as _changed_ since the last
    /// time it has been marked as seen.
    ///
    /// Unlike [`Receiver::has_changed()`], this method does not fail if the channel is closed.
    ///
    /// When borrowed from the [`Sender`] this function will always return `false`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = watch::channel("hello");
    ///
    ///     tx.send("goodbye").unwrap();
    ///     // The sender does never consider the value as changed.
    ///     assert!(!tx.borrow().has_changed());
    ///
    ///     // Drop the sender immediately, just for testing purposes.
    ///     drop(tx);
    ///
    ///     // Even if the sender has already been dropped...
    ///     assert!(rx.has_changed().is_err());
    ///     // ...the modified value is still readable and detected as changed.
    ///     assert_eq!(*rx.borrow(), "goodbye");
    ///     assert!(rx.borrow().has_changed());
    ///
    ///     // Read the changed value and mark it as seen.
    ///     {
    ///         let received = rx.borrow_and_update();
    ///         assert_eq!(*received, "goodbye");
    ///         assert!(received.has_changed());
    ///         // Release the read lock when leaving this scope.
    ///     }
    ///
    ///     // Now the value has already been marked as seen and could
    ///     // never be modified again (after the sender has been dropped).
    ///     assert!(!rx.borrow().has_changed());
    /// }
    /// ```
    pub fn has_changed(&self) -> bool {
        self.has_changed
    }
}

#[derive(Debug)]
struct Shared<T> {
    /// The most recent value.
    value: RwLock<T>,

    /// The current version.
    ///
    /// The lowest bit represents a "closed" state. The rest of the bits
    /// represent the current version.
    state: AtomicState,

    /// Tracks the number of `Receiver` instances.
    ref_count_rx: AtomicUsize,

    /// Notifies waiting receivers that the value changed.
    notify_rx: Notify,

    /// Notifies any task listening for `Receiver` dropped events.
    notify_tx: Notify,
}

pub mod error {
    //! Watch error types.

    use std::fmt;

    /// Error produced when sending a value fails.
    #[derive(Debug)]
    pub struct SendError<T>(pub T);

    // ===== impl SendError =====

    impl<T: fmt::Debug> fmt::Display for SendError<T> {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "channel closed")
        }
    }

    impl<T: fmt::Debug> std::error::Error for SendError<T> {}

    /// Error produced when receiving a change notification.
    #[derive(Debug, Clone)]
    pub struct RecvError(pub(super) ());

    // ===== impl RecvError =====

    impl fmt::Display for RecvError {
        fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(fmt, "channel closed")
        }
    }

    impl std::error::Error for RecvError {}
}

use self::state::{AtomicState, Version};
mod state {
    use crate::loom::sync::atomic::AtomicUsize;
    use crate::loom::sync::atomic::Ordering::SeqCst;

    const CLOSED: usize = 1;

    /// The version part of the state. The lowest bit is always zero.
    #[derive(Copy, Clone, Debug, Eq, PartialEq)]
    pub(super) struct Version(usize);

    /// Snapshot of the state. The first bit is used as the CLOSED bit.
    /// The remaining bits are used as the version.
    ///
    /// The CLOSED bit tracks whether the Sender has been dropped. Dropping all
    /// receivers does not set it.
    #[derive(Copy, Clone, Debug)]
    pub(super) struct StateSnapshot(usize);

    /// The state stored in an atomic integer.
    #[derive(Debug)]
    pub(super) struct AtomicState(AtomicUsize);

    impl Version {
        /// Get the initial version when creating the channel.
        pub(super) fn initial() -> Self {
            Version(0)
        }
    }

    impl StateSnapshot {
        /// Extract the version from the state.
        pub(super) fn version(self) -> Version {
            Version(self.0 & !CLOSED)
        }

        /// Is the closed bit set?
        pub(super) fn is_closed(self) -> bool {
            (self.0 & CLOSED) == CLOSED
        }
    }

    impl AtomicState {
        /// Create a new `AtomicState` that is not closed and which has the
        /// version set to `Version::initial()`.
        pub(super) fn new() -> Self {
            AtomicState(AtomicUsize::new(0))
        }

        /// Load the current value of the state.
        pub(super) fn load(&self) -> StateSnapshot {
            StateSnapshot(self.0.load(SeqCst))
        }

        /// Increment the version counter.
        pub(super) fn increment_version(&self) {
            // Increment by two to avoid touching the CLOSED bit.
            self.0.fetch_add(2, SeqCst);
        }

        /// Set the closed bit in the state.
        pub(super) fn set_closed(&self) {
            self.0.fetch_or(CLOSED, SeqCst);
        }
    }
}

/// Creates a new watch channel, returning the "send" and "receive" handles.
///
/// All values sent by [`Sender`] will become visible to the [`Receiver`] handles.
/// Only the last value sent is made available to the [`Receiver`] half. All
/// intermediate values are dropped.
///
/// # Examples
///
/// ```
/// use tokio::sync::watch;
///
/// # async fn dox() -> Result<(), Box<dyn std::error::Error>> {
///     let (tx, mut rx) = watch::channel("hello");
///
///     tokio::spawn(async move {
///         while rx.changed().await.is_ok() {
///             println!("received = {:?}", *rx.borrow());
///         }
///     });
///
///     tx.send("world")?;
/// # Ok(())
/// # }
/// ```
///
/// [`Sender`]: struct@Sender
/// [`Receiver`]: struct@Receiver
pub fn channel<T>(init: T) -> (Sender<T>, Receiver<T>) {
    let shared = Arc::new(Shared {
        value: RwLock::new(init),
        state: AtomicState::new(),
        ref_count_rx: AtomicUsize::new(1),
        notify_rx: Notify::new(),
        notify_tx: Notify::new(),
    });

    let tx = Sender {
        shared: shared.clone(),
    };

    let rx = Receiver {
        shared,
        version: Version::initial(),
    };

    (tx, rx)
}

impl<T> Receiver<T> {
    fn from_shared(version: Version, shared: Arc<Shared<T>>) -> Self {
        // No synchronization necessary as this is only used as a counter and
        // not memory access.
        shared.ref_count_rx.fetch_add(1, Relaxed);

        Self { shared, version }
    }

    /// Returns a reference to the most recently sent value.
    ///
    /// This method does not mark the returned value as seen, so future calls to
    /// [`changed`] may return immediately even if you have already seen the
    /// value with a call to `borrow`.
    ///
    /// Outstanding borrows hold a read lock. This means that long lived borrows
    /// could cause the send half to block. It is recommended to keep the borrow
    /// as short lived as possible. Additionally, if you are running in an
    /// environment that allows `!Send` futures, you must ensure that the
    /// returned `Ref` type is never held alive across an `.await` point.
    ///
    /// The priority policy of the lock is dependent on the underlying lock
    /// implementation, and this type does not guarantee that any particular policy
    /// will be used. In particular, a producer which is waiting to acquire the lock
    /// in `send` might or might not block concurrent calls to `borrow`, e.g.:
    ///
    /// <details><summary>Potential deadlock example</summary>
    ///
    /// ```text
    /// // Task 1 (on thread A)    |  // Task 2 (on thread B)
    /// let _ref1 = rx.borrow();   |
    ///                            |  // will block
    ///                            |  let _ = tx.send(());
    /// // may deadlock            |
    /// let _ref2 = rx.borrow();   |
    /// ```
    /// </details>
    ///
    /// [`changed`]: Receiver::changed
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// let (_, rx) = watch::channel("hello");
    /// assert_eq!(*rx.borrow(), "hello");
    /// ```
    pub fn borrow(&self) -> Ref<'_, T> {
        let inner = self.shared.value.read().unwrap();

        // After obtaining a read-lock no concurrent writes could occur
        // and the loaded version matches that of the borrowed reference.
        let new_version = self.shared.state.load().version();
        let has_changed = self.version != new_version;

        Ref { inner, has_changed }
    }

    /// Returns a reference to the most recently sent value and marks that value
    /// as seen.
    ///
    /// This method marks the current value as seen. Subsequent calls to [`changed`]
    /// will not return immediately until the [`Sender`] has modified the shared
    /// value again.
    ///
    /// Outstanding borrows hold a read lock. This means that long lived borrows
    /// could cause the send half to block. It is recommended to keep the borrow
    /// as short lived as possible. Additionally, if you are running in an
    /// environment that allows `!Send` futures, you must ensure that the
    /// returned `Ref` type is never held alive across an `.await` point.
    ///
    /// The priority policy of the lock is dependent on the underlying lock
    /// implementation, and this type does not guarantee that any particular policy
    /// will be used. In particular, a producer which is waiting to acquire the lock
    /// in `send` might or might not block concurrent calls to `borrow`, e.g.:
    ///
    /// <details><summary>Potential deadlock example</summary>
    ///
    /// ```text
    /// // Task 1 (on thread A)                |  // Task 2 (on thread B)
    /// let _ref1 = rx1.borrow_and_update();   |
    ///                                        |  // will block
    ///                                        |  let _ = tx.send(());
    /// // may deadlock                        |
    /// let _ref2 = rx2.borrow_and_update();   |
    /// ```
    /// </details>
    ///
    /// [`changed`]: Receiver::changed
    pub fn borrow_and_update(&mut self) -> Ref<'_, T> {
        let inner = self.shared.value.read().unwrap();

        // After obtaining a read-lock no concurrent writes could occur
        // and the loaded version matches that of the borrowed reference.
        let new_version = self.shared.state.load().version();
        let has_changed = self.version != new_version;

        // Mark the shared value as seen by updating the version
        self.version = new_version;

        Ref { inner, has_changed }
    }

    /// Checks if this channel contains a message that this receiver has not yet
    /// seen. The new value is not marked as seen.
    ///
    /// Although this method is called `has_changed`, it does not check new
    /// messages for equality, so this call will return true even if the new
    /// message is equal to the old message.
    ///
    /// Returns an error if the channel has been closed.
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = watch::channel("hello");
    ///
    ///     tx.send("goodbye").unwrap();
    ///
    ///     assert!(rx.has_changed().unwrap());
    ///     assert_eq!(*rx.borrow_and_update(), "goodbye");
    ///
    ///     // The value has been marked as seen
    ///     assert!(!rx.has_changed().unwrap());
    ///
    ///     drop(tx);
    ///     // The `tx` handle has been dropped
    ///     assert!(rx.has_changed().is_err());
    /// }
    /// ```
    pub fn has_changed(&self) -> Result<bool, error::RecvError> {
        // Load the version from the state
        let state = self.shared.state.load();
        if state.is_closed() {
            // The sender has dropped.
            return Err(error::RecvError(()));
        }
        let new_version = state.version();

        Ok(self.version != new_version)
    }

    /// Waits for a change notification, then marks the newest value as seen.
    ///
    /// If the newest value in the channel has not yet been marked seen when
    /// this method is called, the method marks that value seen and returns
    /// immediately. If the newest value has already been marked seen, then the
    /// method sleeps until a new message is sent by the [`Sender`] connected to
    /// this `Receiver`, or until the [`Sender`] is dropped.
    ///
    /// This method returns an error if and only if the [`Sender`] is dropped.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe. If you use it as the event in a
    /// [`tokio::select!`](crate::select) statement and some other branch
    /// completes first, then it is guaranteed that no values have been marked
    /// seen by this call to `changed`.
    ///
    /// [`Sender`]: struct@Sender
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, mut rx) = watch::channel("hello");
    ///
    ///     tokio::spawn(async move {
    ///         tx.send("goodbye").unwrap();
    ///     });
    ///
    ///     assert!(rx.changed().await.is_ok());
    ///     assert_eq!(*rx.borrow(), "goodbye");
    ///
    ///     // The `tx` handle has been dropped
    ///     assert!(rx.changed().await.is_err());
    /// }
    /// ```
    pub async fn changed(&mut self) -> Result<(), error::RecvError> {
        loop {
            // In order to avoid a race condition, we first request a notification,
            // **then** check the current value's version. If a new version exists,
            // the notification request is dropped.
            let notified = self.shared.notify_rx.notified();

            if let Some(ret) = maybe_changed(&self.shared, &mut self.version) {
                return ret;
            }

            notified.await;
            // loop around again in case the wake-up was spurious
        }
    }

    /// Returns `true` if receivers belong to the same channel.
    ///
    /// # Examples
    ///
    /// ```
    /// let (tx, rx) = tokio::sync::watch::channel(true);
    /// let rx2 = rx.clone();
    /// assert!(rx.same_channel(&rx2));
    ///
    /// let (tx3, rx3) = tokio::sync::watch::channel(true);
    /// assert!(!rx3.same_channel(&rx2));
    /// ```
    pub fn same_channel(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.shared, &other.shared)
    }

    cfg_process_driver! {
        pub(crate) fn try_has_changed(&mut self) -> Option<Result<(), error::RecvError>> {
            maybe_changed(&self.shared, &mut self.version)
        }
    }
}

fn maybe_changed<T>(
    shared: &Shared<T>,
    version: &mut Version,
) -> Option<Result<(), error::RecvError>> {
    // Load the version from the state
    let state = shared.state.load();
    let new_version = state.version();

    if *version != new_version {
        // Observe the new version and return
        *version = new_version;
        return Some(Ok(()));
    }

    if state.is_closed() {
        // All receivers have dropped.
        return Some(Err(error::RecvError(())));
    }

    None
}

impl<T> Clone for Receiver<T> {
    fn clone(&self) -> Self {
        let version = self.version;
        let shared = self.shared.clone();

        Self::from_shared(version, shared)
    }
}

impl<T> Drop for Receiver<T> {
    fn drop(&mut self) {
        // No synchronization necessary as this is only used as a counter and
        // not memory access.
        if 1 == self.shared.ref_count_rx.fetch_sub(1, Relaxed) {
            // This is the last `Receiver` handle, tasks waiting on `Sender::closed()`
            self.shared.notify_tx.notify_waiters();
        }
    }
}

impl<T> Sender<T> {
    /// Sends a new value via the channel, notifying all receivers.
    ///
    /// This method fails if the channel has been closed, which happens when
    /// every receiver has been dropped.
    pub fn send(&self, value: T) -> Result<(), error::SendError<T>> {
        // This is pretty much only useful as a hint anyway, so synchronization isn't critical.
        if 0 == self.receiver_count() {
            return Err(error::SendError(value));
        }

        self.send_replace(value);
        Ok(())
    }

    /// Modifies the watched value **unconditionally** in-place,
    /// notifying all receivers.
    ///
    /// This can useful for modifying the watched value, without
    /// having to allocate a new instance. Additionally, this
    /// method permits sending values even when there are no receivers.
    ///
    /// Prefer to use the more versatile function [`Self::send_if_modified()`]
    /// if the value is only modified conditionally during the mutable borrow
    /// to prevent unneeded change notifications for unmodified values.
    ///
    /// # Panics
    ///
    /// This function panics when the invocation of the `modify` closure panics.
    /// No receivers are notified when panicking. All changes of the watched
    /// value applied by the closure before panicking will be visible in
    /// subsequent calls to `borrow`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// struct State {
    ///     counter: usize,
    /// }
    /// let (state_tx, state_rx) = watch::channel(State { counter: 0 });
    /// state_tx.send_modify(|state| state.counter += 1);
    /// assert_eq!(state_rx.borrow().counter, 1);
    /// ```
    pub fn send_modify<F>(&self, modify: F)
    where
        F: FnOnce(&mut T),
    {
        self.send_if_modified(|value| {
            modify(value);
            true
        });
    }

    /// Modifies the watched value **conditionally** in-place,
    /// notifying all receivers only if modified.
    ///
    /// This can useful for modifying the watched value, without
    /// having to allocate a new instance. Additionally, this
    /// method permits sending values even when there are no receivers.
    ///
    /// The `modify` closure must return `true` if the value has actually
    /// been modified during the mutable borrow. It should only return `false`
    /// if the value is guaranteed to be unnmodified despite the mutable
    /// borrow.
    ///
    /// Receivers are only notified if the closure returned `true`. If the
    /// closure has modified the value but returned `false` this results
    /// in a *silent modification*, i.e. the modified value will be visible
    /// in subsequent calls to `borrow`, but receivers will not receive
    /// a change notification.
    ///
    /// Returns the result of the closure, i.e. `true` if the value has
    /// been modified and `false` otherwise.
    ///
    /// # Panics
    ///
    /// This function panics when the invocation of the `modify` closure panics.
    /// No receivers are notified when panicking. All changes of the watched
    /// value applied by the closure before panicking will be visible in
    /// subsequent calls to `borrow`.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// struct State {
    ///     counter: usize,
    /// }
    /// let (state_tx, mut state_rx) = watch::channel(State { counter: 1 });
    /// let inc_counter_if_odd = |state: &mut State| {
    ///     if state.counter % 2 == 1 {
    ///         state.counter += 1;
    ///         return true;
    ///     }
    ///     false
    /// };
    ///
    /// assert_eq!(state_rx.borrow().counter, 1);
    ///
    /// assert!(!state_rx.has_changed().unwrap());
    /// assert!(state_tx.send_if_modified(inc_counter_if_odd));
    /// assert!(state_rx.has_changed().unwrap());
    /// assert_eq!(state_rx.borrow_and_update().counter, 2);
    ///
    /// assert!(!state_rx.has_changed().unwrap());
    /// assert!(!state_tx.send_if_modified(inc_counter_if_odd));
    /// assert!(!state_rx.has_changed().unwrap());
    /// assert_eq!(state_rx.borrow_and_update().counter, 2);
    /// ```
    pub fn send_if_modified<F>(&self, modify: F) -> bool
    where
        F: FnOnce(&mut T) -> bool,
    {
        {
            // Acquire the write lock and update the value.
            let mut lock = self.shared.value.write().unwrap();

            // Update the value and catch possible panic inside func.
            let result = panic::catch_unwind(panic::AssertUnwindSafe(|| modify(&mut lock)));
            match result {
                Ok(modified) => {
                    if !modified {
                        // Abort, i.e. don't notify receivers if unmodified
                        return false;
                    }
                    // Continue if modified
                }
                Err(panicked) => {
                    // Drop the lock to avoid poisoning it.
                    drop(lock);
                    // Forward the panic to the caller.
                    panic::resume_unwind(panicked);
                    // Unreachable
                }
            };

            self.shared.state.increment_version();

            // Release the write lock.
            //
            // Incrementing the version counter while holding the lock ensures
            // that receivers are able to figure out the version number of the
            // value they are currently looking at.
            drop(lock);
        }

        self.shared.notify_rx.notify_waiters();

        true
    }

    /// Sends a new value via the channel, notifying all receivers and returning
    /// the previous value in the channel.
    ///
    /// This can be useful for reusing the buffers inside a watched value.
    /// Additionally, this method permits sending values even when there are no
    /// receivers.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// let (tx, _rx) = watch::channel(1);
    /// assert_eq!(tx.send_replace(2), 1);
    /// assert_eq!(tx.send_replace(3), 2);
    /// ```
    pub fn send_replace(&self, mut value: T) -> T {
        // swap old watched value with the new one
        self.send_modify(|old| mem::swap(old, &mut value));

        value
    }

    /// Returns a reference to the most recently sent value
    ///
    /// Outstanding borrows hold a read lock. This means that long lived borrows
    /// could cause the send half to block. It is recommended to keep the borrow
    /// as short lived as possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// let (tx, _) = watch::channel("hello");
    /// assert_eq!(*tx.borrow(), "hello");
    /// ```
    pub fn borrow(&self) -> Ref<'_, T> {
        let inner = self.shared.value.read().unwrap();

        // The sender/producer always sees the current version
        let has_changed = false;

        Ref { inner, has_changed }
    }

    /// Checks if the channel has been closed. This happens when all receivers
    /// have dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// let (tx, rx) = tokio::sync::watch::channel(());
    /// assert!(!tx.is_closed());
    ///
    /// drop(rx);
    /// assert!(tx.is_closed());
    /// ```
    pub fn is_closed(&self) -> bool {
        self.receiver_count() == 0
    }

    /// Completes when all receivers have dropped.
    ///
    /// This allows the producer to get notified when interest in the produced
    /// values is canceled and immediately stop doing work.
    ///
    /// # Cancel safety
    ///
    /// This method is cancel safe. Once the channel is closed, it stays closed
    /// forever and all future calls to `closed` will return immediately.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, rx) = watch::channel("hello");
    ///
    ///     tokio::spawn(async move {
    ///         // use `rx`
    ///         drop(rx);
    ///     });
    ///
    ///     // Waits for `rx` to drop
    ///     tx.closed().await;
    ///     println!("the `rx` handles dropped")
    /// }
    /// ```
    pub async fn closed(&self) {
        while self.receiver_count() > 0 {
            let notified = self.shared.notify_tx.notified();

            if self.receiver_count() == 0 {
                return;
            }

            notified.await;
            // The channel could have been reopened in the meantime by calling
            // `subscribe`, so we loop again.
        }
    }

    /// Creates a new [`Receiver`] connected to this `Sender`.
    ///
    /// All messages sent before this call to `subscribe` are initially marked
    /// as seen by the new `Receiver`.
    ///
    /// This method can be called even if there are no other receivers. In this
    /// case, the channel is reopened.
    ///
    /// # Examples
    ///
    /// The new channel will receive messages sent on this `Sender`.
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, _rx) = watch::channel(0u64);
    ///
    ///     tx.send(5).unwrap();
    ///
    ///     let rx = tx.subscribe();
    ///     assert_eq!(5, *rx.borrow());
    ///
    ///     tx.send(10).unwrap();
    ///     assert_eq!(10, *rx.borrow());
    /// }
    /// ```
    ///
    /// The most recent message is considered seen by the channel, so this test
    /// is guaranteed to pass.
    ///
    /// ```
    /// use tokio::sync::watch;
    /// use tokio::time::Duration;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, _rx) = watch::channel(0u64);
    ///     tx.send(5).unwrap();
    ///     let mut rx = tx.subscribe();
    ///
    ///     tokio::spawn(async move {
    ///         // by spawning and sleeping, the message is sent after `main`
    ///         // hits the call to `changed`.
    ///         # if false {
    ///         tokio::time::sleep(Duration::from_millis(10)).await;
    ///         # }
    ///         tx.send(100).unwrap();
    ///     });
    ///
    ///     rx.changed().await.unwrap();
    ///     assert_eq!(100, *rx.borrow());
    /// }
    /// ```
    pub fn subscribe(&self) -> Receiver<T> {
        let shared = self.shared.clone();
        let version = shared.state.load().version();

        // The CLOSED bit in the state tracks only whether the sender is
        // dropped, so we do not need to unset it if this reopens the channel.
        Receiver::from_shared(version, shared)
    }

    /// Returns the number of receivers that currently exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokio::sync::watch;
    ///
    /// #[tokio::main]
    /// async fn main() {
    ///     let (tx, rx1) = watch::channel("hello");
    ///
    ///     assert_eq!(1, tx.receiver_count());
    ///
    ///     let mut _rx2 = rx1.clone();
    ///
    ///     assert_eq!(2, tx.receiver_count());
    /// }
    /// ```
    pub fn receiver_count(&self) -> usize {
        self.shared.ref_count_rx.load(Relaxed)
    }
}

impl<T> Drop for Sender<T> {
    fn drop(&mut self) {
        self.shared.state.set_closed();
        self.shared.notify_rx.notify_waiters();
    }
}

// ===== impl Ref =====

impl<T> ops::Deref for Ref<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        self.inner.deref()
    }
}

#[cfg(all(test, loom))]
mod tests {
    use futures::future::FutureExt;
    use loom::thread;

    // test for https://github.com/tokio-rs/tokio/issues/3168
    #[test]
    fn watch_spurious_wakeup() {
        loom::model(|| {
            let (send, mut recv) = crate::sync::watch::channel(0i32);

            send.send(1).unwrap();

            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
                send
            });

            recv.changed().now_or_never();

            let send = send_thread.join().unwrap();
            let recv_thread = thread::spawn(move || {
                recv.changed().now_or_never();
                recv.changed().now_or_never();
                recv
            });

            send.send(3).unwrap();

            let mut recv = recv_thread.join().unwrap();
            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
            });

            recv.changed().now_or_never();

            send_thread.join().unwrap();
        });
    }

    #[test]
    fn watch_borrow() {
        loom::model(|| {
            let (send, mut recv) = crate::sync::watch::channel(0i32);

            assert!(send.borrow().eq(&0));
            assert!(recv.borrow().eq(&0));

            send.send(1).unwrap();
            assert!(send.borrow().eq(&1));

            let send_thread = thread::spawn(move || {
                send.send(2).unwrap();
                send
            });

            recv.changed().now_or_never();

            let send = send_thread.join().unwrap();
            let recv_thread = thread::spawn(move || {
                recv.changed().now_or_never();
                recv.changed().now_or_never();
                recv
            });

            send.send(3).unwrap();

            let recv = recv_thread.join().unwrap();
            assert!(recv.borrow().eq(&3));
            assert!(send.borrow().eq(&3));

            send.send(2).unwrap();

            thread::spawn(move || {
                assert!(recv.borrow().eq(&2));
            });
            assert!(send.borrow().eq(&2));
        });
    }
}
