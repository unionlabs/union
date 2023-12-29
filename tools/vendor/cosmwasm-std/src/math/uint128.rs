use core::fmt::{self};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign,
    Sub, SubAssign,
};
use core::str::FromStr;
use std::ops::Not;

use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{
    CheckedMultiplyFractionError, CheckedMultiplyRatioError, DivideByZeroError, OverflowError,
    OverflowOperation, StdError,
};
use crate::{
    forward_ref_partial_eq, impl_mul_fraction, Fraction, Int128, Int256, Int512, Int64, Uint256,
    Uint64,
};

use super::conversion::forward_try_from;
use super::num_consts::NumConsts;

/// A thin wrapper around u128 that is using strings for JSON encoding/decoding,
/// such that the full u128 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances of this and `u128` to get the value out:
///
/// ```
/// # use cosmwasm_std::Uint128;
/// let a = Uint128::from(123u128);
/// assert_eq!(a.u128(), 123);
///
/// let b = Uint128::from(42u64);
/// assert_eq!(b.u128(), 42);
///
/// let c = Uint128::from(70u32);
/// assert_eq!(c.u128(), 70);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Uint128(#[schemars(with = "String")] pub(crate) u128);

forward_ref_partial_eq!(Uint128, Uint128);

impl Uint128 {
    pub const MAX: Self = Self(u128::MAX);
    pub const MIN: Self = Self(u128::MIN);

    /// Creates a Uint128(value).
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    pub const fn new(value: u128) -> Self {
        Uint128(value)
    }

    /// Creates a Uint128(0)
    #[inline]
    pub const fn zero() -> Self {
        Uint128(0)
    }

    /// Creates a Uint128(1)
    #[inline]
    pub const fn one() -> Self {
        Self(1)
    }

    /// Returns a copy of the internal data
    pub const fn u128(&self) -> u128 {
        self.0
    }

    /// Returns a copy of the number as big endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_be_bytes(self) -> [u8; 16] {
        self.0.to_be_bytes()
    }

    /// Returns a copy of the number as little endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_le_bytes(self) -> [u8; 16] {
        self.0.to_le_bytes()
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn pow(self, exp: u32) -> Self {
        self.0.pow(exp).into()
    }

    /// Returns `self * numerator / denominator`.
    ///
    /// Due to the nature of the integer division involved, the result is always floored.
    /// E.g. 5 * 99/100 = 4.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn multiply_ratio<A: Into<u128>, B: Into<u128>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Uint128 {
        match self.checked_multiply_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedMultiplyRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedMultiplyRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns `self * numerator / denominator`.
    ///
    /// Due to the nature of the integer division involved, the result is always floored.
    /// E.g. 5 * 99/100 = 4.
    pub fn checked_multiply_ratio<A: Into<u128>, B: Into<u128>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Result<Uint128, CheckedMultiplyRatioError> {
        let numerator: u128 = numerator.into();
        let denominator: u128 = denominator.into();
        if denominator == 0 {
            return Err(CheckedMultiplyRatioError::DivideByZero);
        }
        match (self.full_mul(numerator) / Uint256::from(denominator)).try_into() {
            Ok(ratio) => Ok(ratio),
            Err(_) => Err(CheckedMultiplyRatioError::Overflow),
        }
    }

    /// Multiplies two u128 values without overflow, producing an
    /// [`Uint256`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmwasm_std::Uint128;
    ///
    /// let a = Uint128::MAX;
    /// let result = a.full_mul(2u32);
    /// assert_eq!(result.to_string(), "680564733841876926926749214863536422910");
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn full_mul(self, rhs: impl Into<u128>) -> Uint256 {
        Uint256::from(self.u128())
            .checked_mul(Uint256::from(rhs.into()))
            .unwrap()
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Add, self, other))
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Sub, self, other))
    }

    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_mul(other.0)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Mul, self, other))
    }

    pub fn checked_pow(self, exp: u32) -> Result<Self, OverflowError> {
        self.0
            .checked_pow(exp)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Pow, self, exp))
    }

    pub fn checked_div(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_div(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_div_euclid(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_div_euclid(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_shr(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 128 {
            return Err(OverflowError::new(OverflowOperation::Shr, self, other));
        }

        Ok(Self(self.0.shr(other)))
    }

    pub fn checked_shl(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 128 {
            return Err(OverflowError::new(OverflowOperation::Shl, self, other));
        }

        Ok(Self(self.0.shl(other)))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_add(self, other: Self) -> Self {
        Self(self.0.wrapping_add(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_sub(self, other: Self) -> Self {
        Self(self.0.wrapping_sub(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_mul(self, other: Self) -> Self {
        Self(self.0.wrapping_mul(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    #[inline]
    pub fn wrapping_pow(self, other: u32) -> Self {
        Self(self.0.wrapping_pow(other))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_add(self, other: Self) -> Self {
        Self(self.0.saturating_add(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_sub(self, other: Self) -> Self {
        Self(self.0.saturating_sub(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_mul(self, other: Self) -> Self {
        Self(self.0.saturating_mul(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_pow(self, exp: u32) -> Self {
        Self(self.0.saturating_pow(exp))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs_diff(self, other: Self) -> Self {
        Self(if self.0 < other.0 {
            other.0 - self.0
        } else {
            self.0 - other.0
        })
    }
}

impl NumConsts for Uint128 {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
}

impl_mul_fraction!(Uint128);

// `From<u{128,64,32,16,8}>` is implemented manually instead of
// using `impl<T: Into<u128>> From<T> for Uint128` because
// of the conflict with `TryFrom<&str>` as described here
// https://stackoverflow.com/questions/63136970/how-do-i-work-around-the-upstream-crates-may-add-a-new-impl-of-trait-error

impl From<Uint64> for Uint128 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}

impl From<u128> for Uint128 {
    fn from(val: u128) -> Self {
        Uint128(val)
    }
}

impl From<u64> for Uint128 {
    fn from(val: u64) -> Self {
        Uint128(val.into())
    }
}

impl From<u32> for Uint128 {
    fn from(val: u32) -> Self {
        Uint128(val.into())
    }
}

impl From<u16> for Uint128 {
    fn from(val: u16) -> Self {
        Uint128(val.into())
    }
}

impl From<u8> for Uint128 {
    fn from(val: u8) -> Self {
        Uint128(val.into())
    }
}

forward_try_from!(Uint128, Uint64);

// Int to Uint
forward_try_from!(Int64, Uint128);
forward_try_from!(Int128, Uint128);
forward_try_from!(Int256, Uint128);
forward_try_from!(Int512, Uint128);

impl TryFrom<&str> for Uint128 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Uint128 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u128: {e}"))),
        }
    }
}

impl From<Uint128> for String {
    fn from(original: Uint128) -> Self {
        original.to_string()
    }
}

impl From<Uint128> for u128 {
    fn from(original: Uint128) -> Self {
        original.0
    }
}

impl fmt::Display for Uint128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<Uint128> for Uint128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint128(
            self.u128()
                .checked_add(rhs.u128())
                .expect("attempt to add with overflow"),
        )
    }
}

impl<'a> Add<&'a Uint128> for Uint128 {
    type Output = Self;

    fn add(self, rhs: &'a Uint128) -> Self {
        self + *rhs
    }
}

impl Sub<Uint128> for Uint128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Uint128(
            self.u128()
                .checked_sub(rhs.u128())
                .expect("attempt to subtract with overflow"),
        )
    }
}
forward_ref_binop!(impl Sub, sub for Uint128, Uint128);

impl SubAssign<Uint128> for Uint128 {
    fn sub_assign(&mut self, rhs: Uint128) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Uint128, Uint128);

impl Mul<Uint128> for Uint128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(
            self.u128()
                .checked_mul(rhs.u128())
                .expect("attempt to multiply with overflow"),
        )
    }
}
forward_ref_binop!(impl Mul, mul for Uint128, Uint128);

impl MulAssign<Uint128> for Uint128 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Uint128, Uint128);

impl Div<Uint128> for Uint128 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(
            self.u128()
                .checked_div(rhs.u128())
                .expect("attempt to divide by zero"),
        )
    }
}

impl<'a> Div<&'a Uint128> for Uint128 {
    type Output = Self;

    fn div(self, rhs: &'a Uint128) -> Self::Output {
        self / *rhs
    }
}

impl Shr<u32> for Uint128 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        Self(
            self.u128()
                .checked_shr(rhs)
                .expect("attempt to shift right with overflow"),
        )
    }
}

impl<'a> Shr<&'a u32> for Uint128 {
    type Output = Self;

    fn shr(self, rhs: &'a u32) -> Self::Output {
        self >> *rhs
    }
}

impl Shl<u32> for Uint128 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        Self(
            self.u128()
                .checked_shl(rhs)
                .expect("attempt to shift left with overflow"),
        )
    }
}

impl<'a> Shl<&'a u32> for Uint128 {
    type Output = Self;

    fn shl(self, rhs: &'a u32) -> Self::Output {
        self.shl(*rhs)
    }
}

impl AddAssign<Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: Uint128) {
        *self = *self + rhs;
    }
}

impl<'a> AddAssign<&'a Uint128> for Uint128 {
    fn add_assign(&mut self, rhs: &'a Uint128) {
        *self = *self + rhs;
    }
}

impl DivAssign<Uint128> for Uint128 {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl<'a> DivAssign<&'a Uint128> for Uint128 {
    fn div_assign(&mut self, rhs: &'a Uint128) {
        *self = *self / rhs;
    }
}

impl Rem for Uint128 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero.
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Uint128, Uint128);

impl Not for Uint128 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl RemAssign<Uint128> for Uint128 {
    fn rem_assign(&mut self, rhs: Uint128) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Uint128, Uint128);

impl ShrAssign<u32> for Uint128 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = *self >> rhs;
    }
}

impl<'a> ShrAssign<&'a u32> for Uint128 {
    fn shr_assign(&mut self, rhs: &'a u32) {
        *self = *self >> rhs;
    }
}

impl ShlAssign<u32> for Uint128 {
    fn shl_assign(&mut self, rhs: u32) {
        *self = Shl::<u32>::shl(*self, rhs);
    }
}

impl<'a> ShlAssign<&'a u32> for Uint128 {
    fn shl_assign(&mut self, rhs: &'a u32) {
        *self = Shl::<u32>::shl(*self, *rhs);
    }
}

impl Serialize for Uint128 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint128 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint128, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint128Visitor)
    }
}

struct Uint128Visitor;

impl<'de> de::Visitor<'de> for Uint128Visitor {
    type Value = Uint128;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match v.parse::<u128>() {
            Ok(u) => Ok(Uint128(u)),
            Err(e) => Err(E::custom(format!("invalid Uint128 '{v}' - {e}"))),
        }
    }
}

impl<A> core::iter::Sum<A> for Uint128
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

#[cfg(test)]
mod tests {
    use crate::errors::CheckedMultiplyFractionError::{ConversionOverflow, DivideByZero};
    use crate::math::conversion::test_try_from_int_to_uint;
    use crate::{from_json, to_json_vec, ConversionOverflowError, Decimal};

    use super::*;

    #[test]
    fn size_of_works() {
        assert_eq!(core::mem::size_of::<Uint128>(), 16);
    }

    #[test]
    fn uint128_not_works() {
        assert_eq!(!Uint128::new(1234806), Uint128::new(!1234806));

        assert_eq!(!Uint128::MAX, Uint128::new(!u128::MAX));
        assert_eq!(!Uint128::MIN, Uint128::new(!u128::MIN));
    }

    #[test]
    fn uint128_zero_works() {
        let zero = Uint128::zero();
        assert_eq!(
            zero.to_be_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn uint128_one_works() {
        let one = Uint128::one();
        assert_eq!(
            one.to_be_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
    }

    #[test]
    fn uint128_convert_into() {
        let original = Uint128(12345);
        let a = u128::from(original);
        assert_eq!(a, 12345);

        let original = Uint128(12345);
        let a = String::from(original);
        assert_eq!(a, "12345");
    }

    #[test]
    fn uint128_convert_from() {
        let a = Uint128::from(5u128);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u64);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u32);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u16);
        assert_eq!(a.0, 5);

        let a = Uint128::from(5u8);
        assert_eq!(a.0, 5);

        let result = Uint128::try_from("34567");
        assert_eq!(result.unwrap().0, 34567);

        let result = Uint128::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint128_try_from_signed_works() {
        test_try_from_int_to_uint::<Int64, Uint128>("Int64", "Uint128");
        test_try_from_int_to_uint::<Int128, Uint128>("Int128", "Uint128");
        test_try_from_int_to_uint::<Int256, Uint128>("Int256", "Uint128");
        test_try_from_int_to_uint::<Int512, Uint128>("Int512", "Uint128");
    }

    #[test]
    fn uint128_try_into() {
        assert!(Uint64::try_from(Uint128::MAX).is_err());

        assert_eq!(Uint64::try_from(Uint128::zero()), Ok(Uint64::zero()));

        assert_eq!(
            Uint64::try_from(Uint128::from(42u64)),
            Ok(Uint64::from(42u64))
        );
    }

    #[test]
    fn uint128_implements_display() {
        let a = Uint128(12345);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint128(0);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint128_display_padding_works() {
        // width > natural representation
        let a = Uint128::from(123u64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: 00123");

        // width < natural representation
        let a = Uint128::from(123u64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: 123");
    }

    #[test]
    fn uint128_to_be_bytes_works() {
        assert_eq!(
            Uint128::zero().to_be_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            Uint128::MAX.to_be_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ]
        );
        assert_eq!(
            Uint128::new(1).to_be_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(16, "big")]`
        assert_eq!(
            Uint128::new(240282366920938463463374607431768124608).to_be_bytes(),
            [180, 196, 179, 87, 165, 121, 59, 133, 246, 117, 221, 191, 255, 254, 172, 192]
        );
    }

    #[test]
    fn uint128_to_le_bytes_works() {
        assert_eq!(
            Uint128::zero().to_le_bytes(),
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        assert_eq!(
            Uint128::MAX.to_le_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff
            ]
        );
        assert_eq!(
            Uint128::new(1).to_le_bytes(),
            [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(16, "little")]`
        assert_eq!(
            Uint128::new(240282366920938463463374607431768124608).to_le_bytes(),
            [192, 172, 254, 255, 191, 221, 117, 246, 133, 59, 121, 165, 87, 179, 196, 180]
        );
    }

    #[test]
    fn uint128_is_zero_works() {
        assert!(Uint128::zero().is_zero());
        assert!(Uint128(0).is_zero());

        assert!(!Uint128(1).is_zero());
        assert!(!Uint128(123).is_zero());
    }

    #[test]
    fn uint128_json() {
        let orig = Uint128(1234567890987654321);
        let serialized = to_json_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint128 = from_json(serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint128_compare() {
        let a = Uint128(12345);
        let b = Uint128(23456);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint128(12345));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_math() {
        let a = Uint128(12345);
        let b = Uint128(23456);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint128(35801));
        assert_eq!(a + &b, Uint128(35801));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Uint128(11111));
        assert_eq!(b - &a, Uint128(11111));

        // test += with owned and reference right hand side
        let mut c = Uint128(300000);
        c += b;
        assert_eq!(c, Uint128(323456));
        let mut d = Uint128(300000);
        d += &b;
        assert_eq!(d, Uint128(323456));

        // test -= with owned and reference right hand side
        let mut c = Uint128(300000);
        c -= b;
        assert_eq!(c, Uint128(276544));
        let mut d = Uint128(300000);
        d -= &b;
        assert_eq!(d, Uint128(276544));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint128_add_overflow_panics() {
        // almost_max is 2^128 - 10
        let almost_max = Uint128(340282366920938463463374607431768211446);
        let _ = almost_max + Uint128(12);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_sub_works() {
        assert_eq!(Uint128(2) - Uint128(1), Uint128(1));
        assert_eq!(Uint128(2) - Uint128(0), Uint128(2));
        assert_eq!(Uint128(2) - Uint128(2), Uint128(0));

        // works for refs
        let a = Uint128::new(10);
        let b = Uint128::new(3);
        let expected = Uint128::new(7);
        assert_eq!(a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn uint128_sub_overflow_panics() {
        let _ = Uint128(1) - Uint128(2);
    }

    #[test]
    fn uint128_sub_assign_works() {
        let mut a = Uint128(14);
        a -= Uint128(2);
        assert_eq!(a, Uint128(12));

        // works for refs
        let mut a = Uint128::new(10);
        let b = Uint128::new(3);
        let expected = Uint128::new(7);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_mul_works() {
        assert_eq!(
            Uint128::from(2u32) * Uint128::from(3u32),
            Uint128::from(6u32)
        );
        assert_eq!(Uint128::from(2u32) * Uint128::zero(), Uint128::zero());

        // works for refs
        let a = Uint128::from(11u32);
        let b = Uint128::from(3u32);
        let expected = Uint128::from(33u32);
        assert_eq!(a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn uint128_mul_assign_works() {
        let mut a = Uint128::from(14u32);
        a *= Uint128::from(2u32);
        assert_eq!(a, Uint128::from(28u32));

        // works for refs
        let mut a = Uint128::from(10u32);
        let b = Uint128::from(3u32);
        a *= &b;
        assert_eq!(a, Uint128::from(30u32));
    }

    #[test]
    fn uint128_pow_works() {
        assert_eq!(Uint128::from(2u32).pow(2), Uint128::from(4u32));
        assert_eq!(Uint128::from(2u32).pow(10), Uint128::from(1024u32));
    }

    #[test]
    #[should_panic]
    fn uint128_pow_overflow_panics() {
        _ = Uint128::MAX.pow(2u32);
    }

    #[test]
    fn uint128_multiply_ratio_works() {
        let base = Uint128(500);

        // factor 1/1
        assert_eq!(base.multiply_ratio(1u128, 1u128), base);
        assert_eq!(base.multiply_ratio(3u128, 3u128), base);
        assert_eq!(base.multiply_ratio(654321u128, 654321u128), base);
        assert_eq!(base.multiply_ratio(u128::MAX, u128::MAX), base);

        // factor 3/2
        assert_eq!(base.multiply_ratio(3u128, 2u128), Uint128(750));
        assert_eq!(base.multiply_ratio(333333u128, 222222u128), Uint128(750));

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(2u128, 3u128), Uint128(333));
        assert_eq!(base.multiply_ratio(222222u128, 333333u128), Uint128(333));

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.multiply_ratio(5u128, 6u128), Uint128(416));
        assert_eq!(base.multiply_ratio(100u128, 120u128), Uint128(416));
    }

    #[test]
    fn uint128_multiply_ratio_does_not_overflow_when_result_fits() {
        // Almost max value for Uint128.
        let base = Uint128(u128::MAX - 9);

        assert_eq!(base.multiply_ratio(2u128, 2u128), base);
    }

    #[test]
    #[should_panic]
    fn uint128_multiply_ratio_panicks_on_overflow() {
        // Almost max value for Uint128.
        let base = Uint128(u128::MAX - 9);

        assert_eq!(base.multiply_ratio(2u128, 1u128), base);
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn uint128_multiply_ratio_panics_for_zero_denominator() {
        _ = Uint128(500).multiply_ratio(1u128, 0u128);
    }

    #[test]
    fn uint128_checked_multiply_ratio_does_not_panic() {
        assert_eq!(
            Uint128(500u128).checked_multiply_ratio(1u128, 0u128),
            Err(CheckedMultiplyRatioError::DivideByZero),
        );
        assert_eq!(
            Uint128(500u128).checked_multiply_ratio(u128::MAX, 1u128),
            Err(CheckedMultiplyRatioError::Overflow),
        );
    }

    #[test]
    fn uint128_shr_works() {
        let original = Uint128::new(u128::from_be_bytes([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]));

        let shifted = Uint128::new(u128::from_be_bytes([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]));

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint128_shr_overflow_panics() {
        let _ = Uint128::from(1u32) >> 128u32;
    }

    #[test]
    fn uint128_shl_works() {
        let original = Uint128::new(u128::from_be_bytes([
            64u8, 128u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]));

        let shifted = Uint128::new(u128::from_be_bytes([
            2u8, 0u8, 4u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]));

        assert_eq!(original << 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint128_shl_overflow_panics() {
        let _ = Uint128::from(1u32) << 128u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![Uint128(17), Uint128(123), Uint128(540), Uint128(82)];
        let expected = Uint128(762);

        let sum_as_ref: Uint128 = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned: Uint128 = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint128_methods() {
        // checked_*
        assert!(matches!(
            Uint128::MAX.checked_add(Uint128(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(Uint128(1).checked_add(Uint128(1)), Ok(Uint128(2))));
        assert!(matches!(
            Uint128(0).checked_sub(Uint128(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(Uint128(2).checked_sub(Uint128(1)), Ok(Uint128(1))));
        assert!(matches!(
            Uint128::MAX.checked_mul(Uint128(2)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(Uint128(2).checked_mul(Uint128(2)), Ok(Uint128(4))));
        assert!(matches!(
            Uint128::MAX.checked_pow(2u32),
            Err(OverflowError { .. })
        ));
        assert!(matches!(Uint128(2).checked_pow(3), Ok(Uint128(8))));
        assert!(matches!(
            Uint128::MAX.checked_div(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(Uint128(6).checked_div(Uint128(2)), Ok(Uint128(3))));
        assert!(matches!(
            Uint128::MAX.checked_div_euclid(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));
        assert!(matches!(
            Uint128(6).checked_div_euclid(Uint128(2)),
            Ok(Uint128(3)),
        ));
        assert!(matches!(
            Uint128::MAX.checked_rem(Uint128(0)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(Uint128::MAX.saturating_add(Uint128(1)), Uint128::MAX);
        assert_eq!(Uint128(0).saturating_sub(Uint128(1)), Uint128(0));
        assert_eq!(Uint128::MAX.saturating_mul(Uint128(2)), Uint128::MAX);
        assert_eq!(Uint128::MAX.saturating_pow(2), Uint128::MAX);
    }

    #[test]
    fn uint128_wrapping_methods() {
        // wrapping_add
        assert_eq!(Uint128(2).wrapping_add(Uint128(2)), Uint128(4)); // non-wrapping
        assert_eq!(Uint128::MAX.wrapping_add(Uint128(1)), Uint128(0)); // wrapping

        // wrapping_sub
        assert_eq!(Uint128(7).wrapping_sub(Uint128(5)), Uint128(2)); // non-wrapping
        assert_eq!(Uint128(0).wrapping_sub(Uint128(1)), Uint128::MAX); // wrapping

        // wrapping_mul
        assert_eq!(Uint128(3).wrapping_mul(Uint128(2)), Uint128(6)); // non-wrapping
        assert_eq!(
            Uint128::MAX.wrapping_mul(Uint128(2)),
            Uint128::MAX - Uint128::one()
        ); // wrapping

        // wrapping_pow
        assert_eq!(Uint128(2).wrapping_pow(3), Uint128(8)); // non-wrapping
        assert_eq!(Uint128::MAX.wrapping_pow(2), Uint128(1)); // wrapping
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_implements_rem() {
        let a = Uint128::new(10);
        assert_eq!(a % Uint128::new(10), Uint128::zero());
        assert_eq!(a % Uint128::new(2), Uint128::zero());
        assert_eq!(a % Uint128::new(1), Uint128::zero());
        assert_eq!(a % Uint128::new(3), Uint128::new(1));
        assert_eq!(a % Uint128::new(4), Uint128::new(2));

        // works for refs
        let a = Uint128::new(10);
        let b = Uint128::new(3);
        let expected = Uint128::new(1);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn uint128_rem_panics_for_zero() {
        let _ = Uint128::new(10) % Uint128::zero();
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint128_rem_works() {
        assert_eq!(
            Uint128::from(12u32) % Uint128::from(10u32),
            Uint128::from(2u32)
        );
        assert_eq!(Uint128::from(50u32) % Uint128::from(5u32), Uint128::zero());

        // works for refs
        let a = Uint128::from(42u32);
        let b = Uint128::from(5u32);
        let expected = Uint128::from(2u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn uint128_rem_assign_works() {
        let mut a = Uint128::from(30u32);
        a %= Uint128::from(4u32);
        assert_eq!(a, Uint128::from(2u32));

        // works for refs
        let mut a = Uint128::from(25u32);
        let b = Uint128::from(6u32);
        a %= &b;
        assert_eq!(a, Uint128::from(1u32));
    }

    #[test]
    fn uint128_abs_diff_works() {
        let a = Uint128::from(42u32);
        let b = Uint128::from(5u32);
        let expected = Uint128::from(37u32);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    fn uint128_partial_eq() {
        let test_cases = [(1, 1, true), (42, 42, true), (42, 24, false), (0, 0, true)]
            .into_iter()
            .map(|(lhs, rhs, expected)| (Uint128::new(lhs), Uint128::new(rhs), expected));

        #[allow(clippy::op_ref)]
        for (lhs, rhs, expected) in test_cases {
            assert_eq!(lhs == rhs, expected);
            assert_eq!(&lhs == rhs, expected);
            assert_eq!(lhs == &rhs, expected);
            assert_eq!(&lhs == &rhs, expected);
        }
    }

    #[test]
    fn mul_floor_works_with_zero() {
        let fraction = (Uint128::zero(), Uint128::new(21));
        let res = Uint128::new(123456).mul_floor(fraction);
        assert_eq!(Uint128::zero(), res)
    }

    #[test]
    fn mul_floor_does_nothing_with_one() {
        let fraction = (Uint128::one(), Uint128::one());
        let res = Uint128::new(123456).mul_floor(fraction);
        assert_eq!(Uint128::new(123456), res)
    }

    #[test]
    fn mul_floor_rounds_down_with_normal_case() {
        let fraction = (8u128, 21u128);
        let res = Uint128::new(123456).mul_floor(fraction); // 47030.8571
        assert_eq!(Uint128::new(47030), res)
    }

    #[test]
    fn mul_floor_does_not_round_on_even_divide() {
        let fraction = (2u128, 5u128);
        let res = Uint128::new(25).mul_floor(fraction);
        assert_eq!(Uint128::new(10), res)
    }

    #[test]
    fn mul_floor_works_when_operation_temporarily_takes_above_max() {
        let fraction = (8u128, 21u128);
        let res = Uint128::MAX.mul_floor(fraction); // 129_631_377_874_643_224_176_523_659_974_006_937_697.14285
        assert_eq!(
            Uint128::new(129_631_377_874_643_224_176_523_659_974_006_937_697),
            res
        )
    }

    #[test]
    fn mul_floor_works_with_decimal() {
        let decimal = Decimal::from_ratio(8u128, 21u128);
        let res = Uint128::new(123456).mul_floor(decimal); // 47030.8571
        assert_eq!(Uint128::new(47030), res)
    }

    #[test]
    #[should_panic(expected = "ConversionOverflowError")]
    fn mul_floor_panics_on_overflow() {
        let fraction = (21u128, 8u128);
        _ = Uint128::MAX.mul_floor(fraction);
    }

    #[test]
    fn checked_mul_floor_does_not_panic_on_overflow() {
        let fraction = (21u128, 8u128);
        assert_eq!(
            Uint128::MAX.checked_mul_floor(fraction),
            Err(ConversionOverflow(ConversionOverflowError {
                source_type: "Uint256",
                target_type: "Uint128",
                value: "893241213167463466591358344508391555069".to_string()
            })),
        );
    }

    #[test]
    #[should_panic(expected = "DivideByZeroError")]
    fn mul_floor_panics_on_zero_div() {
        let fraction = (21u128, 0u128);
        _ = Uint128::new(123456).mul_floor(fraction);
    }

    #[test]
    fn checked_mul_floor_does_not_panic_on_zero_div() {
        let fraction = (21u128, 0u128);
        assert_eq!(
            Uint128::new(123456).checked_mul_floor(fraction),
            Err(DivideByZero(DivideByZeroError {
                operand: "2592576".to_string()
            })),
        );
    }

    #[test]
    fn mul_ceil_works_with_zero() {
        let fraction = (Uint128::zero(), Uint128::new(21));
        let res = Uint128::new(123456).mul_ceil(fraction);
        assert_eq!(Uint128::zero(), res)
    }

    #[test]
    fn mul_ceil_does_nothing_with_one() {
        let fraction = (Uint128::one(), Uint128::one());
        let res = Uint128::new(123456).mul_ceil(fraction);
        assert_eq!(Uint128::new(123456), res)
    }

    #[test]
    fn mul_ceil_rounds_up_with_normal_case() {
        let fraction = (8u128, 21u128);
        let res = Uint128::new(123456).mul_ceil(fraction); // 47030.8571
        assert_eq!(Uint128::new(47031), res)
    }

    #[test]
    fn mul_ceil_does_not_round_on_even_divide() {
        let fraction = (2u128, 5u128);
        let res = Uint128::new(25).mul_ceil(fraction);
        assert_eq!(Uint128::new(10), res)
    }

    #[test]
    fn mul_ceil_works_when_operation_temporarily_takes_above_max() {
        let fraction = (8u128, 21u128);
        let res = Uint128::MAX.mul_ceil(fraction); // 129_631_377_874_643_224_176_523_659_974_006_937_697.14285
        assert_eq!(
            Uint128::new(129_631_377_874_643_224_176_523_659_974_006_937_698),
            res
        )
    }

    #[test]
    fn mul_ceil_works_with_decimal() {
        let decimal = Decimal::from_ratio(8u128, 21u128);
        let res = Uint128::new(123456).mul_ceil(decimal); // 47030.8571
        assert_eq!(Uint128::new(47031), res)
    }

    #[test]
    #[should_panic(expected = "ConversionOverflowError")]
    fn mul_ceil_panics_on_overflow() {
        let fraction = (21u128, 8u128);
        _ = Uint128::MAX.mul_ceil(fraction);
    }

    #[test]
    fn checked_mul_ceil_does_not_panic_on_overflow() {
        let fraction = (21u128, 8u128);
        assert_eq!(
            Uint128::MAX.checked_mul_ceil(fraction),
            Err(ConversionOverflow(ConversionOverflowError {
                source_type: "Uint256",
                target_type: "Uint128",
                value: "893241213167463466591358344508391555069".to_string() // raises prior to rounding up
            })),
        );
    }

    #[test]
    #[should_panic(expected = "DivideByZeroError")]
    fn mul_ceil_panics_on_zero_div() {
        let fraction = (21u128, 0u128);
        _ = Uint128::new(123456).mul_ceil(fraction);
    }

    #[test]
    fn checked_mul_ceil_does_not_panic_on_zero_div() {
        let fraction = (21u128, 0u128);
        assert_eq!(
            Uint128::new(123456).checked_mul_ceil(fraction),
            Err(DivideByZero(DivideByZeroError {
                operand: "2592576".to_string()
            })),
        );
    }

    #[test]
    #[should_panic(expected = "DivideByZeroError")]
    fn div_floor_raises_with_zero() {
        let fraction = (Uint128::zero(), Uint128::new(21));
        _ = Uint128::new(123456).div_floor(fraction);
    }

    #[test]
    fn div_floor_does_nothing_with_one() {
        let fraction = (Uint128::one(), Uint128::one());
        let res = Uint128::new(123456).div_floor(fraction);
        assert_eq!(Uint128::new(123456), res)
    }

    #[test]
    fn div_floor_rounds_down_with_normal_case() {
        let fraction = (5u128, 21u128);
        let res = Uint128::new(123456).div_floor(fraction); // 518515.2
        assert_eq!(Uint128::new(518515), res)
    }

    #[test]
    fn div_floor_does_not_round_on_even_divide() {
        let fraction = (5u128, 2u128);
        let res = Uint128::new(25).div_floor(fraction);
        assert_eq!(Uint128::new(10), res)
    }

    #[test]
    fn div_floor_works_when_operation_temporarily_takes_above_max() {
        let fraction = (21u128, 8u128);
        let res = Uint128::MAX.div_floor(fraction); // 129_631_377_874_643_224_176_523_659_974_006_937_697.1428
        assert_eq!(
            Uint128::new(129_631_377_874_643_224_176_523_659_974_006_937_697),
            res
        )
    }

    #[test]
    fn div_floor_works_with_decimal() {
        let decimal = Decimal::from_ratio(21u128, 8u128);
        let res = Uint128::new(123456).div_floor(decimal); // 47030.8571
        assert_eq!(Uint128::new(47030), res)
    }

    #[test]
    fn div_floor_works_with_decimal_evenly() {
        let res = Uint128::new(60).div_floor(Decimal::from_atomics(6u128, 0).unwrap());
        assert_eq!(res, Uint128::new(10));
    }

    #[test]
    #[should_panic(expected = "ConversionOverflowError")]
    fn div_floor_panics_on_overflow() {
        let fraction = (8u128, 21u128);
        _ = Uint128::MAX.div_floor(fraction);
    }

    #[test]
    fn div_floor_does_not_panic_on_overflow() {
        let fraction = (8u128, 21u128);
        assert_eq!(
            Uint128::MAX.checked_div_floor(fraction),
            Err(ConversionOverflow(ConversionOverflowError {
                source_type: "Uint256",
                target_type: "Uint128",
                value: "893241213167463466591358344508391555069".to_string()
            })),
        );
    }

    #[test]
    #[should_panic(expected = "DivideByZeroError")]
    fn div_ceil_raises_with_zero() {
        let fraction = (Uint128::zero(), Uint128::new(21));
        _ = Uint128::new(123456).div_ceil(fraction);
    }

    #[test]
    fn div_ceil_does_nothing_with_one() {
        let fraction = (Uint128::one(), Uint128::one());
        let res = Uint128::new(123456).div_ceil(fraction);
        assert_eq!(Uint128::new(123456), res)
    }

    #[test]
    fn div_ceil_rounds_up_with_normal_case() {
        let fraction = (5u128, 21u128);
        let res = Uint128::new(123456).div_ceil(fraction); // 518515.2
        assert_eq!(Uint128::new(518516), res)
    }

    #[test]
    fn div_ceil_does_not_round_on_even_divide() {
        let fraction = (5u128, 2u128);
        let res = Uint128::new(25).div_ceil(fraction);
        assert_eq!(Uint128::new(10), res)
    }

    #[test]
    fn div_ceil_works_when_operation_temporarily_takes_above_max() {
        let fraction = (21u128, 8u128);
        let res = Uint128::MAX.div_ceil(fraction); // 129_631_377_874_643_224_176_523_659_974_006_937_697.1428
        assert_eq!(
            Uint128::new(129_631_377_874_643_224_176_523_659_974_006_937_698),
            res
        )
    }

    #[test]
    fn div_ceil_works_with_decimal() {
        let decimal = Decimal::from_ratio(21u128, 8u128);
        let res = Uint128::new(123456).div_ceil(decimal); // 47030.8571
        assert_eq!(Uint128::new(47031), res)
    }

    #[test]
    fn div_ceil_works_with_decimal_evenly() {
        let res = Uint128::new(60).div_ceil(Decimal::from_atomics(6u128, 0).unwrap());
        assert_eq!(res, Uint128::new(10));
    }

    #[test]
    #[should_panic(expected = "ConversionOverflowError")]
    fn div_ceil_panics_on_overflow() {
        let fraction = (8u128, 21u128);
        _ = Uint128::MAX.div_ceil(fraction);
    }

    #[test]
    fn div_ceil_does_not_panic_on_overflow() {
        let fraction = (8u128, 21u128);
        assert_eq!(
            Uint128::MAX.checked_div_ceil(fraction),
            Err(ConversionOverflow(ConversionOverflowError {
                source_type: "Uint256",
                target_type: "Uint128",
                value: "893241213167463466591358344508391555069".to_string()
            })),
        );
    }
}
