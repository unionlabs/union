//! Trap codes describing the reason for a trap.

use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
#[cfg(feature = "enable-serde")]
use serde::{Deserialize, Serialize};

/// A trap code describing the reason for a trap.
///
/// All trap instructions have an explicit trap code.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
#[cfg_attr(feature = "enable-serde", derive(Serialize, Deserialize))]
pub enum TrapCode {
    /// The current stack space was exhausted.
    StackOverflow,

    /// A `heap_addr` instruction detected an out-of-bounds error.
    ///
    /// Note that not all out-of-bounds heap accesses are reported this way;
    /// some are detected by a segmentation fault on the heap unmapped or
    /// offset-guard pages.
    HeapOutOfBounds,

    /// A wasm atomic operation was presented with a not-naturally-aligned linear-memory address.
    HeapMisaligned,

    /// A `table_addr` instruction detected an out-of-bounds error.
    TableOutOfBounds,

    /// Indirect call to a null table entry.
    IndirectCallToNull,

    /// Signature mismatch on indirect call.
    BadSignature,

    /// An integer arithmetic operation caused an overflow.
    IntegerOverflow,

    /// An integer division by zero.
    IntegerDivisionByZero,

    /// Failed float-to-int conversion.
    BadConversionToInteger,

    /// Code that was supposed to have been unreachable was reached.
    UnreachableCodeReached,

    /// Execution has potentially run too long and may be interrupted.
    /// This trap is resumable.
    Interrupt,

    /// A user-defined trap code.
    User(u16),
}

impl Display for TrapCode {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        use self::TrapCode::*;
        let identifier = match *self {
            StackOverflow => "stk_ovf",
            HeapOutOfBounds => "heap_oob",
            HeapMisaligned => "heap_misaligned",
            TableOutOfBounds => "table_oob",
            IndirectCallToNull => "icall_null",
            BadSignature => "bad_sig",
            IntegerOverflow => "int_ovf",
            IntegerDivisionByZero => "int_divz",
            BadConversionToInteger => "bad_toint",
            UnreachableCodeReached => "unreachable",
            Interrupt => "interrupt",
            User(x) => return write!(f, "user{}", x),
        };
        f.write_str(identifier)
    }
}

impl FromStr for TrapCode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use self::TrapCode::*;
        match s {
            "stk_ovf" => Ok(StackOverflow),
            "heap_oob" => Ok(HeapOutOfBounds),
            "heap_misaligned" => Ok(HeapMisaligned),
            "table_oob" => Ok(TableOutOfBounds),
            "icall_null" => Ok(IndirectCallToNull),
            "bad_sig" => Ok(BadSignature),
            "int_ovf" => Ok(IntegerOverflow),
            "int_divz" => Ok(IntegerDivisionByZero),
            "bad_toint" => Ok(BadConversionToInteger),
            "unreachable" => Ok(UnreachableCodeReached),
            "interrupt" => Ok(Interrupt),
            _ if s.starts_with("user") => s[4..].parse().map(User).map_err(|_| ()),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;

    // Everything but user-defined codes.
    const CODES: [TrapCode; 11] = [
        TrapCode::StackOverflow,
        TrapCode::HeapOutOfBounds,
        TrapCode::HeapMisaligned,
        TrapCode::TableOutOfBounds,
        TrapCode::IndirectCallToNull,
        TrapCode::BadSignature,
        TrapCode::IntegerOverflow,
        TrapCode::IntegerDivisionByZero,
        TrapCode::BadConversionToInteger,
        TrapCode::UnreachableCodeReached,
        TrapCode::Interrupt,
    ];

    #[test]
    fn display() {
        for r in &CODES {
            let tc = *r;
            assert_eq!(tc.to_string().parse(), Ok(tc));
        }
        assert_eq!("bogus".parse::<TrapCode>(), Err(()));

        assert_eq!(TrapCode::User(17).to_string(), "user17");
        assert_eq!("user22".parse(), Ok(TrapCode::User(22)));
        assert_eq!("user".parse::<TrapCode>(), Err(()));
        assert_eq!("user-1".parse::<TrapCode>(), Err(()));
        assert_eq!("users".parse::<TrapCode>(), Err(()));
    }
}
