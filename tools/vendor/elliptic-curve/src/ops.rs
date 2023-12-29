//! Traits for arithmetic operations on elliptic curve field elements.

pub use core::ops::{Add, AddAssign, Mul, Neg, Shr, ShrAssign, Sub, SubAssign};

use crypto_bigint::Integer;
use group::Group;

/// Perform an inversion on a field element (i.e. base field element or scalar)
pub trait Invert {
    /// Field element type
    type Output;

    /// Invert a field element.
    fn invert(&self) -> Self::Output;

    /// Invert a field element in variable time.
    ///
    /// ⚠️ WARNING!
    ///
    /// This method should not be used with secret values, as its variable-time
    /// operation can potentially leak secrets through sidechannels.
    fn invert_vartime(&self) -> Self::Output {
        // Fall back on constant-time implementation by default.
        self.invert()
    }
}

/// Linear combination.
///
/// This trait enables crates to provide an optimized implementation of
/// linear combinations (e.g. Shamir's Trick), or otherwise provides a default
/// non-optimized implementation.
// TODO(tarcieri): replace this with a trait from the `group` crate? (see zkcrypto/group#25)
pub trait LinearCombination: Group {
    /// Calculates `x * k + y * l`.
    fn lincomb(x: &Self, k: &Self::Scalar, y: &Self, l: &Self::Scalar) -> Self {
        (*x * k) + (*y * l)
    }
}

/// Multiplication by the generator.
///
/// May use optimizations (e.g. precomputed tables) when available.
// TODO(tarcieri): replace this with `Group::mul_by_generator``? (see zkcrypto/group#44)
pub trait MulByGenerator: Group {
    /// Multiply by the generator of the prime-order subgroup.
    #[must_use]
    fn mul_by_generator(scalar: &Self::Scalar) -> Self {
        Self::generator() * scalar
    }
}

/// Modular reduction.
pub trait Reduce<Uint: Integer>: Sized {
    /// Bytes used as input to [`Reduce::reduce_bytes`].
    type Bytes: AsRef<[u8]>;

    /// Perform a modular reduction, returning a field element.
    fn reduce(n: Uint) -> Self;

    /// Interpret the given bytes as an integer and perform a modular reduction.
    fn reduce_bytes(bytes: &Self::Bytes) -> Self;
}

/// Modular reduction to a non-zero output.
///
/// This trait is primarily intended for use by curve implementations such
/// as the `k256` and `p256` crates.
///
/// End users should use the [`Reduce`] impl on
/// [`NonZeroScalar`][`crate::NonZeroScalar`] instead.
pub trait ReduceNonZero<Uint: Integer>: Reduce<Uint> + Sized {
    /// Perform a modular reduction, returning a field element.
    fn reduce_nonzero(n: Uint) -> Self;

    /// Interpret the given bytes as an integer and perform a modular reduction
    /// to a non-zero output.
    fn reduce_nonzero_bytes(bytes: &Self::Bytes) -> Self;
}
