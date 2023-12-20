use core::fmt;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};
use core::str::FromStr;
use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{DivideByZeroError, DivisionError, OverflowError, OverflowOperation, StdError};
use crate::{
    forward_ref_partial_eq, CheckedMultiplyRatioError, Int128, Int256, Int512, Uint128, Uint256,
    Uint512, Uint64,
};

use super::conversion::{forward_try_from, try_from_int_to_int};
use super::num_consts::NumConsts;

/// An implementation of i64 that is using strings for JSON encoding/decoding,
/// such that the full i64 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances of this and `i64` to get the value out:
///
/// ```
/// # use cosmwasm_std::Int64;
/// let a = Int64::from(258i64);
/// assert_eq!(a.i64(), 258);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Int64(#[schemars(with = "String")] pub(crate) i64);

forward_ref_partial_eq!(Int64, Int64);

impl Int64 {
    pub const MAX: Int64 = Int64(i64::MAX);
    pub const MIN: Int64 = Int64(i64::MIN);

    /// Creates a Int64(value).
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    #[inline]
    pub const fn new(value: i64) -> Self {
        Self(value)
    }

    /// Creates a Int64(0)
    #[inline]
    pub const fn zero() -> Self {
        Int64(0)
    }

    /// Creates a Int64(1)
    #[inline]
    pub const fn one() -> Self {
        Self(1)
    }

    /// Returns a copy of the internal data
    pub const fn i64(&self) -> i64 {
        self.0
    }

    #[must_use]
    pub const fn from_be_bytes(data: [u8; 8]) -> Self {
        Self(i64::from_be_bytes(data))
    }

    #[must_use]
    pub const fn from_le_bytes(data: [u8; 8]) -> Self {
        Self(i64::from_le_bytes(data))
    }

    /// Returns a copy of the number as big endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_be_bytes(self) -> [u8; 8] {
        self.0.to_be_bytes()
    }

    /// Returns a copy of the number as little endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_le_bytes(self) -> [u8; 8] {
        self.0.to_le_bytes()
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0 == 0
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn pow(self, exp: u32) -> Self {
        Self(self.0.pow(exp))
    }

    /// Returns `self * numerator / denominator`.
    ///
    /// Due to the nature of the integer division involved, the result is always floored.
    /// E.g. 5 * 99/100 = 4.
    pub fn checked_multiply_ratio<A: Into<Self>, B: Into<Self>>(
        &self,
        numerator: A,
        denominator: B,
    ) -> Result<Self, CheckedMultiplyRatioError> {
        let numerator = numerator.into();
        let denominator = denominator.into();
        if denominator.is_zero() {
            return Err(CheckedMultiplyRatioError::DivideByZero);
        }
        match (self.full_mul(numerator) / Int128::from(denominator)).try_into() {
            Ok(ratio) => Ok(ratio),
            Err(_) => Err(CheckedMultiplyRatioError::Overflow),
        }
    }

    /// Multiplies two [`Int64`] values without overflow, producing an
    /// [`Int128`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmwasm_std::Int64;
    ///
    /// let a = Int64::MAX;
    /// let result = a.full_mul(2i32);
    /// assert_eq!(result.to_string(), "18446744073709551614");
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn full_mul(self, rhs: impl Into<Self>) -> Int128 {
        Int128::from(self)
            .checked_mul(Int128::from(rhs.into()))
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

    pub fn checked_div(self, other: Self) -> Result<Self, DivisionError> {
        if other.is_zero() {
            return Err(DivisionError::DivideByZero);
        }
        self.0
            .checked_div(other.0)
            .map(Self)
            .ok_or(DivisionError::Overflow)
    }

    pub fn checked_div_euclid(self, other: Self) -> Result<Self, DivisionError> {
        if other.is_zero() {
            return Err(DivisionError::DivideByZero);
        }
        self.0
            .checked_div_euclid(other.0)
            .map(Self)
            .ok_or(DivisionError::Overflow)
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_shr(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 64 {
            return Err(OverflowError::new(OverflowOperation::Shr, self, other));
        }

        Ok(Self(self.0.shr(other)))
    }

    pub fn checked_shl(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 64 {
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
    pub const fn abs_diff(self, other: Self) -> Uint64 {
        Uint64(self.0.abs_diff(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn unsigned_abs(self) -> Uint64 {
        Uint64(self.0.unsigned_abs())
    }
}

impl NumConsts for Int64 {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
}

// uint to Int
impl From<u32> for Int64 {
    fn from(val: u32) -> Self {
        Int64(val.into())
    }
}

impl From<u16> for Int64 {
    fn from(val: u16) -> Self {
        Int64(val.into())
    }
}

impl From<u8> for Int64 {
    fn from(val: u8) -> Self {
        Int64(val.into())
    }
}

// int to Int
impl From<i64> for Int64 {
    fn from(val: i64) -> Self {
        Int64(val)
    }
}

impl From<i32> for Int64 {
    fn from(val: i32) -> Self {
        Int64(val.into())
    }
}

impl From<i16> for Int64 {
    fn from(val: i16) -> Self {
        Int64(val.into())
    }
}

impl From<i8> for Int64 {
    fn from(val: i8) -> Self {
        Int64(val.into())
    }
}

// Int to Int
try_from_int_to_int!(Int128, Int64);
try_from_int_to_int!(Int256, Int64);
try_from_int_to_int!(Int512, Int64);

// Uint to Int
forward_try_from!(Uint64, Int64);
forward_try_from!(Uint128, Int64);
forward_try_from!(Uint256, Int64);
forward_try_from!(Uint512, Int64);

impl TryFrom<&str> for Int64 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Int64 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i64>() {
            Ok(u) => Ok(Self(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing Int64: {e}"))),
        }
    }
}

impl From<Int64> for String {
    fn from(original: Int64) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Int64 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<Int64> for Int64 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Int64(self.0.checked_add(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Add, add for Int64, Int64);

impl Sub<Int64> for Int64 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Int64(self.0.checked_sub(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Sub, sub for Int64, Int64);

impl SubAssign<Int64> for Int64 {
    fn sub_assign(&mut self, rhs: Int64) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Int64, Int64);

impl Div<Int64> for Int64 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Div, div for Int64, Int64);

impl Rem for Int64 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero.
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Int64, Int64);

impl Not for Int64 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Neg for Int64 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl RemAssign<Int64> for Int64 {
    fn rem_assign(&mut self, rhs: Int64) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Int64, Int64);

impl Mul<Int64> for Int64 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Mul, mul for Int64, Int64);

impl MulAssign<Int64> for Int64 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Int64, Int64);

impl Shr<u32> for Int64 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!("right shift error: {rhs} is larger or equal than the number of bits in Int64",)
        })
    }
}
forward_ref_binop!(impl Shr, shr for Int64, u32);

impl Shl<u32> for Int64 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs).unwrap_or_else(|_| {
            panic!("left shift error: {rhs} is larger or equal than the number of bits in Int64",)
        })
    }
}
forward_ref_binop!(impl Shl, shl for Int64, u32);

impl AddAssign<Int64> for Int64 {
    fn add_assign(&mut self, rhs: Int64) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Int64, Int64);

impl DivAssign<Int64> for Int64 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Int64, Int64);

impl ShrAssign<u32> for Int64 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShrAssign, shr_assign for Int64, u32);

impl ShlAssign<u32> for Int64 {
    fn shl_assign(&mut self, rhs: u32) {
        *self = Shl::<u32>::shl(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShlAssign, shl_assign for Int64, u32);

impl Serialize for Int64 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Int64 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Int64, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Int64Visitor)
    }
}

struct Int64Visitor;

impl<'de> de::Visitor<'de> for Int64Visitor {
    type Value = Int64;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Int64::try_from(v).map_err(|e| E::custom(format!("invalid Int64 '{v}' - {e}")))
    }
}

impl<A> core::iter::Sum<A> for Int64
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_json, math::conversion::test_try_from_uint_to_int, to_json_vec};

    #[test]
    fn size_of_works() {
        assert_eq!(core::mem::size_of::<Int64>(), 8);
    }

    #[test]
    fn int64_from_be_bytes_works() {
        let num = Int64::from_be_bytes([1; 8]);
        let a: [u8; 8] = num.to_be_bytes();
        assert_eq!(a, [1; 8]);

        let be_bytes = [0u8, 222u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8];
        let num = Int64::from_be_bytes(be_bytes);
        let resulting_bytes: [u8; 8] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn int64_new_works() {
        let num = Int64::new(222);
        assert_eq!(num.i64(), 222);

        let num = Int64::new(-222);
        assert_eq!(num.i64(), -222);

        let num = Int64::new(i64::MAX);
        assert_eq!(num.i64(), i64::MAX);

        let num = Int64::new(i64::MIN);
        assert_eq!(num.i64(), i64::MIN);
    }

    #[test]
    fn int64_not_works() {
        assert_eq!(!Int64::new(222), Int64::new(!222));
        assert_eq!(!Int64::new(-222), Int64::new(!-222));

        assert_eq!(!Int64::MAX, Int64::new(!i64::MAX));
        assert_eq!(!Int64::MIN, Int64::new(!i64::MIN));
    }

    #[test]
    fn int64_zero_works() {
        let zero = Int64::zero();
        assert_eq!(zero.to_be_bytes(), [0; 8]);
    }

    #[test]
    fn uint64_one_works() {
        let one = Int64::one();
        let mut one_be = [0; 8];
        one_be[7] = 1;

        assert_eq!(one.to_be_bytes(), one_be);
    }

    #[test]
    fn int64_endianness() {
        let be_bytes = [0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8];
        let le_bytes = [3u8, 2u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8];

        // These should all be the same.
        let num1 = Int64::from_be_bytes(be_bytes);
        let num2 = Int64::from_le_bytes(le_bytes);
        assert_eq!(num1, Int64::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
    }

    #[test]
    fn int64_convert_from() {
        let a = Int64::from(5i64);
        assert_eq!(a.0, i64::from(5u32));

        let a = Int64::from(5i64);
        assert_eq!(a.0, i64::from(5u32));

        let a = Int64::from(5u32);
        assert_eq!(a.0, i64::from(5u32));

        let a = Int64::from(5u16);
        assert_eq!(a.0, i64::from(5u32));

        let a = Int64::from(5u8);
        assert_eq!(a.0, i64::from(5u32));

        let a = Int64::from(-5i64);
        assert_eq!(a.0, i64::from(-5i32));

        let a = Int64::from(-5i64);
        assert_eq!(a.0, i64::from(-5i32));

        let a = Int64::from(-5i32);
        assert_eq!(a.0, i64::from(-5i32));

        let a = Int64::from(-5i16);
        assert_eq!(a.0, i64::from(-5i32));

        let a = Int64::from(-5i8);
        assert_eq!(a.0, i64::from(-5i32));

        let result = Int64::try_from("34567");
        assert_eq!(result.unwrap().0, "34567".parse::<i64>().unwrap());

        let result = Int64::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn int64_try_from_unsigned_works() {
        test_try_from_uint_to_int::<Uint64, Int64>("Uint64", "Int64");
        test_try_from_uint_to_int::<Uint128, Int64>("Uint128", "Int64");
        test_try_from_uint_to_int::<Uint256, Int64>("Uint256", "Int64");
        test_try_from_uint_to_int::<Uint512, Int64>("Uint512", "Int64");
    }

    #[test]
    fn int64_implements_display() {
        let a = Int64::from(12345u32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Int64::from(-12345i32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: -12345");
        assert_eq!(a.to_string(), "-12345");

        let a = Int64::zero();
        assert_eq!(format!("Embedded: {a}"), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn int64_display_padding_works() {
        // width > natural representation
        let a = Int64::from(123i64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: 00123");
        let a = Int64::from(-123i64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: -0123");

        // width < natural representation
        let a = Int64::from(123i64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: 123");
        let a = Int64::from(-123i64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: -123");
    }

    #[test]
    fn int64_to_be_bytes_works() {
        assert_eq!(Int64::zero().to_be_bytes(), [0; 8]);

        let mut max = [0xff; 8];
        max[0] = 0x7f;
        assert_eq!(Int64::MAX.to_be_bytes(), max);

        let mut one = [0; 8];
        one[7] = 1;
        assert_eq!(Int64::from(1i64).to_be_bytes(), one);
        // Python: `[b for b in (8535972485454015680).to_bytes(8, "big")]`
        assert_eq!(
            Int64::from(8535972485454015680i64).to_be_bytes(),
            [118, 117, 221, 191, 255, 254, 172, 192]
        );
        assert_eq!(
            Int64::from_be_bytes([17, 4, 23, 32, 87, 67, 123, 200]).to_be_bytes(),
            [17, 4, 23, 32, 87, 67, 123, 200]
        );
    }

    #[test]
    fn int64_to_le_bytes_works() {
        assert_eq!(Int64::zero().to_le_bytes(), [0; 8]);

        let mut max = [0xff; 8];
        max[7] = 0x7f;
        assert_eq!(Int64::MAX.to_le_bytes(), max);

        let mut one = [0; 8];
        one[0] = 1;
        assert_eq!(Int64::from(1i64).to_le_bytes(), one);
        // Python: `[b for b in (8535972485454015680).to_bytes(8, "little")]`
        assert_eq!(
            Int64::from(8535972485454015680i64).to_le_bytes(),
            [192, 172, 254, 255, 191, 221, 117, 118]
        );
        assert_eq!(
            Int64::from_be_bytes([17, 4, 23, 32, 87, 67, 123, 200]).to_le_bytes(),
            [200, 123, 67, 87, 32, 23, 4, 17]
        );
    }

    #[test]
    fn int64_is_zero_works() {
        assert!(Int64::zero().is_zero());
        assert!(Int64(i64::from(0u32)).is_zero());

        assert!(!Int64::from(1u32).is_zero());
        assert!(!Int64::from(123u32).is_zero());
        assert!(!Int64::from(-123i32).is_zero());
    }

    #[test]
    fn int64_is_negative_works() {
        assert!(Int64::MIN.is_negative());
        assert!(Int64::from(-123i32).is_negative());

        assert!(!Int64::MAX.is_negative());
        assert!(!Int64::zero().is_negative());
        assert!(!Int64::from(123u32).is_negative());
    }

    #[test]
    fn int64_wrapping_methods() {
        // wrapping_add
        assert_eq!(
            Int64::from(2u32).wrapping_add(Int64::from(2u32)),
            Int64::from(4u32)
        ); // non-wrapping
        assert_eq!(Int64::MAX.wrapping_add(Int64::from(1u32)), Int64::MIN); // wrapping

        // wrapping_sub
        assert_eq!(
            Int64::from(7u32).wrapping_sub(Int64::from(5u32)),
            Int64::from(2u32)
        ); // non-wrapping
        assert_eq!(Int64::MIN.wrapping_sub(Int64::from(1u32)), Int64::MAX); // wrapping

        // wrapping_mul
        assert_eq!(
            Int64::from(3u32).wrapping_mul(Int64::from(2u32)),
            Int64::from(6u32)
        ); // non-wrapping
        assert_eq!(
            Int64::MAX.wrapping_mul(Int64::from(2u32)),
            Int64::from(-2i32)
        ); // wrapping

        // wrapping_pow
        assert_eq!(Int64::from(2u32).wrapping_pow(3), Int64::from(8u32)); // non-wrapping
        assert_eq!(Int64::MAX.wrapping_pow(2), Int64::from(1u32)); // wrapping
    }

    #[test]
    fn int64_json() {
        let orig = Int64::from(1234567890987654321i64);
        let serialized = to_json_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Int64 = from_json(serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn int64_compare() {
        let a = Int64::from(12345u32);
        let b = Int64::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Int64::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int64_math() {
        let a = Int64::from(-12345i32);
        let b = Int64::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Int64::from(11111u32));
        assert_eq!(a + &b, Int64::from(11111u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Int64::from(35801u32));
        assert_eq!(b - &a, Int64::from(35801u32));

        // test += with owned and reference right hand side
        let mut c = Int64::from(300000u32);
        c += b;
        assert_eq!(c, Int64::from(323456u32));
        let mut d = Int64::from(300000u32);
        d += &b;
        assert_eq!(d, Int64::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Int64::from(300000u32);
        c -= b;
        assert_eq!(c, Int64::from(276544u32));
        let mut d = Int64::from(300000u32);
        d -= &b;
        assert_eq!(d, Int64::from(276544u32));

        // test - with negative result
        assert_eq!(a - b, Int64::from(-35801i32));
    }

    #[test]
    #[should_panic]
    fn int64_add_overflow_panics() {
        let _ = Int64::MAX + Int64::from(12u32);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int64_sub_works() {
        assert_eq!(Int64::from(2u32) - Int64::from(1u32), Int64::from(1u32));
        assert_eq!(Int64::from(2u32) - Int64::from(0u32), Int64::from(2u32));
        assert_eq!(Int64::from(2u32) - Int64::from(2u32), Int64::from(0u32));
        assert_eq!(Int64::from(2u32) - Int64::from(3u32), Int64::from(-1i32));

        // works for refs
        let a = Int64::from(10u32);
        let b = Int64::from(3u32);
        let expected = Int64::from(7u32);
        assert_eq!(a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn int64_sub_overflow_panics() {
        let _ = Int64::MIN + Int64::one() - Int64::from(2u32);
    }

    #[test]
    fn int64_sub_assign_works() {
        let mut a = Int64::from(14u32);
        a -= Int64::from(2u32);
        assert_eq!(a, Int64::from(12u32));

        // works for refs
        let mut a = Int64::from(10u32);
        let b = Int64::from(3u32);
        let expected = Int64::from(7u32);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int64_mul_works() {
        assert_eq!(Int64::from(2u32) * Int64::from(3u32), Int64::from(6u32));
        assert_eq!(Int64::from(2u32) * Int64::zero(), Int64::zero());

        // works for refs
        let a = Int64::from(11u32);
        let b = Int64::from(3u32);
        let expected = Int64::from(33u32);
        assert_eq!(a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn int64_mul_assign_works() {
        let mut a = Int64::from(14u32);
        a *= Int64::from(2u32);
        assert_eq!(a, Int64::from(28u32));

        // works for refs
        let mut a = Int64::from(10u32);
        let b = Int64::from(3u32);
        a *= &b;
        assert_eq!(a, Int64::from(30u32));
    }

    #[test]
    fn int64_pow_works() {
        assert_eq!(Int64::from(2u32).pow(2), Int64::from(4u32));
        assert_eq!(Int64::from(2u32).pow(10), Int64::from(1024u32));
    }

    #[test]
    #[should_panic]
    fn int64_pow_overflow_panics() {
        _ = Int64::MAX.pow(2u32);
    }

    #[test]
    fn int64_checked_multiply_ratio_works() {
        let base = Int64(500);

        // factor 1/1
        assert_eq!(base.checked_multiply_ratio(1i64, 1i64).unwrap(), base);
        assert_eq!(base.checked_multiply_ratio(3i64, 3i64).unwrap(), base);
        assert_eq!(
            base.checked_multiply_ratio(654321i64, 654321i64).unwrap(),
            base
        );
        assert_eq!(
            base.checked_multiply_ratio(i64::MAX, i64::MAX).unwrap(),
            base
        );

        // factor 3/2
        assert_eq!(base.checked_multiply_ratio(3i64, 2i64).unwrap(), Int64(750));
        assert_eq!(
            base.checked_multiply_ratio(333333i64, 222222i64).unwrap(),
            Int64(750)
        );

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(base.checked_multiply_ratio(2i64, 3i64).unwrap(), Int64(333));
        assert_eq!(
            base.checked_multiply_ratio(222222i64, 333333i64).unwrap(),
            Int64(333)
        );

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(base.checked_multiply_ratio(5i64, 6i64).unwrap(), Int64(416));
        assert_eq!(
            base.checked_multiply_ratio(100i64, 120i64).unwrap(),
            Int64(416)
        );
    }

    #[test]
    fn int64_checked_multiply_ratio_does_not_panic() {
        assert_eq!(
            Int64(500i64).checked_multiply_ratio(1i64, 0i64),
            Err(CheckedMultiplyRatioError::DivideByZero),
        );
        assert_eq!(
            Int64(500i64).checked_multiply_ratio(i64::MAX, 1i64),
            Err(CheckedMultiplyRatioError::Overflow),
        );
    }

    #[test]
    fn int64_shr_works() {
        let original = Int64::from_be_bytes([0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8]);

        let shifted = Int64::from_be_bytes([0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn int64_shr_overflow_panics() {
        let _ = Int64::from(1u32) >> 64u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Int64::from(17u32),
            Int64::from(123u32),
            Int64::from(540u32),
            Int64::from(82u32),
        ];
        let expected = Int64::from(762u32);

        let sum_as_ref: Int64 = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned: Int64 = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn int64_methods() {
        // checked_*
        assert!(matches!(
            Int64::MAX.checked_add(Int64::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int64::from(1u32).checked_add(Int64::from(1u32)),
            Ok(Int64::from(2u32)),
        );
        assert!(matches!(
            Int64::MIN.checked_sub(Int64::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int64::from(2u32).checked_sub(Int64::from(1u32)),
            Ok(Int64::from(1u32)),
        );
        assert!(matches!(
            Int64::MAX.checked_mul(Int64::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int64::from(2u32).checked_mul(Int64::from(2u32)),
            Ok(Int64::from(4u32)),
        );
        assert!(matches!(
            Int64::MAX.checked_pow(2u32),
            Err(OverflowError { .. })
        ));
        assert_eq!(Int64::from(2u32).checked_pow(3u32), Ok(Int64::from(8u32)),);
        assert_eq!(
            Int64::MAX.checked_div(Int64::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int64::from(6u32).checked_div(Int64::from(2u32)),
            Ok(Int64::from(3u32)),
        );
        assert_eq!(
            Int64::MAX.checked_div_euclid(Int64::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int64::from(6u32).checked_div_euclid(Int64::from(2u32)),
            Ok(Int64::from(3u32)),
        );
        assert_eq!(
            Int64::from(7u32).checked_div_euclid(Int64::from(2u32)),
            Ok(Int64::from(3u32)),
        );
        assert!(matches!(
            Int64::MAX.checked_rem(Int64::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        // checked_* with negative numbers
        assert_eq!(
            Int64::from(-12i32).checked_div(Int64::from(10i32)),
            Ok(Int64::from(-1i32)),
        );
        assert_eq!(Int64::from(-2i32).checked_pow(3u32), Ok(Int64::from(-8i32)),);
        assert_eq!(
            Int64::from(-6i32).checked_mul(Int64::from(-7i32)),
            Ok(Int64::from(42i32)),
        );
        assert_eq!(
            Int64::from(-2i32).checked_add(Int64::from(3i32)),
            Ok(Int64::from(1i32)),
        );
        assert_eq!(
            Int64::from(-1i32).checked_div_euclid(Int64::from(-2i32)),
            Ok(Int64::from(1u32)),
        );

        // saturating_*
        assert_eq!(Int64::MAX.saturating_add(Int64::from(1u32)), Int64::MAX);
        assert_eq!(Int64::MIN.saturating_sub(Int64::from(1u32)), Int64::MIN);
        assert_eq!(Int64::MAX.saturating_mul(Int64::from(2u32)), Int64::MAX);
        assert_eq!(Int64::from(4u32).saturating_pow(2u32), Int64::from(16u32));
        assert_eq!(Int64::MAX.saturating_pow(2u32), Int64::MAX);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int64_implements_rem() {
        let a = Int64::from(10u32);
        assert_eq!(a % Int64::from(10u32), Int64::zero());
        assert_eq!(a % Int64::from(2u32), Int64::zero());
        assert_eq!(a % Int64::from(1u32), Int64::zero());
        assert_eq!(a % Int64::from(3u32), Int64::from(1u32));
        assert_eq!(a % Int64::from(4u32), Int64::from(2u32));

        assert_eq!(Int64::from(-12i32) % Int64::from(10i32), Int64::from(-2i32));
        assert_eq!(Int64::from(12i32) % Int64::from(-10i32), Int64::from(2i32));
        assert_eq!(
            Int64::from(-12i32) % Int64::from(-10i32),
            Int64::from(-2i32)
        );

        // works for refs
        let a = Int64::from(10u32);
        let b = Int64::from(3u32);
        let expected = Int64::from(1u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn int64_rem_panics_for_zero() {
        let _ = Int64::from(10u32) % Int64::zero();
    }

    #[test]
    fn int64_rem_assign_works() {
        let mut a = Int64::from(30u32);
        a %= Int64::from(4u32);
        assert_eq!(a, Int64::from(2u32));

        // works for refs
        let mut a = Int64::from(25u32);
        let b = Int64::from(6u32);
        a %= &b;
        assert_eq!(a, Int64::from(1u32));
    }

    #[test]
    fn int64_shr() {
        let x: Int64 = 0x4000_0000_0000_0000i64.into();
        assert_eq!(x >> 0, x); // right shift by 0 should be no-op
        assert_eq!(x >> 1, Int64::from(0x2000_0000_0000_0000i64));
        assert_eq!(x >> 4, Int64::from(0x0400_0000_0000_0000i64));
        // right shift of MIN value by the maximum shift value should result in -1 (filled with 1s)
        assert_eq!(
            Int64::MIN >> (core::mem::size_of::<Int64>() as u32 * 8 - 1),
            -Int64::one()
        );
    }

    #[test]
    fn int64_shl() {
        let x: Int64 = 0x0800_0000_0000_0000i64.into();
        assert_eq!(x << 0, x); // left shift by 0 should be no-op
        assert_eq!(x << 1, Int64::from(0x1000_0000_0000_0000i64));
        assert_eq!(x << 4, Int64::from(0x0800_0000_0000_0000i64 << 4));
        // left shift by by the maximum shift value should result in MIN
        assert_eq!(
            Int64::one() << (core::mem::size_of::<Int64>() as u32 * 8 - 1),
            Int64::MIN
        );
    }

    #[test]
    fn int64_abs_diff_works() {
        let a = Int64::from(42u32);
        let b = Int64::from(5u32);
        let expected = Uint64::from(37u32);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let c = Int64::from(-5i32);
        assert_eq!(b.abs_diff(c), Uint64::from(10u32));
        assert_eq!(c.abs_diff(b), Uint64::from(10u32));
    }

    #[test]
    fn int64_abs_works() {
        let a = Int64::from(42i32);
        assert_eq!(a.abs(), a);

        let b = Int64::from(-42i32);
        assert_eq!(b.abs(), a);

        assert_eq!(Int64::zero().abs(), Int64::zero());
        assert_eq!((Int64::MIN + Int64::one()).abs(), Int64::MAX);
    }

    #[test]
    fn int64_unsigned_abs_works() {
        assert_eq!(Int64::zero().unsigned_abs(), Uint64::zero());
        assert_eq!(Int64::one().unsigned_abs(), Uint64::one());
        assert_eq!(
            Int64::MIN.unsigned_abs(),
            Uint64::new(Int64::MAX.0 as u64) + Uint64::one()
        );

        let v = Int64::from(-42i32);
        assert_eq!(v.unsigned_abs(), v.abs_diff(Int64::zero()));
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int64_abs_min_panics() {
        _ = Int64::MIN.abs();
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int64_neg_min_panics() {
        _ = -Int64::MIN;
    }

    #[test]
    fn int64_partial_eq() {
        let test_cases = [(1, 1, true), (42, 42, true), (42, 24, false), (0, 0, true)]
            .into_iter()
            .map(|(lhs, rhs, expected): (i64, i64, bool)| {
                (Int64::from(lhs), Int64::from(rhs), expected)
            });

        #[allow(clippy::op_ref)]
        for (lhs, rhs, expected) in test_cases {
            assert_eq!(lhs == rhs, expected);
            assert_eq!(&lhs == rhs, expected);
            assert_eq!(lhs == &rhs, expected);
            assert_eq!(&lhs == &rhs, expected);
        }
    }
}
