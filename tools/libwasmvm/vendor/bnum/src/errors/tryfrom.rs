use core::fmt::{self, Display, Formatter};

/// The error type that is returned when a failed conversion from an integer occurs.
///
/// This error will occur for example when using the [`TryFrom`](https://doc.rust-lang.org/core/convert/trait.TryFrom.html) trait to convert a negative [`i32`] to a [`BUint`](crate::BUint).
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct TryFromIntError(pub(crate) ());

const ERROR_MESSAGE: &str = concat!(
    super::err_prefix!(),
    "out of range integral type conversion attempted"
);

impl Display for TryFromIntError {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", ERROR_MESSAGE)
    }
}
