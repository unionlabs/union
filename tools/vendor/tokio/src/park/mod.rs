//! Abstraction over blocking and unblocking the current thread.
//!
//! Provides an abstraction over blocking the current thread. This is similar to
//! the park / unpark constructs provided by `std` but made generic. This allows
//! embedding custom functionality to perform when the thread is blocked.
//!
//! A blocked `Park` instance is unblocked by calling `unpark` on its
//! `Unpark` handle.
//!
//! The `ParkThread` struct implements `Park` using `thread::park` to put the
//! thread to sleep. The Tokio reactor also implements park, but uses
//! `mio::Poll` to block the thread instead.
//!
//! The `Park` trait is composable. A timer implementation might decorate a
//! `Park` implementation by checking if any timeouts have elapsed after the
//! inner `Park` implementation unblocks.
//!
//! # Model
//!
//! Conceptually, each `Park` instance has an associated token, which is
//! initially not present:
//!
//! * The `park` method blocks the current thread unless or until the token is
//!   available, at which point it atomically consumes the token.
//! * The `unpark` method atomically makes the token available if it wasn't
//!   already.
//!
//! Some things to note:
//!
//! * If `unpark` is called before `park`, the next call to `park` will
//!   **not** block the thread.
//! * **Spurious** wakeups are permitted, i.e., the `park` method may unblock
//!   even if `unpark` was not called.
//! * `park_timeout` does the same as `park` but allows specifying a maximum
//!   time to block the thread for.

cfg_rt! {
    pub(crate) mod either;
}

#[cfg(any(feature = "rt", feature = "sync"))]
pub(crate) mod thread;

use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;

/// Blocks the current thread.
pub(crate) trait Park {
    /// Unpark handle type for the `Park` implementation.
    type Unpark: Unpark;

    /// Error returned by `park`.
    type Error: Debug;

    /// Gets a new `Unpark` handle associated with this `Park` instance.
    fn unpark(&self) -> Self::Unpark;

    /// Blocks the current thread unless or until the token is available.
    ///
    /// A call to `park` does not guarantee that the thread will remain blocked
    /// forever, and callers should be prepared for this possibility. This
    /// function may wakeup spuriously for any reason.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Park` implementation.
    fn park(&mut self) -> Result<(), Self::Error>;

    /// Parks the current thread for at most `duration`.
    ///
    /// This function is the same as `park` but allows specifying a maximum time
    /// to block the thread for.
    ///
    /// Same as `park`, there is no guarantee that the thread will remain
    /// blocked for any amount of time. Spurious wakeups are permitted for any
    /// reason.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Park` implementation.
    fn park_timeout(&mut self, duration: Duration) -> Result<(), Self::Error>;

    /// Releases all resources held by the parker for proper leak-free shutdown.
    fn shutdown(&mut self);
}

/// Unblock a thread blocked by the associated `Park` instance.
pub(crate) trait Unpark: Sync + Send + 'static {
    /// Unblocks a thread that is blocked by the associated `Park` handle.
    ///
    /// Calling `unpark` atomically makes available the unpark token, if it is
    /// not already available.
    ///
    /// # Panics
    ///
    /// This function **should** not panic, but ultimately, panics are left as
    /// an implementation detail. Refer to the documentation for the specific
    /// `Unpark` implementation.
    fn unpark(&self);
}

impl Unpark for Box<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}

impl Unpark for Arc<dyn Unpark> {
    fn unpark(&self) {
        (**self).unpark()
    }
}
