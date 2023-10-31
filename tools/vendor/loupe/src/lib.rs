//! 🔎 `loupe` is a set of tools to analyse and to profile Rust
//! code. For the moment, it only provides tools about memory
//! usage. It's mostly driven by
//! [Wasmer](https://github.com/wasmerio/wasmer)'s needs, but feel
//! free to propose new features!
//!
//! `loupe` is a French word to express _magnifying glass_, and can be
//! pronounced exactly like _loop_.
//!
//! ## Memory Usage
//!
//! `loupe` provides the [`MemoryUsage`] trait. It allows to know the
//! size of a value in bytes, _recursively_. So it traverses most of
//! the types (some are missing, feel free to contribute!), and its
//! fields or variants as deep as possible. Hopefully, it tracks
//! already visited values so that it doesn't enter an infinite
//! ~~_loupe_~~ loop. The trait looks like this:
//!
//! ```rust,ignore
//! pub trait MemoryUsage {
//!     fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize;
//! }
//! ```
//!
//! `loupe` provides a [`size_of_val`] function that is a close sibling
//! of
//! [`std::mem::size_of_val`](https://doc.rust-lang.org/std/mem/fn.size_of_val.html). It
//! can be used the same way.
//!
//! `loupe` exports its best companion: handful procedural macros from
//! the [`loupe-derive`](../loupe_derive/index.html) crate, to
//! automatically implement the [`MemoryUsage`] trait if possible, on
//! `struct`s and `enum`s.
//!
//! Thus, one only needs to write:
//!
//! ```rust
//! use loupe::MemoryUsage;
//! use std::mem;
//!
//! #[derive(MemoryUsage)]
//! struct S {
//!     x: Vec<i32>,
//!     y: Vec<i32>,
//! }
//!
//! fn main() {
//!     let s = S {
//!         x: vec![1, 2, 3],
//!         y: vec![1, 2, 3],
//!     };
//!     
//!     assert_eq!(48, mem::size_of_val(&s));
//!     assert_eq!(72, loupe::size_of_val(&s));
//! }
//! ```
//!
//! In the example above, we see that each elements of `Vec<i32>` has
//! been counted in the size of the value `s`. In
//! [Wasmer](https://github.com/wasmerio/wasmer), it is possible to
//! get the size of an `Instance`, which traverses `Module`, `Store`,
//! `Engine`, `Compiler` etc. It's an entire tree of values that is
//! traversed and the size of each value is summed.
//!
//! ### Opinionated implementations
//!
//! Even if [`MemoryUsage`] is already implemented for common types,
//! some types are missing. We happily welcome more implementations!
//! However, implementations of `MemoryUsage`:
//!
//! * must never alter the values,
//! * must never panic of fail,
//! * must be deterministic as much as possible (ideally, everytime).
//!
//! In the same spirit, our implementation of [`MemoryUsage`] for
//! `*const T` or `*mut T` (and other pointer types, like `NonNull`,
//! `UnsafeCell` etc.) just returns the size of the pointer, but it
//! doesn't dereference the pointer as it's unsafe. It doesn't mean
//! one must not do that: It's totally possible if it's sure that the
//! pointer can be safely dereferenced.
//!
//! Remember that a user can implement [`MemoryUsage`] by hand; no
//! need to try to have a default implementation for all the standard
//! types.
//!
//! Finally, our implementations are certainly not perfect! Feel free to
//! challenge it and come to discuss!

mod memory_usage;

#[cfg(feature = "derive")]
pub use loupe_derive::*;
pub use memory_usage::*;

use std::collections::BTreeSet;

/// Returns the size of the pointer-to value in bytes. The size is
/// calculated with `MemoryUsage::size_of_val`.
///
/// # Example
///
/// ```rust
/// use loupe::MemoryUsage;
/// use std::mem;
///
/// #[derive(MemoryUsage)]
/// struct S {
///     x: Vec<i32>,
///     y: Vec<i32>,
/// }
///
/// fn main() {
///     let s = S {
///         x: vec![1, 2, 3],
///         y: vec![1, 2, 3],
///     };
///     
///     assert_eq!(48, mem::size_of_val(&s));
///     assert_eq!(72, loupe::size_of_val(&s));
/// }
/// ```
pub fn size_of_val<T: MemoryUsage>(value: &T) -> usize {
    <T as MemoryUsage>::size_of_val(value, &mut BTreeSet::new())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_of_val_helper() {
        assert_eq!(size_of_val(&"abc"), 2 * POINTER_BYTE_SIZE + 1 * 3);
    }
}
