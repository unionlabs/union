//! Generic scalar type with primitive functionality.

use crate::{
    bigint::{prelude::*, Limb, NonZero},
    scalar::FromUintUnchecked,
    scalar::IsHigh,
    Curve, Error, FieldBytes, FieldBytesEncoding, Result,
};
use base16ct::HexDisplay;
use core::{
    cmp::Ordering,
    fmt,
    ops::{Add, AddAssign, Neg, ShrAssign, Sub, SubAssign},
    str,
};
use generic_array::{typenum::Unsigned, GenericArray};
use rand_core::CryptoRngCore;
use subtle::{
    Choice, ConditionallySelectable, ConstantTimeEq, ConstantTimeGreater, ConstantTimeLess,
    CtOption,
};
use zeroize::DefaultIsZeroes;

#[cfg(feature = "arithmetic")]
use super::{CurveArithmetic, Scalar};

#[cfg(feature = "serde")]
use serdect::serde::{de, ser, Deserialize, Serialize};

/// Generic scalar type with primitive functionality.
///
/// This type provides a baseline level of scalar arithmetic functionality
/// which is always available for all curves, regardless of if they implement
/// any arithmetic traits.
///
/// # `serde` support
///
/// When the optional `serde` feature of this create is enabled, [`Serialize`]
/// and [`Deserialize`] impls are provided for this type.
///
/// The serialization is a fixed-width big endian encoding. When used with
/// textual formats, the binary data is encoded as hexadecimal.
// TODO(tarcieri): use `crypto-bigint`'s `Residue` type, expose more functionality?
#[derive(Copy, Clone, Debug, Default)]
pub struct ScalarPrimitive<C: Curve> {
    /// Inner unsigned integer type.
    inner: C::Uint,
}

impl<C> ScalarPrimitive<C>
where
    C: Curve,
{
    /// Zero scalar.
    pub const ZERO: Self = Self {
        inner: C::Uint::ZERO,
    };

    /// Multiplicative identity.
    pub const ONE: Self = Self {
        inner: C::Uint::ONE,
    };

    /// Scalar modulus.
    pub const MODULUS: C::Uint = C::ORDER;

    /// Generate a random [`ScalarPrimitive`].
    pub fn random(rng: &mut impl CryptoRngCore) -> Self {
        Self {
            inner: C::Uint::random_mod(rng, &NonZero::new(Self::MODULUS).unwrap()),
        }
    }

    /// Create a new scalar from [`Curve::Uint`].
    pub fn new(uint: C::Uint) -> CtOption<Self> {
        CtOption::new(Self { inner: uint }, uint.ct_lt(&Self::MODULUS))
    }

    /// Decode [`ScalarPrimitive`] from a serialized field element
    pub fn from_bytes(bytes: &FieldBytes<C>) -> CtOption<Self> {
        Self::new(C::Uint::decode_field_bytes(bytes))
    }

    /// Decode [`ScalarPrimitive`] from a big endian byte slice.
    pub fn from_slice(slice: &[u8]) -> Result<Self> {
        if slice.len() == C::FieldBytesSize::USIZE {
            Option::from(Self::from_bytes(GenericArray::from_slice(slice))).ok_or(Error)
        } else {
            Err(Error)
        }
    }

    /// Borrow the inner `C::Uint`.
    pub fn as_uint(&self) -> &C::Uint {
        &self.inner
    }

    /// Borrow the inner limbs as a slice.
    pub fn as_limbs(&self) -> &[Limb] {
        self.inner.as_ref()
    }

    /// Is this [`ScalarPrimitive`] value equal to zero?
    pub fn is_zero(&self) -> Choice {
        self.inner.is_zero()
    }

    /// Is this [`ScalarPrimitive`] value even?
    pub fn is_even(&self) -> Choice {
        self.inner.is_even()
    }

    /// Is this [`ScalarPrimitive`] value odd?
    pub fn is_odd(&self) -> Choice {
        self.inner.is_odd()
    }

    /// Encode [`ScalarPrimitive`] as a serialized field element.
    pub fn to_bytes(&self) -> FieldBytes<C> {
        self.inner.encode_field_bytes()
    }

    /// Convert to a `C::Uint`.
    pub fn to_uint(&self) -> C::Uint {
        self.inner
    }
}

impl<C> FromUintUnchecked for ScalarPrimitive<C>
where
    C: Curve,
{
    type Uint = C::Uint;

    fn from_uint_unchecked(uint: C::Uint) -> Self {
        Self { inner: uint }
    }
}

#[cfg(feature = "arithmetic")]
impl<C> ScalarPrimitive<C>
where
    C: CurveArithmetic,
{
    /// Convert [`ScalarPrimitive`] into a given curve's scalar type.
    pub(super) fn to_scalar(self) -> Scalar<C> {
        Scalar::<C>::from_uint_unchecked(self.inner)
    }
}

// TODO(tarcieri): better encapsulate this?
impl<C> AsRef<[Limb]> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn as_ref(&self) -> &[Limb] {
        self.as_limbs()
    }
}

impl<C> ConditionallySelectable for ScalarPrimitive<C>
where
    C: Curve,
{
    fn conditional_select(a: &Self, b: &Self, choice: Choice) -> Self {
        Self {
            inner: C::Uint::conditional_select(&a.inner, &b.inner, choice),
        }
    }
}

impl<C> ConstantTimeEq for ScalarPrimitive<C>
where
    C: Curve,
{
    fn ct_eq(&self, other: &Self) -> Choice {
        self.inner.ct_eq(&other.inner)
    }
}

impl<C> ConstantTimeLess for ScalarPrimitive<C>
where
    C: Curve,
{
    fn ct_lt(&self, other: &Self) -> Choice {
        self.inner.ct_lt(&other.inner)
    }
}

impl<C> ConstantTimeGreater for ScalarPrimitive<C>
where
    C: Curve,
{
    fn ct_gt(&self, other: &Self) -> Choice {
        self.inner.ct_gt(&other.inner)
    }
}

impl<C: Curve> DefaultIsZeroes for ScalarPrimitive<C> {}

impl<C: Curve> Eq for ScalarPrimitive<C> {}

impl<C> PartialEq for ScalarPrimitive<C>
where
    C: Curve,
{
    fn eq(&self, other: &Self) -> bool {
        self.ct_eq(other).into()
    }
}

impl<C> PartialOrd for ScalarPrimitive<C>
where
    C: Curve,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C> Ord for ScalarPrimitive<C>
where
    C: Curve,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.inner.cmp(&other.inner)
    }
}

impl<C> From<u64> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn from(n: u64) -> Self {
        Self {
            inner: C::Uint::from(n),
        }
    }
}

impl<C> Add<ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = Self;

    fn add(self, other: Self) -> Self {
        self.add(&other)
    }
}

impl<C> Add<&ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = Self;

    fn add(self, other: &Self) -> Self {
        Self {
            inner: self.inner.add_mod(&other.inner, &Self::MODULUS),
        }
    }
}

impl<C> AddAssign<ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn add_assign(&mut self, other: Self) {
        *self = *self + other;
    }
}

impl<C> AddAssign<&ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn add_assign(&mut self, other: &Self) {
        *self = *self + other;
    }
}

impl<C> Sub<ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        self.sub(&other)
    }
}

impl<C> Sub<&ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = Self;

    fn sub(self, other: &Self) -> Self {
        Self {
            inner: self.inner.sub_mod(&other.inner, &Self::MODULUS),
        }
    }
}

impl<C> SubAssign<ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn sub_assign(&mut self, other: Self) {
        *self = *self - other;
    }
}

impl<C> SubAssign<&ScalarPrimitive<C>> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn sub_assign(&mut self, other: &Self) {
        *self = *self - other;
    }
}

impl<C> Neg for ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = Self;

    fn neg(self) -> Self {
        Self {
            inner: self.inner.neg_mod(&Self::MODULUS),
        }
    }
}

impl<C> Neg for &ScalarPrimitive<C>
where
    C: Curve,
{
    type Output = ScalarPrimitive<C>;

    fn neg(self) -> ScalarPrimitive<C> {
        -*self
    }
}

impl<C> ShrAssign<usize> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn shr_assign(&mut self, rhs: usize) {
        self.inner >>= rhs;
    }
}

impl<C> IsHigh for ScalarPrimitive<C>
where
    C: Curve,
{
    fn is_high(&self) -> Choice {
        let n_2 = C::ORDER >> 1;
        self.inner.ct_gt(&n_2)
    }
}

impl<C> fmt::Display for ScalarPrimitive<C>
where
    C: Curve,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:X}")
    }
}

impl<C> fmt::LowerHex for ScalarPrimitive<C>
where
    C: Curve,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:x}", HexDisplay(&self.to_bytes()))
    }
}

impl<C> fmt::UpperHex for ScalarPrimitive<C>
where
    C: Curve,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:X}", HexDisplay(&self.to_bytes()))
    }
}

impl<C> str::FromStr for ScalarPrimitive<C>
where
    C: Curve,
{
    type Err = Error;

    fn from_str(hex: &str) -> Result<Self> {
        let mut bytes = FieldBytes::<C>::default();
        base16ct::lower::decode(hex, &mut bytes)?;
        Self::from_slice(&bytes)
    }
}

#[cfg(feature = "serde")]
impl<C> Serialize for ScalarPrimitive<C>
where
    C: Curve,
{
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serdect::array::serialize_hex_upper_or_bin(&self.to_bytes(), serializer)
    }
}

#[cfg(feature = "serde")]
impl<'de, C> Deserialize<'de> for ScalarPrimitive<C>
where
    C: Curve,
{
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        let mut bytes = FieldBytes::<C>::default();
        serdect::array::deserialize_hex_or_bin(&mut bytes, deserializer)?;
        Self::from_slice(&bytes).map_err(|_| de::Error::custom("scalar out of range"))
    }
}
