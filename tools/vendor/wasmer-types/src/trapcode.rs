// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Trap codes describing the reason for a trap.

use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
use loupe::MemoryUsage;
#[cfg(feature = "enable-rkyv")]
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A trap code describing the reason for a trap.
///
/// All trap instructions have an explicit trap code.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash, Error, MemoryUsage)]
#[cfg_attr(
    feature = "enable-rkyv",
    derive(RkyvSerialize, RkyvDeserialize, Archive)
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[repr(u32)]
pub enum TrapCode {
    /// The current stack space was exhausted.
    ///
    /// On some platforms, a stack overflow may also be indicated by a segmentation fault from the
    /// stack guard page.
    StackOverflow = 0,

    /// A `heap_addr` instruction detected an out-of-bounds error.
    ///
    /// Note that not all out-of-bounds heap accesses are reported this way;
    /// some are detected by a segmentation fault on the heap unmapped or
    /// offset-guard pages.
    HeapAccessOutOfBounds = 1,

    /// A `heap_addr` instruction was misaligned.
    HeapMisaligned = 2,

    /// A `table_addr` instruction detected an out-of-bounds error.
    TableAccessOutOfBounds = 3,

    /// Other bounds checking error.
    OutOfBounds = 4,

    /// Indirect call to a null table entry.
    IndirectCallToNull = 5,

    /// Signature mismatch on indirect call.
    BadSignature = 6,

    /// An integer arithmetic operation caused an overflow.
    IntegerOverflow = 7,

    /// An integer division by zero.
    IntegerDivisionByZero = 8,

    /// Failed float-to-int conversion.
    BadConversionToInteger = 9,

    /// Code that was supposed to have been unreachable was reached.
    UnreachableCodeReached = 10,

    /// An atomic memory access was attempted with an unaligned pointer.
    UnalignedAtomic = 11,
}

impl TrapCode {
    /// Gets the message for this trap code
    pub fn message(&self) -> &str {
        match self {
            Self::StackOverflow => "call stack exhausted",
            Self::HeapAccessOutOfBounds => "out of bounds memory access",
            Self::HeapMisaligned => "misaligned heap",
            Self::TableAccessOutOfBounds => "undefined element: out of bounds table access",
            Self::OutOfBounds => "out of bounds",
            Self::IndirectCallToNull => "uninitialized element",
            Self::BadSignature => "indirect call type mismatch",
            Self::IntegerOverflow => "integer overflow",
            Self::IntegerDivisionByZero => "integer divide by zero",
            Self::BadConversionToInteger => "invalid conversion to integer",
            Self::UnreachableCodeReached => "unreachable",
            Self::UnalignedAtomic => "unaligned atomic access",
        }
    }
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        let identifier = match *self {
            Self::StackOverflow => "stk_ovf",
            Self::HeapAccessOutOfBounds => "heap_get_oob",
            Self::HeapMisaligned => "heap_misaligned",
            Self::TableAccessOutOfBounds => "table_get_oob",
            Self::OutOfBounds => "oob",
            Self::IndirectCallToNull => "icall_null",
            Self::BadSignature => "bad_sig",
            Self::IntegerOverflow => "int_ovf",
            Self::IntegerDivisionByZero => "int_divz",
            Self::BadConversionToInteger => "bad_toint",
            Self::UnreachableCodeReached => "unreachable",
            Self::UnalignedAtomic => "unalign_atom",
        };
        f.write_str(identifier)
    }
}

impl FromStr for TrapCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "stk_ovf" => Ok(TrapCode::StackOverflow),
            "heap_get_oob" => Ok(TrapCode::HeapAccessOutOfBounds),
            "heap_misaligned" => Ok(TrapCode::HeapMisaligned),
            "table_get_oob" => Ok(TrapCode::TableAccessOutOfBounds),
            "oob" => Ok(TrapCode::OutOfBounds),
            "icall_null" => Ok(TrapCode::IndirectCallToNull),
            "bad_sig" => Ok(TrapCode::BadSignature),
            "int_ovf" => Ok(TrapCode::IntegerOverflow),
            "int_divz" => Ok(TrapCode::IntegerDivisionByZero),
            "bad_toint" => Ok(TrapCode::BadConversionToInteger),
            "unreachable" => Ok(TrapCode::UnreachableCodeReached),
            "unalign_atom" => Ok(TrapCode::UnalignedAtomic),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Everything but user-defined codes.
    const CODES: [TrapCode; 12] = [
        TrapCode::StackOverflow,
        TrapCode::HeapAccessOutOfBounds,
        TrapCode::HeapMisaligned,
        TrapCode::TableAccessOutOfBounds,
        TrapCode::OutOfBounds,
        TrapCode::IndirectCallToNull,
        TrapCode::BadSignature,
        TrapCode::IntegerOverflow,
        TrapCode::IntegerDivisionByZero,
        TrapCode::BadConversionToInteger,
        TrapCode::UnreachableCodeReached,
        TrapCode::UnalignedAtomic,
    ];

    #[test]
    fn display() {
        for r in &CODES {
            let tc = *r;
            assert_eq!(tc.to_string().parse(), Ok(tc));
        }
        assert_eq!("bogus".parse::<TrapCode>(), Err(()));

        // assert_eq!(TrapCode::User(17).to_string(), "user17");
        // assert_eq!("user22".parse(), Ok(TrapCode::User(22)));
        assert_eq!("user".parse::<TrapCode>(), Err(()));
        assert_eq!("user-1".parse::<TrapCode>(), Err(()));
        assert_eq!("users".parse::<TrapCode>(), Err(()));
    }
}
