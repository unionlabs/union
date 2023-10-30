#[cfg(not(futures_no_atomic_cas))]
mod atomic_waker;
#[cfg(not(futures_no_atomic_cas))]
pub use self::atomic_waker::AtomicWaker;
