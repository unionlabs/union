//! Atomic types.
//!
//! * [`AtomicCell`], a thread-safe mutable memory location.
//! * [`AtomicConsume`], for reading from primitive atomic types with "consume" ordering.

#[cfg(not(crossbeam_no_atomic_cas))]
#[cfg(not(crossbeam_loom))]
cfg_if::cfg_if! {
    // Use "wide" sequence lock if the pointer width <= 32 for preventing its counter against wrap
    // around.
    //
    // We are ignoring too wide architectures (pointer width >= 256), since such a system will not
    // appear in a conceivable future.
    //
    // In narrow architectures (pointer width <= 16), the counter is still <= 32-bit and may be
    // vulnerable to wrap around. But it's mostly okay, since in such a primitive hardware, the
    // counter will not be increased that fast.
    if #[cfg(any(target_pointer_width = "64", target_pointer_width = "128"))] {
        mod seq_lock;
    } else {
        #[path = "seq_lock_wide.rs"]
        mod seq_lock;
    }
}

#[cfg(not(crossbeam_no_atomic_cas))]
// We cannot provide AtomicCell under cfg(crossbeam_loom) because loom's atomic
// types have a different in-memory representation than the underlying type.
// TODO: The latest loom supports fences, so fallback using seqlock may be available.
#[cfg(not(crossbeam_loom))]
mod atomic_cell;
mod consume;

#[cfg(not(crossbeam_no_atomic_cas))]
#[cfg(not(crossbeam_loom))]
pub use self::atomic_cell::AtomicCell;
pub use self::consume::AtomicConsume;
