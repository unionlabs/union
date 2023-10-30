use core::fmt::{self, Debug, Display, Formatter};
use core::num::IntErrorKind;

/// The error type that is returned when parsing an integer from an invalid source.
///
/// This error can occur when the `from_str_radix` or [`FromStr::from_str`](https://doc.rust-lang.org/core/str/trait.FromStr.html#tymethod.from_str) methods of e.g. [`BUint`](crate::BUint::from_str_radix) are called with an invalid input string.
#[derive(PartialEq, Eq, Clone)]
pub struct ParseIntError {
    pub(crate) kind: IntErrorKind,
}

impl ParseIntError {
    /// Returns the enum [`IntErrorKind`](https://doc.rust-lang.org/core/num/enum.IntErrorKind.html), which shows the reason that the parsing input was invalid.
    pub const fn kind(&self) -> &IntErrorKind {
        &self.kind
    }

    pub(crate) const fn description(&self) -> &str {
        match &self.kind {
            IntErrorKind::Empty => "attempt to parse integer from empty string",
            IntErrorKind::InvalidDigit => {
                "attempt to parse integer from string containing invalid digit"
            }
            IntErrorKind::PosOverflow => {
                "attempt to parse integer too large to be represented by the target type"
            }
            IntErrorKind::NegOverflow => {
                "attempt to parse integer too small to be represented by the target type"
            }
            IntErrorKind::Zero => {
                "attempt to parse the integer `0` which cannot be represented by the target type"
            }
            _ => panic!("unsupported `IntErrorKind` variant"), // necessary as `IntErrorKind` is non-exhaustive
        }
    }
}

impl Display for ParseIntError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{} {}", super::err_prefix!(), self.description())
    }
}

impl Debug for ParseIntError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Display::fmt(&self, f)
    }
}
