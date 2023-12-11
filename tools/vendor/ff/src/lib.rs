//! This crate provides traits for working with finite fields.

// Catch documentation errors caused by code changes.
#![no_std]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(rustdoc::broken_intra_doc_links)]
#![forbid(unsafe_code)]

#[cfg(feature = "alloc")]
extern crate alloc;

mod batch;
pub use batch::*;

pub mod helpers;

#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub use ff_derive::PrimeField;

#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
pub use bitvec::view::BitViewSized;

#[cfg(feature = "bits")]
use bitvec::{array::BitArray, order::Lsb0};

use core::fmt;
use core::iter::{Product, Sum};
use core::ops::{Add, AddAssign, Mul, MulAssign, Neg, Sub, SubAssign};

use rand_core::RngCore;
use subtle::{Choice, ConditionallySelectable, ConstantTimeEq, CtOption};

/// Bit representation of a field element.
#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
pub type FieldBits<V> = BitArray<V, Lsb0>;

/// This trait represents an element of a field.
pub trait Field:
    Sized
    + Eq
    + Copy
    + Clone
    + Default
    + Send
    + Sync
    + fmt::Debug
    + 'static
    + ConditionallySelectable
    + ConstantTimeEq
    + Neg<Output = Self>
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Sum
    + Product
    + for<'a> Add<&'a Self, Output = Self>
    + for<'a> Sub<&'a Self, Output = Self>
    + for<'a> Mul<&'a Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + for<'a> Product<&'a Self>
    + AddAssign
    + SubAssign
    + MulAssign
    + for<'a> AddAssign<&'a Self>
    + for<'a> SubAssign<&'a Self>
    + for<'a> MulAssign<&'a Self>
{
    /// The zero element of the field, the additive identity.
    const ZERO: Self;

    /// The one element of the field, the multiplicative identity.
    const ONE: Self;

    /// Returns an element chosen uniformly at random using a user-provided RNG.
    fn random(rng: impl RngCore) -> Self;

    /// Returns true iff this element is zero.
    fn is_zero(&self) -> Choice {
        self.ct_eq(&Self::ZERO)
    }

    /// Returns true iff this element is zero.
    ///
    /// # Security
    ///
    /// This method provides **no** constant-time guarantees. Implementors of the
    /// `Field` trait **may** optimise this method using non-constant-time logic.
    fn is_zero_vartime(&self) -> bool {
        self.is_zero().into()
    }

    /// Squares this element.
    #[must_use]
    fn square(&self) -> Self;

    /// Cubes this element.
    #[must_use]
    fn cube(&self) -> Self {
        self.square() * self
    }

    /// Doubles this element.
    #[must_use]
    fn double(&self) -> Self;

    /// Computes the multiplicative inverse of this element,
    /// failing if the element is zero.
    fn invert(&self) -> CtOption<Self>;

    /// Computes:
    ///
    /// - $(\textsf{true}, \sqrt{\textsf{num}/\textsf{div}})$, if $\textsf{num}$ and
    ///   $\textsf{div}$ are nonzero and $\textsf{num}/\textsf{div}$ is a square in the
    ///   field;
    /// - $(\textsf{true}, 0)$, if $\textsf{num}$ is zero;
    /// - $(\textsf{false}, 0)$, if $\textsf{num}$ is nonzero and $\textsf{div}$ is zero;
    /// - $(\textsf{false}, \sqrt{G_S \cdot \textsf{num}/\textsf{div}})$, if
    ///   $\textsf{num}$ and $\textsf{div}$ are nonzero and $\textsf{num}/\textsf{div}$ is
    ///   a nonsquare in the field;
    ///
    /// where $G_S$ is a non-square.
    ///
    /// # Warnings
    ///
    /// - The choice of root from `sqrt` is unspecified.
    /// - The value of $G_S$ is unspecified, and cannot be assumed to have any specific
    ///   value in a generic context.
    fn sqrt_ratio(num: &Self, div: &Self) -> (Choice, Self);

    /// Equivalent to `Self::sqrt_ratio(self, one())`.
    ///
    /// The provided method is implemented in terms of [`Self::sqrt_ratio`].
    fn sqrt_alt(&self) -> (Choice, Self) {
        Self::sqrt_ratio(self, &Self::ONE)
    }

    /// Returns the square root of the field element, if it is
    /// quadratic residue.
    ///
    /// The provided method is implemented in terms of [`Self::sqrt_ratio`].
    fn sqrt(&self) -> CtOption<Self> {
        let (is_square, res) = Self::sqrt_ratio(self, &Self::ONE);
        CtOption::new(res, is_square)
    }

    /// Exponentiates `self` by `exp`, where `exp` is a little-endian order integer
    /// exponent.
    ///
    /// # Guarantees
    ///
    /// This operation is constant time with respect to `self`, for all exponents with the
    /// same number of digits (`exp.as_ref().len()`). It is variable time with respect to
    /// the number of digits in the exponent.
    fn pow<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::ONE;
        for e in exp.as_ref().iter().rev() {
            for i in (0..64).rev() {
                res = res.square();
                let mut tmp = res;
                tmp *= self;
                res.conditional_assign(&tmp, (((*e >> i) & 1) as u8).into());
            }
        }
        res
    }

    /// Exponentiates `self` by `exp`, where `exp` is a little-endian order integer
    /// exponent.
    ///
    /// # Guarantees
    ///
    /// **This operation is variable time with respect to `self`, for all exponent.** If
    /// the exponent is fixed, this operation is effectively constant time. However, for
    /// stronger constant-time guarantees, [`Field::pow`] should be used.
    fn pow_vartime<S: AsRef<[u64]>>(&self, exp: S) -> Self {
        let mut res = Self::ONE;
        for e in exp.as_ref().iter().rev() {
            for i in (0..64).rev() {
                res = res.square();

                if ((*e >> i) & 1) == 1 {
                    res.mul_assign(self);
                }
            }
        }

        res
    }
}

/// This represents an element of a non-binary prime field.
pub trait PrimeField: Field + From<u64> {
    /// The prime field can be converted back and forth into this binary
    /// representation.
    type Repr: Copy + Default + Send + Sync + 'static + AsRef<[u8]> + AsMut<[u8]>;

    /// Interpret a string of numbers as a (congruent) prime field element.
    /// Does not accept unnecessary leading zeroes or a blank string.
    ///
    /// # Security
    ///
    /// This method provides **no** constant-time guarantees.
    fn from_str_vartime(s: &str) -> Option<Self> {
        if s.is_empty() {
            return None;
        }

        if s == "0" {
            return Some(Self::ZERO);
        }

        let mut res = Self::ZERO;

        let ten = Self::from(10);

        let mut first_digit = true;

        for c in s.chars() {
            match c.to_digit(10) {
                Some(c) => {
                    if first_digit {
                        if c == 0 {
                            return None;
                        }

                        first_digit = false;
                    }

                    res.mul_assign(&ten);
                    res.add_assign(&Self::from(u64::from(c)));
                }
                None => {
                    return None;
                }
            }
        }

        Some(res)
    }

    /// Obtains a field element congruent to the integer `v`.
    ///
    /// For fields where `Self::CAPACITY >= 128`, this is injective and will produce a
    /// unique field element.
    ///
    /// For fields where `Self::CAPACITY < 128`, this is surjective; some field elements
    /// will be produced by multiple values of `v`.
    ///
    /// If you want to deterministically sample a field element representing a value, use
    /// [`FromUniformBytes`] instead.
    fn from_u128(v: u128) -> Self {
        let lower = v as u64;
        let upper = (v >> 64) as u64;
        let mut tmp = Self::from(upper);
        for _ in 0..64 {
            tmp = tmp.double();
        }
        tmp + Self::from(lower)
    }

    /// Attempts to convert a byte representation of a field element into an element of
    /// this prime field, failing if the input is not canonical (is not smaller than the
    /// field's modulus).
    ///
    /// The byte representation is interpreted with the same endianness as elements
    /// returned by [`PrimeField::to_repr`].
    fn from_repr(repr: Self::Repr) -> CtOption<Self>;

    /// Attempts to convert a byte representation of a field element into an element of
    /// this prime field, failing if the input is not canonical (is not smaller than the
    /// field's modulus).
    ///
    /// The byte representation is interpreted with the same endianness as elements
    /// returned by [`PrimeField::to_repr`].
    ///
    /// # Security
    ///
    /// This method provides **no** constant-time guarantees. Implementors of the
    /// `PrimeField` trait **may** optimise this method using non-constant-time logic.
    fn from_repr_vartime(repr: Self::Repr) -> Option<Self> {
        Self::from_repr(repr).into()
    }

    /// Converts an element of the prime field into the standard byte representation for
    /// this field.
    ///
    /// The endianness of the byte representation is implementation-specific. Generic
    /// encodings of field elements should be treated as opaque.
    fn to_repr(&self) -> Self::Repr;

    /// Returns true iff this element is odd.
    fn is_odd(&self) -> Choice;

    /// Returns true iff this element is even.
    #[inline(always)]
    fn is_even(&self) -> Choice {
        !self.is_odd()
    }

    /// Modulus of the field written as a string for debugging purposes.
    ///
    /// The encoding of the modulus is implementation-specific. Generic users of the
    /// `PrimeField` trait should treat this string as opaque.
    const MODULUS: &'static str;

    /// How many bits are needed to represent an element of this field.
    const NUM_BITS: u32;

    /// How many bits of information can be reliably stored in the field element.
    ///
    /// This is usually `Self::NUM_BITS - 1`.
    const CAPACITY: u32;

    /// Inverse of $2$ in the field.
    const TWO_INV: Self;

    /// A fixed multiplicative generator of `modulus - 1` order. This element must also be
    /// a quadratic nonresidue.
    ///
    /// It can be calculated using [SageMath] as `GF(modulus).primitive_element()`.
    ///
    /// Implementations of this trait MUST ensure that this is the generator used to
    /// derive `Self::ROOT_OF_UNITY`.
    ///
    /// [SageMath]: https://www.sagemath.org/
    const MULTIPLICATIVE_GENERATOR: Self;

    /// An integer `s` satisfying the equation `2^s * t = modulus - 1` with `t` odd.
    ///
    /// This is the number of leading zero bits in the little-endian bit representation of
    /// `modulus - 1`.
    const S: u32;

    /// The `2^s` root of unity.
    ///
    /// It can be calculated by exponentiating `Self::MULTIPLICATIVE_GENERATOR` by `t`,
    /// where `t = (modulus - 1) >> Self::S`.
    const ROOT_OF_UNITY: Self;

    /// Inverse of [`Self::ROOT_OF_UNITY`].
    const ROOT_OF_UNITY_INV: Self;

    /// Generator of the `t-order` multiplicative subgroup.
    ///
    /// It can be calculated by exponentiating [`Self::MULTIPLICATIVE_GENERATOR`] by `2^s`,
    /// where `s` is [`Self::S`].
    const DELTA: Self;
}

/// The subset of prime-order fields such that `(modulus - 1)` is divisible by `N`.
///
/// If `N` is prime, there will be `N - 1` valid choices of [`Self::ZETA`]. Similarly to
/// [`PrimeField::MULTIPLICATIVE_GENERATOR`], the specific choice does not matter, as long
/// as the choice is consistent across all uses of the field.
pub trait WithSmallOrderMulGroup<const N: u8>: PrimeField {
    /// A field element of small multiplicative order $N$.
    ///
    /// The presense of this element allows you to perform (certain types of)
    /// endomorphisms on some elliptic curves.
    ///
    /// It can be calculated using [SageMath] as
    /// `GF(modulus).primitive_element() ^ ((modulus - 1) // N)`.
    /// Choosing the element of order $N$ that is smallest, when considered
    /// as an integer, may help to ensure consistency.
    ///
    /// [SageMath]: https://www.sagemath.org/
    const ZETA: Self;
}

/// Trait for constructing a [`PrimeField`] element from a fixed-length uniform byte
/// array.
///
/// "Uniform" means that the byte array's contents must be indistinguishable from the
/// [discrete uniform distribution]. Suitable byte arrays can be obtained:
/// - from a cryptographically-secure randomness source (which makes this constructor
///   equivalent to [`Field::random`]).
/// - from a cryptographic hash function output, which enables a "random" field element to
///   be selected deterministically. This is the primary use case for `FromUniformBytes`.
///
/// The length `N` of the byte array is chosen by the trait implementer such that the loss
/// of uniformity in the mapping from byte arrays to field elements is cryptographically
/// negligible.
///
/// [discrete uniform distribution]: https://en.wikipedia.org/wiki/Discrete_uniform_distribution
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "derive")] {
/// # // Fake this so we don't actually need a dev-dependency on bls12_381.
/// # mod bls12_381 {
/// #     use ff::{Field, PrimeField};
/// #
/// #     #[derive(PrimeField)]
/// #     #[PrimeFieldModulus = "52435875175126190479447740508185965837690552500527637822603658699938581184513"]
/// #     #[PrimeFieldGenerator = "7"]
/// #     #[PrimeFieldReprEndianness = "little"]
/// #     pub struct Scalar([u64; 4]);
/// #
/// #     impl ff::FromUniformBytes<64> for Scalar {
/// #         fn from_uniform_bytes(_bytes: &[u8; 64]) -> Self {
/// #             // Fake impl for doctest
/// #             Scalar::ONE
/// #         }
/// #     }
/// # }
/// #
/// use blake2b_simd::blake2b;
/// use bls12_381::Scalar;
/// use ff::FromUniformBytes;
///
/// // `bls12_381::Scalar` implements `FromUniformBytes<64>`, and BLAKE2b (by default)
/// // produces a 64-byte hash.
/// let hash = blake2b(b"Some message");
/// let val = Scalar::from_uniform_bytes(hash.as_array());
/// # }
/// ```
///
/// # Implementing `FromUniformBytes`
///
/// [`Self::from_uniform_bytes`] should always be implemented by interpreting the provided
/// byte array as the little endian unsigned encoding of an integer, and then reducing that
/// integer modulo the field modulus.
///
/// For security, `N` must be chosen so that `N * 8 >= Self::NUM_BITS + 128`. A larger
/// value of `N` may be chosen for convenience; for example, for a field with a 255-bit
/// modulus, `N = 64` is convenient as it matches the output length of several common
/// cryptographic hash functions (such as SHA-512 and BLAKE2b).
///
/// ## Trait design
///
/// This trait exists because `PrimeField::from_uniform_bytes([u8; N])` cannot currently
/// exist (trait methods cannot use associated constants in the const positions of their
/// type signature, and we do not want `PrimeField` to require a generic const parameter).
/// However, this has the side-effect that `FromUniformBytes` can be implemented multiple
/// times for different values of `N`. Most implementations of [`PrimeField`] should only
/// need to implement `FromUniformBytes` trait for one value of `N` (chosen following the
/// above considerations); if you find yourself needing to implement it multiple times,
/// please [let us know about your use case](https://github.com/zkcrypto/ff/issues/new) so
/// we can take it into consideration for future evolutions of the `ff` traits.
pub trait FromUniformBytes<const N: usize>: PrimeField {
    /// Returns a field element that is congruent to the provided little endian unsigned
    /// byte representation of an integer.
    fn from_uniform_bytes(bytes: &[u8; N]) -> Self;
}

/// This represents the bits of an element of a prime field.
#[cfg(feature = "bits")]
#[cfg_attr(docsrs, doc(cfg(feature = "bits")))]
pub trait PrimeFieldBits: PrimeField {
    /// The backing store for a bit representation of a prime field element.
    type ReprBits: BitViewSized + Send + Sync;

    /// Converts an element of the prime field into a little-endian sequence of bits.
    fn to_le_bits(&self) -> FieldBits<Self::ReprBits>;

    /// Returns the bits of the field characteristic (the modulus) in little-endian order.
    fn char_le_bits() -> FieldBits<Self::ReprBits>;
}

/// Functions and re-exported crates used by the [`PrimeField`] derive macro.
#[cfg(feature = "derive")]
#[cfg_attr(docsrs, doc(cfg(feature = "derive")))]
pub mod derive {
    pub use crate::arith_impl::*;

    pub use {byteorder, rand_core, subtle};

    #[cfg(feature = "bits")]
    pub use bitvec;
}

#[cfg(feature = "derive")]
mod arith_impl {
    /// Computes `a - (b + borrow)`, returning the result and the new borrow.
    #[inline(always)]
    pub const fn sbb(a: u64, b: u64, borrow: u64) -> (u64, u64) {
        let ret = (a as u128).wrapping_sub((b as u128) + ((borrow >> 63) as u128));
        (ret as u64, (ret >> 64) as u64)
    }

    /// Computes `a + b + carry`, returning the result and the new carry over.
    #[inline(always)]
    pub const fn adc(a: u64, b: u64, carry: u64) -> (u64, u64) {
        let ret = (a as u128) + (b as u128) + (carry as u128);
        (ret as u64, (ret >> 64) as u64)
    }

    /// Computes `a + (b * c) + carry`, returning the result and the new carry over.
    #[inline(always)]
    pub const fn mac(a: u64, b: u64, c: u64, carry: u64) -> (u64, u64) {
        let ret = (a as u128) + ((b as u128) * (c as u128)) + (carry as u128);
        (ret as u64, (ret >> 64) as u64)
    }
}
