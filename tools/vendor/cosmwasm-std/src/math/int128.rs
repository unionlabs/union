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
    forward_ref_partial_eq, CheckedMultiplyRatioError, Int256, Int512, Int64, Uint128, Uint256,
    Uint512, Uint64,
};

use super::conversion::{forward_try_from, try_from_int_to_int};
use super::num_consts::NumConsts;

/// An implementation of i128 that is using strings for JSON encoding/decoding,
/// such that the full i128 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances of this and `i128` to get the value out:
///
/// ```
/// # use cosmwasm_std::Int128;
/// let a = Int128::from(258i128);
/// assert_eq!(a.i128(), 258);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Int128(#[schemars(with = "String")] pub(crate) i128);

forward_ref_partial_eq!(Int128, Int128);

impl Int128 {
    pub const MAX: Int128 = Int128(i128::MAX);
    pub const MIN: Int128 = Int128(i128::MIN);

    /// Creates a Int128(value).
    ///
    /// This method is less flexible than `from` but can be called in a const context.
    #[inline]
    pub const fn new(value: i128) -> Self {
        Self(value)
    }

    /// Creates a Int128(0)
    #[inline]
    pub const fn zero() -> Self {
        Int128(0)
    }

    /// Creates a Int128(1)
    #[inline]
    pub const fn one() -> Self {
        Self(1)
    }

    /// Returns a copy of the internal data
    pub const fn i128(&self) -> i128 {
        self.0
    }

    #[must_use]
    pub const fn from_be_bytes(data: [u8; 16]) -> Self {
        Self(i128::from_be_bytes(data))
    }

    #[must_use]
    pub const fn from_le_bytes(data: [u8; 16]) -> Self {
        Self(i128::from_le_bytes(data))
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
        match (self.full_mul(numerator) / Int256::from(denominator)).try_into() {
            Ok(ratio) => Ok(ratio),
            Err(_) => Err(CheckedMultiplyRatioError::Overflow),
        }
    }

    /// Multiplies two [`Int128`] values without overflow, producing an
    /// [`Int256`].
    ///
    /// # Examples
    ///
    /// ```
    /// use cosmwasm_std::Int128;
    ///
    /// let a = Int128::MAX;
    /// let result = a.full_mul(2i32);
    /// assert_eq!(result.to_string(), "340282366920938463463374607431768211454");
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn full_mul(self, rhs: impl Into<Self>) -> Int256 {
        Int256::from(self.i128())
            .checked_mul(Int256::from(rhs.into()))
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
    pub const fn abs_diff(self, other: Self) -> Uint128 {
        Uint128(self.0.abs_diff(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn unsigned_abs(self) -> Uint128 {
        Uint128(self.0.unsigned_abs())
    }
}

impl NumConsts for Int128 {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
}

// Uint to Int
impl From<Uint64> for Int128 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}
forward_try_from!(Uint128, Int128);
forward_try_from!(Uint256, Int128);
forward_try_from!(Uint512, Int128);

// uint to Int
impl From<u64> for Int128 {
    fn from(val: u64) -> Self {
        Int128(val.into())
    }
}

impl From<u32> for Int128 {
    fn from(val: u32) -> Self {
        Int128(val.into())
    }
}

impl From<u16> for Int128 {
    fn from(val: u16) -> Self {
        Int128(val.into())
    }
}

impl From<u8> for Int128 {
    fn from(val: u8) -> Self {
        Int128(val.into())
    }
}

// Int to Int
impl From<Int64> for Int128 {
    fn from(val: Int64) -> Self {
        val.i64().into()
    }
}

try_from_int_to_int!(Int256, Int128);
try_from_int_to_int!(Int512, Int128);

// int to Int
impl From<i128> for Int128 {
    fn from(val: i128) -> Self {
        Int128(val)
    }
}

impl From<i64> for Int128 {
    fn from(val: i64) -> Self {
        Int128(val.into())
    }
}

impl From<i32> for Int128 {
    fn from(val: i32) -> Self {
        Int128(val.into())
    }
}

impl From<i16> for Int128 {
    fn from(val: i16) -> Self {
        Int128(val.into())
    }
}

impl From<i8> for Int128 {
    fn from(val: i8) -> Self {
        Int128(val.into())
    }
}

impl TryFrom<&str> for Int128 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Int128 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.parse::<i128>() {
            Ok(u) => Ok(Self(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing Int128: {e}"))),
        }
    }
}

impl From<Int128> for String {
    fn from(original: Int128) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Int128 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<Int128> for Int128 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Int128(self.0.checked_add(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Add, add for Int128, Int128);

impl Sub<Int128> for Int128 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Int128(self.0.checked_sub(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Sub, sub for Int128, Int128);

impl SubAssign<Int128> for Int128 {
    fn sub_assign(&mut self, rhs: Int128) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Int128, Int128);

impl Div<Int128> for Int128 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Div, div for Int128, Int128);

impl Rem for Int128 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero.
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Int128, Int128);

impl Not for Int128 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Neg for Int128 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl RemAssign<Int128> for Int128 {
    fn rem_assign(&mut self, rhs: Int128) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Int128, Int128);

impl Mul<Int128> for Int128 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Mul, mul for Int128, Int128);

impl MulAssign<Int128> for Int128 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Int128, Int128);

impl Shr<u32> for Int128 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!("right shift error: {rhs} is larger or equal than the number of bits in Int128",)
        })
    }
}
forward_ref_binop!(impl Shr, shr for Int128, u32);

impl Shl<u32> for Int128 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs).unwrap_or_else(|_| {
            panic!("left shift error: {rhs} is larger or equal than the number of bits in Int128",)
        })
    }
}
forward_ref_binop!(impl Shl, shl for Int128, u32);

impl AddAssign<Int128> for Int128 {
    fn add_assign(&mut self, rhs: Int128) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Int128, Int128);

impl DivAssign<Int128> for Int128 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Int128, Int128);

impl ShrAssign<u32> for Int128 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShrAssign, shr_assign for Int128, u32);

impl ShlAssign<u32> for Int128 {
    fn shl_assign(&mut self, rhs: u32) {
        *self = Shl::<u32>::shl(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShlAssign, shl_assign for Int128, u32);

impl Serialize for Int128 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Int128 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Int128, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Int128Visitor)
    }
}

struct Int128Visitor;

impl<'de> de::Visitor<'de> for Int128Visitor {
    type Value = Int128;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Int128::try_from(v).map_err(|e| E::custom(format!("invalid Int128 '{v}' - {e}")))
    }
}

impl<A> core::iter::Sum<A> for Int128
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
        assert_eq!(core::mem::size_of::<Int128>(), 16);
    }

    #[test]
    fn int128_from_be_bytes_works() {
        let num = Int128::from_be_bytes([1; 16]);
        let a: [u8; 16] = num.to_be_bytes();
        assert_eq!(a, [1; 16]);

        let be_bytes = [
            0u8, 222u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let num = Int128::from_be_bytes(be_bytes);
        let resulting_bytes: [u8; 16] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn int128_new_works() {
        let num = Int128::new(222);
        assert_eq!(num.i128(), 222);

        let num = Int128::new(-222);
        assert_eq!(num.i128(), -222);

        let num = Int128::new(i128::MAX);
        assert_eq!(num.i128(), i128::MAX);

        let num = Int128::new(i128::MIN);
        assert_eq!(num.i128(), i128::MIN);
    }

    #[test]
    fn int128_not_works() {
        assert_eq!(!Int128::new(222), Int128::new(!222));
        assert_eq!(!Int128::new(-222), Int128::new(!-222));

        assert_eq!(!Int128::MAX, Int128::new(!i128::MAX));
        assert_eq!(!Int128::MIN, Int128::new(!i128::MIN));
    }

    #[test]
    fn int128_zero_works() {
        let zero = Int128::zero();
        assert_eq!(zero.to_be_bytes(), [0; 16]);
    }

    #[test]
    fn uint128_one_works() {
        let one = Int128::one();
        let mut one_be = [0; 16];
        one_be[15] = 1;

        assert_eq!(one.to_be_bytes(), one_be);
    }

    #[test]
    fn int128_endianness() {
        let be_bytes = [
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let le_bytes = [
            3u8, 2u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ];

        // These should all be the same.
        let num1 = Int128::from_be_bytes(be_bytes);
        let num2 = Int128::from_le_bytes(le_bytes);
        assert_eq!(num1, Int128::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
    }

    #[test]
    fn int128_convert_from() {
        let a = Int128::from(5i128);
        assert_eq!(a.0, i128::from(5u32));

        let a = Int128::from(5u64);
        assert_eq!(a.0, i128::from(5u32));

        let a = Int128::from(5u32);
        assert_eq!(a.0, i128::from(5u32));

        let a = Int128::from(5u16);
        assert_eq!(a.0, i128::from(5u32));

        let a = Int128::from(5u8);
        assert_eq!(a.0, i128::from(5u32));

        let a = Int128::from(-5i128);
        assert_eq!(a.0, i128::from(-5i32));

        let a = Int128::from(-5i64);
        assert_eq!(a.0, i128::from(-5i32));

        let a = Int128::from(-5i32);
        assert_eq!(a.0, i128::from(-5i32));

        let a = Int128::from(-5i16);
        assert_eq!(a.0, i128::from(-5i32));

        let a = Int128::from(-5i8);
        assert_eq!(a.0, i128::from(-5i32));

        let result = Int128::try_from("34567");
        assert_eq!(result.unwrap().0, "34567".parse::<i128>().unwrap());

        let result = Int128::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn int128_try_from_unsigned_works() {
        test_try_from_uint_to_int::<Uint128, Int128>("Uint128", "Int128");
        test_try_from_uint_to_int::<Uint256, Int128>("Uint256", "Int128");
        test_try_from_uint_to_int::<Uint512, Int128>("Uint512", "Int128");
    }

    #[test]
    fn int128_implements_display() {
        let a = Int128::from(12345u32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Int128::from(-12345i32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: -12345");
        assert_eq!(a.to_string(), "-12345");

        let a = Int128::zero();
        assert_eq!(format!("Embedded: {a}"), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn int128_display_padding_works() {
        // width > natural representation
        let a = Int128::from(123u64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: 00123");
        let a = Int128::from(-123i64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: -0123");

        // width < natural representation
        let a = Int128::from(123u64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: 123");
        let a = Int128::from(-123i64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: -123");
    }

    #[test]
    fn int128_to_be_bytes_works() {
        assert_eq!(Int128::zero().to_be_bytes(), [0; 16]);

        let mut max = [0xff; 16];
        max[0] = 0x7f;
        assert_eq!(Int128::MAX.to_be_bytes(), max);

        let mut one = [0; 16];
        one[15] = 1;
        assert_eq!(Int128::from(1i128).to_be_bytes(), one);
        // Python: `[b for b in (70141183460469231731687303715884018880).to_bytes(16, "big")]`
        assert_eq!(
            Int128::from(70141183460469231731687303715884018880i128).to_be_bytes(),
            [52, 196, 179, 87, 165, 121, 59, 133, 246, 117, 221, 191, 255, 254, 172, 192]
        );
        assert_eq!(
            Int128::from_be_bytes([17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78])
                .to_be_bytes(),
            [17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78]
        );
    }

    #[test]
    fn int128_to_le_bytes_works() {
        assert_eq!(Int128::zero().to_le_bytes(), [0; 16]);

        let mut max = [0xff; 16];
        max[15] = 0x7f;
        assert_eq!(Int128::MAX.to_le_bytes(), max);

        let mut one = [0; 16];
        one[0] = 1;
        assert_eq!(Int128::from(1i128).to_le_bytes(), one);
        // Python: `[b for b in (70141183460469231731687303715884018880).to_bytes(16, "little")]`
        assert_eq!(
            Int128::from(70141183460469231731687303715884018880i128).to_le_bytes(),
            [192, 172, 254, 255, 191, 221, 117, 246, 133, 59, 121, 165, 87, 179, 196, 52]
        );
        assert_eq!(
            Int128::from_be_bytes([17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78])
                .to_le_bytes(),
            [78, 67, 21, 33, 38, 0, 91, 58, 200, 123, 67, 87, 32, 23, 4, 17]
        );
    }

    #[test]
    fn int128_is_zero_works() {
        assert!(Int128::zero().is_zero());
        assert!(Int128(i128::from(0u32)).is_zero());

        assert!(!Int128::from(1u32).is_zero());
        assert!(!Int128::from(123u32).is_zero());
        assert!(!Int128::from(-123i32).is_zero());
    }

    #[test]
    fn int128_is_negative_works() {
        assert!(Int128::MIN.is_negative());
        assert!(Int128::from(-123i32).is_negative());

        assert!(!Int128::MAX.is_negative());
        assert!(!Int128::zero().is_negative());
        assert!(!Int128::from(123u32).is_negative());
    }

    #[test]
    fn int128_wrapping_methods() {
        // wrapping_add
        assert_eq!(
            Int128::from(2u32).wrapping_add(Int128::from(2u32)),
            Int128::from(4u32)
        ); // non-wrapping
        assert_eq!(Int128::MAX.wrapping_add(Int128::from(1u32)), Int128::MIN); // wrapping

        // wrapping_sub
        assert_eq!(
            Int128::from(7u32).wrapping_sub(Int128::from(5u32)),
            Int128::from(2u32)
        ); // non-wrapping
        assert_eq!(Int128::MIN.wrapping_sub(Int128::from(1u32)), Int128::MAX); // wrapping

        // wrapping_mul
        assert_eq!(
            Int128::from(3u32).wrapping_mul(Int128::from(2u32)),
            Int128::from(6u32)
        ); // non-wrapping
        assert_eq!(
            Int128::MAX.wrapping_mul(Int128::from(2u32)),
            Int128::from(-2i32)
        ); // wrapping

        // wrapping_pow
        assert_eq!(Int128::from(2u32).wrapping_pow(3), Int128::from(8u32)); // non-wrapping
        assert_eq!(Int128::MAX.wrapping_pow(2), Int128::from(1u32)); // wrapping
    }

    #[test]
    fn int128_json() {
        let orig = Int128::from(1234567890987654321i128);
        let serialized = to_json_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Int128 = from_json(serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn int128_compare() {
        let a = Int128::from(12345u32);
        let b = Int128::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Int128::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int128_math() {
        let a = Int128::from(-12345i32);
        let b = Int128::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Int128::from(11111u32));
        assert_eq!(a + &b, Int128::from(11111u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Int128::from(35801u32));
        assert_eq!(b - &a, Int128::from(35801u32));

        // test += with owned and reference right hand side
        let mut c = Int128::from(300000u32);
        c += b;
        assert_eq!(c, Int128::from(323456u32));
        let mut d = Int128::from(300000u32);
        d += &b;
        assert_eq!(d, Int128::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Int128::from(300000u32);
        c -= b;
        assert_eq!(c, Int128::from(276544u32));
        let mut d = Int128::from(300000u32);
        d -= &b;
        assert_eq!(d, Int128::from(276544u32));

        // test - with negative result
        assert_eq!(a - b, Int128::from(-35801i32));
    }

    #[test]
    #[should_panic]
    fn int128_add_overflow_panics() {
        let _ = Int128::MAX + Int128::from(12u32);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int128_sub_works() {
        assert_eq!(Int128::from(2u32) - Int128::from(1u32), Int128::from(1u32));
        assert_eq!(Int128::from(2u32) - Int128::from(0u32), Int128::from(2u32));
        assert_eq!(Int128::from(2u32) - Int128::from(2u32), Int128::from(0u32));
        assert_eq!(Int128::from(2u32) - Int128::from(3u32), Int128::from(-1i32));

        // works for refs
        let a = Int128::from(10u32);
        let b = Int128::from(3u32);
        let expected = Int128::from(7u32);
        assert_eq!(a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn int128_sub_overflow_panics() {
        let _ = Int128::MIN + Int128::one() - Int128::from(2u32);
    }

    #[test]
    fn int128_sub_assign_works() {
        let mut a = Int128::from(14u32);
        a -= Int128::from(2u32);
        assert_eq!(a, Int128::from(12u32));

        // works for refs
        let mut a = Int128::from(10u32);
        let b = Int128::from(3u32);
        let expected = Int128::from(7u32);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int128_mul_works() {
        assert_eq!(Int128::from(2u32) * Int128::from(3u32), Int128::from(6u32));
        assert_eq!(Int128::from(2u32) * Int128::zero(), Int128::zero());

        // works for refs
        let a = Int128::from(11u32);
        let b = Int128::from(3u32);
        let expected = Int128::from(33u32);
        assert_eq!(a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn int128_mul_assign_works() {
        let mut a = Int128::from(14u32);
        a *= Int128::from(2u32);
        assert_eq!(a, Int128::from(28u32));

        // works for refs
        let mut a = Int128::from(10u32);
        let b = Int128::from(3u32);
        a *= &b;
        assert_eq!(a, Int128::from(30u32));
    }

    #[test]
    fn int128_pow_works() {
        assert_eq!(Int128::from(2u32).pow(2), Int128::from(4u32));
        assert_eq!(Int128::from(2u32).pow(10), Int128::from(1024u32));
    }

    #[test]
    #[should_panic]
    fn int128_pow_overflow_panics() {
        _ = Int128::MAX.pow(2u32);
    }

    #[test]
    fn int128_checked_multiply_ratio_works() {
        let base = Int128(500);

        // factor 1/1
        assert_eq!(base.checked_multiply_ratio(1i128, 1i128).unwrap(), base);
        assert_eq!(base.checked_multiply_ratio(3i128, 3i128).unwrap(), base);
        assert_eq!(
            base.checked_multiply_ratio(654321i128, 654321i128).unwrap(),
            base
        );
        assert_eq!(
            base.checked_multiply_ratio(i128::MAX, i128::MAX).unwrap(),
            base
        );

        // factor 3/2
        assert_eq!(
            base.checked_multiply_ratio(3i128, 2i128).unwrap(),
            Int128(750)
        );
        assert_eq!(
            base.checked_multiply_ratio(333333i128, 222222i128).unwrap(),
            Int128(750)
        );

        // factor 2/3 (integer devision always floors the result)
        assert_eq!(
            base.checked_multiply_ratio(2i128, 3i128).unwrap(),
            Int128(333)
        );
        assert_eq!(
            base.checked_multiply_ratio(222222i128, 333333i128).unwrap(),
            Int128(333)
        );

        // factor 5/6 (integer devision always floors the result)
        assert_eq!(
            base.checked_multiply_ratio(5i128, 6i128).unwrap(),
            Int128(416)
        );
        assert_eq!(
            base.checked_multiply_ratio(100i128, 120i128).unwrap(),
            Int128(416)
        );
    }

    #[test]
    fn int128_checked_multiply_ratio_does_not_panic() {
        assert_eq!(
            Int128(500i128).checked_multiply_ratio(1i128, 0i128),
            Err(CheckedMultiplyRatioError::DivideByZero),
        );
        assert_eq!(
            Int128(500i128).checked_multiply_ratio(i128::MAX, 1i128),
            Err(CheckedMultiplyRatioError::Overflow),
        );
    }

    #[test]
    fn int128_shr_works() {
        let original = Int128::from_be_bytes([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]);

        let shifted = Int128::from_be_bytes([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn int128_shr_overflow_panics() {
        let _ = Int128::from(1u32) >> 128u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Int128::from(17u32),
            Int128::from(123u32),
            Int128::from(540u32),
            Int128::from(82u32),
        ];
        let expected = Int128::from(762u32);

        let sum_as_ref: Int128 = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned: Int128 = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn int128_methods() {
        // checked_*
        assert!(matches!(
            Int128::MAX.checked_add(Int128::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int128::from(1u32).checked_add(Int128::from(1u32)),
            Ok(Int128::from(2u32)),
        );
        assert!(matches!(
            Int128::MIN.checked_sub(Int128::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int128::from(2u32).checked_sub(Int128::from(1u32)),
            Ok(Int128::from(1u32)),
        );
        assert!(matches!(
            Int128::MAX.checked_mul(Int128::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int128::from(2u32).checked_mul(Int128::from(2u32)),
            Ok(Int128::from(4u32)),
        );
        assert!(matches!(
            Int128::MAX.checked_pow(2u32),
            Err(OverflowError { .. })
        ));
        assert_eq!(Int128::from(2u32).checked_pow(3u32), Ok(Int128::from(8u32)),);
        assert_eq!(
            Int128::MAX.checked_div(Int128::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int128::from(6u32).checked_div(Int128::from(2u32)),
            Ok(Int128::from(3u32)),
        );
        assert_eq!(
            Int128::MAX.checked_div_euclid(Int128::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int128::from(6u32).checked_div_euclid(Int128::from(2u32)),
            Ok(Int128::from(3u32)),
        );
        assert_eq!(
            Int128::from(7u32).checked_div_euclid(Int128::from(2u32)),
            Ok(Int128::from(3u32)),
        );
        assert!(matches!(
            Int128::MAX.checked_rem(Int128::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        // checked_* with negative numbers
        assert_eq!(
            Int128::from(-12i32).checked_div(Int128::from(10i32)),
            Ok(Int128::from(-1i32)),
        );
        assert_eq!(
            Int128::from(-2i32).checked_pow(3u32),
            Ok(Int128::from(-8i32)),
        );
        assert_eq!(
            Int128::from(-6i32).checked_mul(Int128::from(-7i32)),
            Ok(Int128::from(42i32)),
        );
        assert_eq!(
            Int128::from(-2i32).checked_add(Int128::from(3i32)),
            Ok(Int128::from(1i32)),
        );
        assert_eq!(
            Int128::from(-1i32).checked_div_euclid(Int128::from(-2i32)),
            Ok(Int128::from(1u32)),
        );

        // saturating_*
        assert_eq!(Int128::MAX.saturating_add(Int128::from(1u32)), Int128::MAX);
        assert_eq!(Int128::MIN.saturating_sub(Int128::from(1u32)), Int128::MIN);
        assert_eq!(Int128::MAX.saturating_mul(Int128::from(2u32)), Int128::MAX);
        assert_eq!(Int128::from(4u32).saturating_pow(2u32), Int128::from(16u32));
        assert_eq!(Int128::MAX.saturating_pow(2u32), Int128::MAX);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int128_implements_rem() {
        let a = Int128::from(10u32);
        assert_eq!(a % Int128::from(10u32), Int128::zero());
        assert_eq!(a % Int128::from(2u32), Int128::zero());
        assert_eq!(a % Int128::from(1u32), Int128::zero());
        assert_eq!(a % Int128::from(3u32), Int128::from(1u32));
        assert_eq!(a % Int128::from(4u32), Int128::from(2u32));

        assert_eq!(
            Int128::from(-12i32) % Int128::from(10i32),
            Int128::from(-2i32)
        );
        assert_eq!(
            Int128::from(12i32) % Int128::from(-10i32),
            Int128::from(2i32)
        );
        assert_eq!(
            Int128::from(-12i32) % Int128::from(-10i32),
            Int128::from(-2i32)
        );

        // works for refs
        let a = Int128::from(10u32);
        let b = Int128::from(3u32);
        let expected = Int128::from(1u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn int128_rem_panics_for_zero() {
        let _ = Int128::from(10u32) % Int128::zero();
    }

    #[test]
    fn int128_rem_assign_works() {
        let mut a = Int128::from(30u32);
        a %= Int128::from(4u32);
        assert_eq!(a, Int128::from(2u32));

        // works for refs
        let mut a = Int128::from(25u32);
        let b = Int128::from(6u32);
        a %= &b;
        assert_eq!(a, Int128::from(1u32));
    }

    #[test]
    fn int128_shr() {
        let x: Int128 = 0x4000_0000_0000_0000_0000_0000_0000_0000i128.into();
        assert_eq!(x >> 0, x); // right shift by 0 should be no-op
        assert_eq!(
            x >> 1,
            Int128::from(0x2000_0000_0000_0000_0000_0000_0000_0000i128)
        );
        assert_eq!(
            x >> 4,
            Int128::from(0x0400_0000_0000_0000_0000_0000_0000_0000i128)
        );
        // right shift of MIN value by the maximum shift value should result in -1 (filled with 1s)
        assert_eq!(
            Int128::MIN >> (core::mem::size_of::<Int128>() as u32 * 8 - 1),
            -Int128::one()
        );
    }

    #[test]
    fn int128_shl() {
        let x: Int128 = 0x0800_0000_0000_0000_0000_0000_0000_0000i128.into();
        assert_eq!(x << 0, x); // left shift by 0 should be no-op
        assert_eq!(
            x << 1,
            Int128::from(0x1000_0000_0000_0000_0000_0000_0000_0000i128)
        );
        assert_eq!(
            x << 4,
            Int128::from(0x0800_0000_0000_0000_0000_0000_0000_0000i128 << 4)
        );
        // left shift by by the maximum shift value should result in MIN
        assert_eq!(
            Int128::one() << (core::mem::size_of::<Int128>() as u32 * 8 - 1),
            Int128::MIN
        );
    }

    #[test]
    fn int128_abs_diff_works() {
        let a = Int128::from(42u32);
        let b = Int128::from(5u32);
        let expected = Uint128::from(37u32);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let c = Int128::from(-5i32);
        assert_eq!(b.abs_diff(c), Uint128::from(10u32));
        assert_eq!(c.abs_diff(b), Uint128::from(10u32));
    }

    #[test]
    fn int128_abs_works() {
        let a = Int128::from(42i32);
        assert_eq!(a.abs(), a);

        let b = Int128::from(-42i32);
        assert_eq!(b.abs(), a);

        assert_eq!(Int128::zero().abs(), Int128::zero());
        assert_eq!((Int128::MIN + Int128::one()).abs(), Int128::MAX);
    }

    #[test]
    fn int128_unsigned_abs_works() {
        assert_eq!(Int128::zero().unsigned_abs(), Uint128::zero());
        assert_eq!(Int128::one().unsigned_abs(), Uint128::one());
        assert_eq!(
            Int128::MIN.unsigned_abs(),
            Uint128::new(Int128::MAX.0 as u128) + Uint128::one()
        );

        let v = Int128::from(-42i32);
        assert_eq!(v.unsigned_abs(), v.abs_diff(Int128::zero()));
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int128_abs_min_panics() {
        _ = Int128::MIN.abs();
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int128_neg_min_panics() {
        _ = -Int128::MIN;
    }

    #[test]
    fn int128_partial_eq() {
        let test_cases = [(1, 1, true), (42, 42, true), (42, 24, false), (0, 0, true)]
            .into_iter()
            .map(|(lhs, rhs, expected): (u64, u64, bool)| {
                (Int128::from(lhs), Int128::from(rhs), expected)
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
