//! Concurrent queues.
//!
//! This crate provides concurrent queues that can be shared among threads:
//!
//! * [`ArrayQueue`], a bounded MPMC queue that allocates a fixed-capacity buffer on construction.
//! * [`SegQueue`], an unbounded MPMC queue that allocates small buffers, segments, on demand.

#![doc(test(
    no_crate_inject,
    attr(
        deny(warnings, rust_2018_idioms),
        allow(dead_code, unused_assignments, unused_variables)
    )
))]
#![warn(
    missing_docs,
    missing_debug_implementations,
    rust_2018_idioms,
    unreachable_pub
)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(crossbeam_no_atomic_cas))]
cfg_if::cfg_if! {
    if #[cfg(feature = "alloc")] {
        extern crate alloc;

        mod array_queue;
        mod seg_queue;

        pub use self::array_queue::ArrayQueue;
        pub use self::seg_queue::SegQueue;
    }
}
