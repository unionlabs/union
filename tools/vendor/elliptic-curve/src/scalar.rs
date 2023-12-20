//! Scalar types.

#[cfg(feature = "arithmetic")]
mod blinded;
#[cfg(feature = "arithmetic")]
mod nonzero;
mod primitive;

pub use self::primitive::ScalarPrimitive;
#[cfg(feature = "arithmetic")]
pub use self::{blinded::BlindedScalar, nonzero::NonZeroScalar};

use crypto_bigint::Integer;
use subtle::Choice;

#[cfg(feature = "arithmetic")]
use crate::CurveArithmetic;

/// Scalar field element for a particular elliptic curve.
#[cfg(feature = "arithmetic")]
pub type Scalar<C> = <C as CurveArithmetic>::Scalar;

/// Bit representation of a scalar field element of a given curve.
#[cfg(feature = "bits")]
pub type ScalarBits<C> = ff::FieldBits<<Scalar<C> as ff::PrimeFieldBits>::ReprBits>;

/// Instantiate a scalar from an unsigned integer without checking for overflow.
pub trait FromUintUnchecked {
    /// Unsigned integer type (i.e. `Curve::Uint`)
    type Uint: Integer;

    /// Instantiate scalar from an unsigned integer without checking
    /// whether the value overflows the field modulus.
    ///
    /// ⚠️ WARNING!
    ///
    /// Incorrectly used this can lead to mathematically invalid results,
    /// which can lead to potential security vulnerabilities.
    ///
    /// Use with care!
    fn from_uint_unchecked(uint: Self::Uint) -> Self;
}

/// Is this scalar greater than n / 2?
///
/// # Returns
///
/// - For scalars 0 through n / 2: `Choice::from(0)`
/// - For scalars (n / 2) + 1 through n - 1: `Choice::from(1)`
pub trait IsHigh {
    /// Is this scalar greater than or equal to n / 2?
    fn is_high(&self) -> Choice;
}
