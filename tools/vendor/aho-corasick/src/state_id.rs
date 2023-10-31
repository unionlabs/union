use std::fmt::Debug;
use std::hash::Hash;

use crate::error::{Error, Result};

// NOTE: Most of this code was copied from regex-automata, but without the
// (de)serialization specific stuff.

/// Check that the premultiplication of the given state identifier can
/// fit into the representation indicated by `S`. If it cannot, or if it
/// overflows `usize` itself, then an error is returned.
pub fn premultiply_overflow_error<S: StateID>(
    last_state: S,
    alphabet_len: usize,
) -> Result<()> {
    let requested = match last_state.to_usize().checked_mul(alphabet_len) {
        Some(requested) => requested,
        None => return Err(Error::premultiply_overflow(0, 0)),
    };
    if requested > S::max_id() {
        return Err(Error::premultiply_overflow(S::max_id(), requested));
    }
    Ok(())
}

/// Convert the given `usize` to the chosen state identifier
/// representation. If the given value cannot fit in the chosen
/// representation, then an error is returned.
pub fn usize_to_state_id<S: StateID>(value: usize) -> Result<S> {
    if value > S::max_id() {
        Err(Error::state_id_overflow(S::max_id()))
    } else {
        Ok(S::from_usize(value))
    }
}

/// Return the unique identifier for an automaton's fail state in the chosen
/// representation indicated by `S`.
pub fn fail_id<S: StateID>() -> S {
    S::from_usize(0)
}

/// Return the unique identifier for an automaton's fail state in the chosen
/// representation indicated by `S`.
pub fn dead_id<S: StateID>() -> S {
    S::from_usize(1)
}

mod private {
    /// Sealed stops crates other than aho-corasick from implementing any
    /// traits that use it.
    pub trait Sealed {}
    impl Sealed for u8 {}
    impl Sealed for u16 {}
    impl Sealed for u32 {}
    impl Sealed for u64 {}
    impl Sealed for usize {}
}

/// A trait describing the representation of an automaton's state identifier.
///
/// The purpose of this trait is to safely express both the possible state
/// identifier representations that can be used in an automaton and to convert
/// between state identifier representations and types that can be used to
/// efficiently index memory (such as `usize`).
///
/// In general, one should not need to implement this trait explicitly. Indeed,
/// for now, this trait is sealed such that it cannot be implemented by any
/// other type. In particular, this crate provides implementations for `u8`,
/// `u16`, `u32`, `u64` and `usize`. (`u32` and `u64` are only provided for
/// targets that can represent all corresponding values in a `usize`.)
pub trait StateID:
    private::Sealed
    + Clone
    + Copy
    + Debug
    + Eq
    + Hash
    + PartialEq
    + PartialOrd
    + Ord
{
    /// Convert from a `usize` to this implementation's representation.
    ///
    /// Implementors may assume that `n <= Self::max_id`. That is, implementors
    /// do not need to check whether `n` can fit inside this implementation's
    /// representation.
    fn from_usize(n: usize) -> Self;

    /// Convert this implementation's representation to a `usize`.
    ///
    /// Implementors must not return a `usize` value greater than
    /// `Self::max_id` and must not permit overflow when converting between the
    /// implementor's representation and `usize`. In general, the preferred
    /// way for implementors to achieve this is to simply not provide
    /// implementations of `StateID` that cannot fit into the target platform's
    /// `usize`.
    fn to_usize(self) -> usize;

    /// Return the maximum state identifier supported by this representation.
    ///
    /// Implementors must return a correct bound. Doing otherwise may result
    /// in unspecified behavior (but will not violate memory safety).
    fn max_id() -> usize;
}

impl StateID for usize {
    #[inline]
    fn from_usize(n: usize) -> usize {
        n
    }

    #[inline]
    fn to_usize(self) -> usize {
        self
    }

    #[inline]
    fn max_id() -> usize {
        ::std::usize::MAX
    }
}

impl StateID for u8 {
    #[inline]
    fn from_usize(n: usize) -> u8 {
        n as u8
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u8::MAX as usize
    }
}

impl StateID for u16 {
    #[inline]
    fn from_usize(n: usize) -> u16 {
        n as u16
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u16::MAX as usize
    }
}

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
impl StateID for u32 {
    #[inline]
    fn from_usize(n: usize) -> u32 {
        n as u32
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u32::MAX as usize
    }
}

#[cfg(target_pointer_width = "64")]
impl StateID for u64 {
    #[inline]
    fn from_usize(n: usize) -> u64 {
        n as u64
    }

    #[inline]
    fn to_usize(self) -> usize {
        self as usize
    }

    #[inline]
    fn max_id() -> usize {
        ::std::u64::MAX as usize
    }
}
