#![deny(missing_debug_implementations)]
#![deny(missing_docs)]
#![deny(unreachable_pub)]
#![warn(rust_2018_idioms)]

//! Data-parallelism library that makes it easy to convert sequential
//! computations into parallel
//!
//! Rayon is lightweight and convenient for introducing parallelism into existing
//! code. It guarantees data-race free executions and takes advantage of
//! parallelism when sensible, based on work-load at runtime.
//!
//! # How to use Rayon
//!
//! There are two ways to use Rayon:
//!
//! - **High-level parallel constructs** are the simplest way to use Rayon and also
//!   typically the most efficient.
//!   - [Parallel iterators][iter module] make it easy to convert a sequential iterator to
//!     execute in parallel.
//!     - The [`ParallelIterator`] trait defines general methods for all parallel iterators.
//!     - The [`IndexedParallelIterator`] trait adds methods for iterators that support random
//!       access.
//!   - The [`par_sort`] method sorts `&mut [T]` slices (or vectors) in parallel.
//!   - [`par_extend`] can be used to efficiently grow collections with items produced
//!     by a parallel iterator.
//! - **Custom tasks** let you divide your work into parallel tasks yourself.
//!   - [`join`] is used to subdivide a task into two pieces.
//!   - [`scope`] creates a scope within which you can create any number of parallel tasks.
//!   - [`ThreadPoolBuilder`] can be used to create your own thread pools or customize
//!     the global one.
//!
//! [iter module]: iter/index.html
//! [`join`]: fn.join.html
//! [`scope`]: fn.scope.html
//! [`par_sort`]: slice/trait.ParallelSliceMut.html#method.par_sort
//! [`par_extend`]: iter/trait.ParallelExtend.html#tymethod.par_extend
//! [`ThreadPoolBuilder`]: struct.ThreadPoolBuilder.html
//!
//! # Basic usage and the Rayon prelude
//!
//! First, you will need to add `rayon` to your `Cargo.toml`.
//!
//! Next, to use parallel iterators or the other high-level methods,
//! you need to import several traits. Those traits are bundled into
//! the module [`rayon::prelude`]. It is recommended that you import
//! all of these traits at once by adding `use rayon::prelude::*` at
//! the top of each module that uses Rayon methods.
//!
//! These traits give you access to the `par_iter` method which provides
//! parallel implementations of many iterative functions such as [`map`],
//! [`for_each`], [`filter`], [`fold`], and [more].
//!
//! [`rayon::prelude`]: prelude/index.html
//! [`map`]: iter/trait.ParallelIterator.html#method.map
//! [`for_each`]: iter/trait.ParallelIterator.html#method.for_each
//! [`filter`]: iter/trait.ParallelIterator.html#method.filter
//! [`fold`]: iter/trait.ParallelIterator.html#method.fold
//! [more]: iter/trait.ParallelIterator.html#provided-methods
//! [`ParallelIterator`]: iter/trait.ParallelIterator.html
//! [`IndexedParallelIterator`]: iter/trait.IndexedParallelIterator.html
//!
//! # Crate Layout
//!
//! Rayon extends many of the types found in the standard library with
//! parallel iterator implementations. The modules in the `rayon`
//! crate mirror [`std`] itself: so, e.g., the `option` module in
//! Rayon contains parallel iterators for the `Option` type, which is
//! found in [the `option` module of `std`]. Similarly, the
//! `collections` module in Rayon offers parallel iterator types for
//! [the `collections` from `std`]. You will rarely need to access
//! these submodules unless you need to name iterator types
//! explicitly.
//!
//! [the `option` module of `std`]: https://doc.rust-lang.org/std/option/index.html
//! [the `collections` from `std`]: https://doc.rust-lang.org/std/collections/index.html
//! [`std`]: https://doc.rust-lang.org/std/
//!
//! # Targets without threading
//!
//! Rayon has limited support for targets without `std` threading implementations.
//! See the [`rayon_core`] documentation for more information about its global fallback.
//!
//! # Other questions?
//!
//! See [the Rayon FAQ][faq].
//!
//! [faq]: https://github.com/rayon-rs/rayon/blob/master/FAQ.md

#[macro_use]
mod delegate;

#[macro_use]
mod private;

mod split_producer;

pub mod array;
pub mod collections;
pub mod iter;
pub mod option;
pub mod prelude;
pub mod range;
pub mod range_inclusive;
pub mod result;
pub mod slice;
pub mod str;
pub mod string;
pub mod vec;

mod math;
mod par_either;

mod compile_fail;

pub use rayon_core::FnContext;
pub use rayon_core::ThreadBuilder;
pub use rayon_core::ThreadPool;
pub use rayon_core::ThreadPoolBuildError;
pub use rayon_core::ThreadPoolBuilder;
pub use rayon_core::{broadcast, spawn_broadcast, BroadcastContext};
pub use rayon_core::{current_num_threads, current_thread_index, max_num_threads};
pub use rayon_core::{in_place_scope, scope, Scope};
pub use rayon_core::{in_place_scope_fifo, scope_fifo, ScopeFifo};
pub use rayon_core::{join, join_context};
pub use rayon_core::{spawn, spawn_fifo};
pub use rayon_core::{yield_local, yield_now, Yield};

/// We need to transmit raw pointers across threads. It is possible to do this
/// without any unsafe code by converting pointers to usize or to AtomicPtr<T>
/// then back to a raw pointer for use. We prefer this approach because code
/// that uses this type is more explicit.
///
/// Unsafe code is still required to dereference the pointer, so this type is
/// not unsound on its own, although it does partly lift the unconditional
/// !Send and !Sync on raw pointers. As always, dereference with care.
struct SendPtr<T>(*mut T);

// SAFETY: !Send for raw pointers is not for safety, just as a lint
unsafe impl<T: Send> Send for SendPtr<T> {}

// SAFETY: !Sync for raw pointers is not for safety, just as a lint
unsafe impl<T: Send> Sync for SendPtr<T> {}

impl<T> SendPtr<T> {
    // Helper to avoid disjoint captures of `send_ptr.0`
    fn get(self) -> *mut T {
        self.0
    }
}

// Implement Clone without the T: Clone bound from the derive
impl<T> Clone for SendPtr<T> {
    fn clone(&self) -> Self {
        Self(self.0)
    }
}

// Implement Copy without the T: Copy bound from the derive
impl<T> Copy for SendPtr<T> {}
