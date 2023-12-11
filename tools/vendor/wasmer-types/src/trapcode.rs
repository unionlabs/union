// This file contains code from external sources.
// Attributions: https://github.com/wasmerio/wasmer/blob/master/ATTRIBUTIONS.md

//! Trap codes describing the reason for a trap.

use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A trap code describing the reason for a trap.
///
/// All trap instructions have an explicit trap code.
#[derive(
    Clone,
    Copy,
    PartialEq,
    Eq,
    Debug,
    Hash,
    Error,
    RkyvSerialize,
    RkyvDeserialize,
    Archive,
    rkyv::CheckBytes,
)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
#[repr(u32)]
#[archive(as = "Self")]
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

    /// Indirect call to a null table entry.
    IndirectCallToNull = 4,

    /// Signature mismatch on indirect call.
    BadSignature = 5,

    /// An integer arithmetic operation caused an overflow.
    IntegerOverflow = 6,

    /// An integer division by zero.
    IntegerDivisionByZero = 7,

    /// Failed float-to-int conversion.
    BadConversionToInteger = 8,

    /// Code that was supposed to have been unreachable was reached.
    UnreachableCodeReached = 9,

    /// An atomic memory access was attempted with an unaligned pointer.
    UnalignedAtomic = 10,
}

impl TrapCode {
    /// Gets the message for this trap code
    pub fn message(&self) -> &str {
        match self {
            Self::StackOverflow => "call stack exhausted",
            Self::HeapAccessOutOfBounds => "out of bounds memory access",
            Self::HeapMisaligned => "misaligned heap",
            Self::TableAccessOutOfBounds => "undefined element: out of bounds table access",
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
            "stk_ovf" => Ok(Self::StackOverflow),
            "heap_get_oob" => Ok(Self::HeapAccessOutOfBounds),
            "heap_misaligned" => Ok(Self::HeapMisaligned),
            "table_get_oob" => Ok(Self::TableAccessOutOfBounds),
            "icall_null" => Ok(Self::IndirectCallToNull),
            "bad_sig" => Ok(Self::BadSignature),
            "int_ovf" => Ok(Self::IntegerOverflow),
            "int_divz" => Ok(Self::IntegerDivisionByZero),
            "bad_toint" => Ok(Self::BadConversionToInteger),
            "unreachable" => Ok(Self::UnreachableCodeReached),
            "unalign_atom" => Ok(Self::UnalignedAtomic),
            _ => Err(()),
        }
    }
}

// TODO: OnCalledAction is needed for asyncify. It will be refactored with https://github.com/wasmerio/wasmer/issues/3451
/// After the stack is unwound via asyncify what
/// should the call loop do next
#[derive(Debug)]
pub enum OnCalledAction {
    /// Will call the function again
    InvokeAgain,
    /// Will return the result of the invocation
    Finish,
    /// Traps with an error
    Trap(Box<dyn std::error::Error + Send + Sync>),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Everything but user-defined codes.
    const CODES: [TrapCode; 11] = [
        TrapCode::StackOverflow,
        TrapCode::HeapAccessOutOfBounds,
        TrapCode::HeapMisaligned,
        TrapCode::TableAccessOutOfBounds,
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
