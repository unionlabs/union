#![cfg_attr(not(feature = "std"), no_std)]
#![forbid(unsafe_code)]
#![cfg_attr(
  feature = "nightly_slice_partition_dedup",
  feature(slice_partition_dedup)
)]
#![cfg_attr(docs_rs, feature(doc_cfg))]
#![warn(clippy::missing_inline_in_public_items)]
#![warn(clippy::must_use_candidate)]
#![warn(missing_docs)]

//! `tinyvec` provides 100% safe vec-like data structures.
//!
//! ## Provided Types
//! With no features enabled, this crate provides the [`ArrayVec`] type, which
//! is an array-backed storage. You can push values into the array and pop them
//! out of the array and so on. If the array is made to overflow it will panic.
//!
//! Similarly, there is also a [`SliceVec`] type available, which is a vec-like
//! that's backed by a slice you provide. You can add and remove elements, but
//! if you overflow the slice it will panic.
//!
//! With the `alloc` feature enabled, the crate also has a [`TinyVec`] type.
//! This is an enum type which is either an `Inline(ArrayVec)` or a `Heap(Vec)`.
//! If a `TinyVec` is `Inline` and would overflow it automatically transitions
//! itself into being `Heap` mode instead of a panic.
//!
//! All of this is done with no `unsafe` code within the crate. Technically the
//! `Vec` type from the standard library uses `unsafe` internally, but *this
//! crate* introduces no new `unsafe` code into your project.
//!
//! The limitation is that the element type of a vec from this crate must
//! support the [`Default`] trait. This means that this crate isn't suitable for
//! all situations, but a very surprising number of types do support `Default`.
//!
//! ## Other Features
//! * `grab_spare_slice` lets you get access to the "inactive" portions of an
//!   ArrayVec.
//! * `rustc_1_40` makes the crate assume a minimum rust version of `1.40.0`,
//!   which allows some better internal optimizations.
//! * `serde` provides a `Serialize` and `Deserialize` implementation for
//!   [`TinyVec`] and [`ArrayVec`] types, provided the inner item also has an
//!   implementation.
//!
//! ## API
//! The general goal of the crate is that, as much as possible, the vecs here
//! should be a "drop in" replacement for the standard library `Vec` type. We
//! strive to provide all of the `Vec` methods with the same names and
//! signatures. The exception is that the element type of some methods will have
//! a `Default` bound that's not part of the normal `Vec` type.
//!
//! The vecs here also have a few additional methods that aren't on the `Vec`
//! type. In this case, the names tend to be fairly long so that they are
//! unlikely to clash with any future methods added to `Vec`.
//!
//! ## Stability
//! * The `1.0` series of the crate works with Rustc `1.34.0` or later, though
//!   you still need to have Rustc `1.36.0` to use the `alloc` feature.
//! * The `2.0` version of the crate is planned for some time after the
//!   `min_const_generics` stuff becomes stable. This would greatly raise the
//!   minimum rust version and also allow us to totally eliminate the need for
//!   the `Array` trait. The actual usage of the crate is not expected to break
//!   significantly in this transition.

#[allow(unused_imports)]
use core::{
  borrow::{Borrow, BorrowMut},
  cmp::PartialEq,
  convert::AsMut,
  default::Default,
  fmt::{
    Binary, Debug, Display, Formatter, LowerExp, LowerHex, Octal, Pointer,
    UpperExp, UpperHex,
  },
  hash::{Hash, Hasher},
  iter::{Extend, FromIterator, FusedIterator, IntoIterator, Iterator},
  mem::{needs_drop, replace},
  ops::{Deref, DerefMut, Index, IndexMut, RangeBounds},
  slice::SliceIndex,
};

#[cfg(feature = "alloc")]
#[doc(hidden)] // re-export for macros
pub extern crate alloc;

mod array;
pub use array::*;

mod arrayvec;
pub use arrayvec::*;

mod arrayvec_drain;
pub use arrayvec_drain::*;

mod slicevec;
pub use slicevec::*;

#[cfg(feature = "alloc")]
mod tinyvec;
#[cfg(feature = "alloc")]
pub use crate::tinyvec::*;

// TODO MSRV(1.40.0): Just call the normal `core::mem::take`
#[inline(always)]
fn take<T: Default>(from: &mut T) -> T {
  replace(from, T::default())
}
