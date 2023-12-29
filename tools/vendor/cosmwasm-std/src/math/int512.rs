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
use crate::{forward_ref_partial_eq, Int128, Int256, Int64, Uint128, Uint256, Uint512, Uint64};

/// Used internally - we don't want to leak this type since we might change
/// the implementation in the future.
use bnum::types::{I512, U512};

use super::conversion::{grow_be_int, try_from_uint_to_int};
use super::num_consts::NumConsts;

/// An implementation of i512 that is using strings for JSON encoding/decoding,
/// such that the full i512 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances out of primitive uint types or `new` to provide big
/// endian bytes:
///
/// ```
/// # use cosmwasm_std::Int512;
/// let a = Int512::from(258u128);
/// let b = Int512::new([
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
///     0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8,
/// ]);
/// assert_eq!(a, b);
/// ```
#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Int512(#[schemars(with = "String")] pub(crate) I512);

forward_ref_partial_eq!(Int512, Int512);

impl Int512 {
    pub const MAX: Int512 = Int512(I512::MAX);
    pub const MIN: Int512 = Int512(I512::MIN);

    /// Creates a Int512(value) from a big endian representation. It's just an alias for
    /// `from_be_bytes`.
    #[inline]
    pub const fn new(value: [u8; 64]) -> Self {
        Self::from_be_bytes(value)
    }

    /// Creates a Int512(0)
    #[inline]
    pub const fn zero() -> Self {
        Int512(I512::ZERO)
    }

    /// Creates a Int512(1)
    #[inline]
    pub const fn one() -> Self {
        Self(I512::ONE)
    }

    #[must_use]
    pub const fn from_be_bytes(data: [u8; 64]) -> Self {
        let words: [u64; 8] = [
            u64::from_le_bytes([
                data[63], data[62], data[61], data[60], data[59], data[58], data[57], data[56],
            ]),
            u64::from_le_bytes([
                data[55], data[54], data[53], data[52], data[51], data[50], data[49], data[48],
            ]),
            u64::from_le_bytes([
                data[47], data[46], data[45], data[44], data[43], data[42], data[41], data[40],
            ]),
            u64::from_le_bytes([
                data[39], data[38], data[37], data[36], data[35], data[34], data[33], data[32],
            ]),
            u64::from_le_bytes([
                data[31], data[30], data[29], data[28], data[27], data[26], data[25], data[24],
            ]),
            u64::from_le_bytes([
                data[23], data[22], data[21], data[20], data[19], data[18], data[17], data[16],
            ]),
            u64::from_le_bytes([
                data[15], data[14], data[13], data[12], data[11], data[10], data[9], data[8],
            ]),
            u64::from_le_bytes([
                data[7], data[6], data[5], data[4], data[3], data[2], data[1], data[0],
            ]),
        ];
        Self(I512::from_bits(U512::from_digits(words)))
    }

    #[must_use]
    pub const fn from_le_bytes(data: [u8; 64]) -> Self {
        let words: [u64; 8] = [
            u64::from_le_bytes([
                data[0], data[1], data[2], data[3], data[4], data[5], data[6], data[7],
            ]),
            u64::from_le_bytes([
                data[8], data[9], data[10], data[11], data[12], data[13], data[14], data[15],
            ]),
            u64::from_le_bytes([
                data[16], data[17], data[18], data[19], data[20], data[21], data[22], data[23],
            ]),
            u64::from_le_bytes([
                data[24], data[25], data[26], data[27], data[28], data[29], data[30], data[31],
            ]),
            u64::from_le_bytes([
                data[32], data[33], data[34], data[35], data[36], data[37], data[38], data[39],
            ]),
            u64::from_le_bytes([
                data[40], data[41], data[42], data[43], data[44], data[45], data[46], data[47],
            ]),
            u64::from_le_bytes([
                data[48], data[49], data[50], data[51], data[52], data[53], data[54], data[55],
            ]),
            u64::from_le_bytes([
                data[56], data[57], data[58], data[59], data[60], data[61], data[62], data[63],
            ]),
        ];
        Self(I512::from_bits(U512::from_digits(words)))
    }

    /// Returns a copy of the number as big endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_be_bytes(self) -> [u8; 64] {
        let bits = self.0.to_bits();
        let words = bits.digits();
        let words = [
            words[7].to_be_bytes(),
            words[6].to_be_bytes(),
            words[5].to_be_bytes(),
            words[4].to_be_bytes(),
            words[3].to_be_bytes(),
            words[2].to_be_bytes(),
            words[1].to_be_bytes(),
            words[0].to_be_bytes(),
        ];
        unsafe { core::mem::transmute::<[[u8; 8]; 8], [u8; 64]>(words) }
    }

    /// Returns a copy of the number as little endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_le_bytes(self) -> [u8; 64] {
        let bits = self.0.to_bits();
        let words = bits.digits();
        let words = [
            words[0].to_le_bytes(),
            words[1].to_le_bytes(),
            words[2].to_le_bytes(),
            words[3].to_le_bytes(),
            words[4].to_le_bytes(),
            words[5].to_le_bytes(),
            words[6].to_le_bytes(),
            words[7].to_le_bytes(),
        ];
        unsafe { core::mem::transmute::<[[u8; 8]; 8], [u8; 64]>(words) }
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn pow(self, exp: u32) -> Self {
        Self(self.0.pow(exp))
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
        if other >= 512 {
            return Err(OverflowError::new(OverflowOperation::Shr, self, other));
        }

        Ok(Self(self.0.shr(other)))
    }

    pub fn checked_shl(self, other: u32) -> Result<Self, OverflowError> {
        if other >= 512 {
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
    pub const fn abs_diff(self, other: Self) -> Uint512 {
        Uint512(self.0.abs_diff(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs(self) -> Self {
        Self(self.0.abs())
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn unsigned_abs(self) -> Uint512 {
        Uint512(self.0.unsigned_abs())
    }
}

impl NumConsts for Int512 {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
}

// Uint to Int
try_from_uint_to_int!(Uint512, Int512);

impl From<Uint256> for Int512 {
    fn from(val: Uint256) -> Self {
        let mut bytes = [0u8; 64];
        bytes[32..].copy_from_slice(&val.to_be_bytes());

        Self::from_be_bytes(bytes)
    }
}

impl From<Uint128> for Int512 {
    fn from(val: Uint128) -> Self {
        val.u128().into()
    }
}

impl From<Uint64> for Int512 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}

// uint to Int
impl From<u128> for Int512 {
    fn from(val: u128) -> Self {
        Int512(val.into())
    }
}

impl From<u64> for Int512 {
    fn from(val: u64) -> Self {
        Int512(val.into())
    }
}

impl From<u32> for Int512 {
    fn from(val: u32) -> Self {
        Int512(val.into())
    }
}

impl From<u16> for Int512 {
    fn from(val: u16) -> Self {
        Int512(val.into())
    }
}

impl From<u8> for Int512 {
    fn from(val: u8) -> Self {
        Int512(val.into())
    }
}

// int to Int
impl From<i128> for Int512 {
    fn from(val: i128) -> Self {
        Int512(val.into())
    }
}

impl From<i64> for Int512 {
    fn from(val: i64) -> Self {
        Int512(val.into())
    }
}

impl From<i32> for Int512 {
    fn from(val: i32) -> Self {
        Int512(val.into())
    }
}

impl From<i16> for Int512 {
    fn from(val: i16) -> Self {
        Int512(val.into())
    }
}

impl From<i8> for Int512 {
    fn from(val: i8) -> Self {
        Int512(val.into())
    }
}

// Int to Int
impl From<Int64> for Int512 {
    fn from(val: Int64) -> Self {
        Int512(val.i64().into())
    }
}

impl From<Int128> for Int512 {
    fn from(val: Int128) -> Self {
        Int512(val.i128().into())
    }
}

impl From<Int256> for Int512 {
    fn from(val: Int256) -> Self {
        Self::from_be_bytes(grow_be_int(val.to_be_bytes()))
    }
}

impl TryFrom<&str> for Int512 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Int512 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match I512::from_str_radix(s, 10) {
            Ok(u) => Ok(Self(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing Int512: {e}"))),
        }
    }
}

impl From<Int512> for String {
    fn from(original: Int512) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Int512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<Int512> for Int512 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Int512(self.0.checked_add(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Add, add for Int512, Int512);

impl Sub<Int512> for Int512 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Int512(self.0.checked_sub(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Sub, sub for Int512, Int512);

impl SubAssign<Int512> for Int512 {
    fn sub_assign(&mut self, rhs: Int512) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Int512, Int512);

impl Div<Int512> for Int512 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Div, div for Int512, Int512);

impl Rem for Int512 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero.
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Int512, Int512);

impl Not for Int512 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl Neg for Int512 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl RemAssign<Int512> for Int512 {
    fn rem_assign(&mut self, rhs: Int512) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Int512, Int512);

impl Mul<Int512> for Int512 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Mul, mul for Int512, Int512);

impl MulAssign<Int512> for Int512 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Int512, Int512);

impl Shr<u32> for Int512 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!("right shift error: {rhs} is larger or equal than the number of bits in Int512",)
        })
    }
}
forward_ref_binop!(impl Shr, shr for Int512, u32);

impl Shl<u32> for Int512 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs).unwrap_or_else(|_| {
            panic!("left shift error: {rhs} is larger or equal than the number of bits in Int512",)
        })
    }
}
forward_ref_binop!(impl Shl, shl for Int512, u32);

impl AddAssign<Int512> for Int512 {
    fn add_assign(&mut self, rhs: Int512) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Int512, Int512);

impl DivAssign<Int512> for Int512 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Int512, Int512);

impl ShrAssign<u32> for Int512 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShrAssign, shr_assign for Int512, u32);

impl ShlAssign<u32> for Int512 {
    fn shl_assign(&mut self, rhs: u32) {
        *self = Shl::<u32>::shl(*self, rhs);
    }
}
forward_ref_op_assign!(impl ShlAssign, shl_assign for Int512, u32);

impl Serialize for Int512 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Int512 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Int512, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Int512Visitor)
    }
}

struct Int512Visitor;

impl<'de> de::Visitor<'de> for Int512Visitor {
    type Value = Int512;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Int512::try_from(v).map_err(|e| E::custom(format!("invalid Int512 '{v}' - {e}")))
    }
}

impl<A> core::iter::Sum<A> for Int512
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
        assert_eq!(core::mem::size_of::<Int512>(), 64);
    }

    #[test]
    fn int512_new_works() {
        let num = Int512::new([1; 64]);
        let a: [u8; 64] = num.to_be_bytes();
        assert_eq!(a, [1; 64]);

        let be_bytes = [
            0u8, 222u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let num = Int512::new(be_bytes);
        let resulting_bytes: [u8; 64] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn int512_not_works() {
        let num = Int512::new([1; 64]);
        let a = (!num).to_be_bytes();
        assert_eq!(a, [254; 64]);

        assert_eq!(!Int512::from(-1234806i128), Int512::from(!-1234806i128));

        assert_eq!(!Int512::MAX, Int512::MIN);
        assert_eq!(!Int512::MIN, Int512::MAX);
    }

    #[test]
    fn int512_zero_works() {
        let zero = Int512::zero();
        assert_eq!(zero.to_be_bytes(), [0; 64]);
    }

    #[test]
    fn uint512_one_works() {
        let one = Int512::one();
        let mut one_be = [0; 64];
        one_be[63] = 1;

        assert_eq!(one.to_be_bytes(), one_be);
    }

    #[test]
    fn int512_endianness() {
        let be_bytes = [
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let le_bytes = [
            3u8, 2u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ];

        // These should all be the same.
        let num1 = Int512::new(be_bytes);
        let num2 = Int512::from_be_bytes(be_bytes);
        let num3 = Int512::from_le_bytes(le_bytes);
        assert_eq!(num1, Int512::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
        assert_eq!(num1, num3);
    }

    #[test]
    fn int512_convert_from() {
        let a = Int512::from(5u128);
        assert_eq!(a.0, I512::from(5u32));

        let a = Int512::from(5u64);
        assert_eq!(a.0, I512::from(5u32));

        let a = Int512::from(5u32);
        assert_eq!(a.0, I512::from(5u32));

        let a = Int512::from(5u16);
        assert_eq!(a.0, I512::from(5u32));

        let a = Int512::from(5u8);
        assert_eq!(a.0, I512::from(5u32));

        let a = Int512::from(-5i128);
        assert_eq!(a.0, I512::from(-5i32));

        let a = Int512::from(-5i64);
        assert_eq!(a.0, I512::from(-5i32));

        let a = Int512::from(-5i32);
        assert_eq!(a.0, I512::from(-5i32));

        let a = Int512::from(-5i16);
        assert_eq!(a.0, I512::from(-5i32));

        let a = Int512::from(-5i8);
        assert_eq!(a.0, I512::from(-5i32));

        // other big signed integers
        let values = [
            Int64::MAX,
            Int64::MIN,
            Int64::one(),
            -Int64::one(),
            Int64::zero(),
        ];
        for v in values {
            assert_eq!(Int512::from(v).to_string(), v.to_string());
        }

        let values = [
            Int128::MAX,
            Int128::MIN,
            Int128::one(),
            -Int128::one(),
            Int128::zero(),
        ];
        for v in values {
            assert_eq!(Int512::from(v).to_string(), v.to_string());
        }

        let values = [
            Int256::MAX,
            Int256::MIN,
            Int256::one(),
            -Int256::one(),
            Int256::zero(),
        ];
        for v in values {
            assert_eq!(Int512::from(v).to_string(), v.to_string());
        }

        let result = Int512::try_from("34567");
        assert_eq!(
            result.unwrap().0,
            I512::from_str_radix("34567", 10).unwrap()
        );

        let result = Int512::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn int512_try_from_unsigned_works() {
        test_try_from_uint_to_int::<Uint256, Int256>("Uint256", "Int256");
        test_try_from_uint_to_int::<Uint512, Int256>("Uint512", "Int256");
    }

    #[test]
    fn int512_implements_display() {
        let a = Int512::from(12345u32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Int512::from(-12345i32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: -12345");
        assert_eq!(a.to_string(), "-12345");

        let a = Int512::zero();
        assert_eq!(format!("Embedded: {a}"), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn int512_display_padding_works() {
        // width > natural representation
        let a = Int512::from(123u64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: 00123");
        let a = Int512::from(-123i64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: -0123");

        // width < natural representation
        let a = Int512::from(123u64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: 123");
        let a = Int512::from(-123i64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: -123");
    }

    #[test]
    fn int512_to_be_bytes_works() {
        assert_eq!(Int512::zero().to_be_bytes(), [0; 64]);

        let mut max = [0xff; 64];
        max[0] = 0x7f;
        assert_eq!(Int512::MAX.to_be_bytes(), max);

        let mut one = [0; 64];
        one[63] = 1;
        assert_eq!(Int512::from(1u128).to_be_bytes(), one);
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(64, "big")]`
        assert_eq!(
            Int512::from(240282366920938463463374607431768124608u128).to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 180, 196, 179, 87, 165,
                121, 59, 133, 246, 117, 221, 191, 255, 254, 172, 192
            ]
        );
        assert_eq!(
            Int512::from_be_bytes([
                17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78, 87, 76, 65, 54,
                211, 201, 192, 7, 42, 233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134,
                238, 119, 85, 22, 14, 88, 166, 195, 154, 73, 64, 10, 44, 59, 13, 22, 47, 12, 99, 8,
                252, 96, 230, 187, 38, 29
            ])
            .to_be_bytes(),
            [
                17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78, 87, 76, 65, 54,
                211, 201, 192, 7, 42, 233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134,
                238, 119, 85, 22, 14, 88, 166, 195, 154, 73, 64, 10, 44, 59, 13, 22, 47, 12, 99, 8,
                252, 96, 230, 187, 38, 29
            ]
        );
    }

    #[test]
    fn int512_to_le_bytes_works() {
        assert_eq!(Int512::zero().to_le_bytes(), [0; 64]);

        let mut max = [0xff; 64];
        max[63] = 0x7f;
        assert_eq!(Int512::MAX.to_le_bytes(), max);

        let mut one = [0; 64];
        one[0] = 1;
        assert_eq!(Int512::from(1u128).to_le_bytes(), one);
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(64, "little")]`
        assert_eq!(
            Int512::from(240282366920938463463374607431768124608u128).to_le_bytes(),
            [
                192, 172, 254, 255, 191, 221, 117, 246, 133, 59, 121, 165, 87, 179, 196, 180, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Int512::from_be_bytes([
                17, 4, 23, 32, 87, 67, 123, 200, 58, 91, 0, 38, 33, 21, 67, 78, 87, 76, 65, 54,
                211, 201, 192, 7, 42, 233, 2, 240, 200, 115, 150, 240, 218, 88, 106, 45, 208, 134,
                238, 119, 85, 22, 14, 88, 166, 195, 154, 73, 64, 10, 44, 59, 13, 22, 47, 12, 99, 8,
                252, 96, 230, 187, 38, 29
            ])
            .to_le_bytes(),
            [
                29, 38, 187, 230, 96, 252, 8, 99, 12, 47, 22, 13, 59, 44, 10, 64, 73, 154, 195,
                166, 88, 14, 22, 85, 119, 238, 134, 208, 45, 106, 88, 218, 240, 150, 115, 200, 240,
                2, 233, 42, 7, 192, 201, 211, 54, 65, 76, 87, 78, 67, 21, 33, 38, 0, 91, 58, 200,
                123, 67, 87, 32, 23, 4, 17
            ]
        );
    }

    #[test]
    fn int512_is_zero_works() {
        assert!(Int512::zero().is_zero());
        assert!(Int512(I512::from(0u32)).is_zero());

        assert!(!Int512::from(1u32).is_zero());
        assert!(!Int512::from(123u32).is_zero());
        assert!(!Int512::from(-123i32).is_zero());
    }

    #[test]
    fn int512_is_negative_works() {
        assert!(Int512::MIN.is_negative());
        assert!(Int512::from(-123i32).is_negative());

        assert!(!Int512::MAX.is_negative());
        assert!(!Int512::zero().is_negative());
        assert!(!Int512::from(123u32).is_negative());
    }

    #[test]
    fn int512_wrapping_methods() {
        // wrapping_add
        assert_eq!(
            Int512::from(2u32).wrapping_add(Int512::from(2u32)),
            Int512::from(4u32)
        ); // non-wrapping
        assert_eq!(Int512::MAX.wrapping_add(Int512::from(1u32)), Int512::MIN); // wrapping

        // wrapping_sub
        assert_eq!(
            Int512::from(7u32).wrapping_sub(Int512::from(5u32)),
            Int512::from(2u32)
        ); // non-wrapping
        assert_eq!(Int512::MIN.wrapping_sub(Int512::from(1u32)), Int512::MAX); // wrapping

        // wrapping_mul
        assert_eq!(
            Int512::from(3u32).wrapping_mul(Int512::from(2u32)),
            Int512::from(6u32)
        ); // non-wrapping
        assert_eq!(
            Int512::MAX.wrapping_mul(Int512::from(2u32)),
            Int512::from(-2i32)
        ); // wrapping

        // wrapping_pow
        assert_eq!(Int512::from(2u32).wrapping_pow(3), Int512::from(8u32)); // non-wrapping
        assert_eq!(Int512::MAX.wrapping_pow(2), Int512::from(1u32)); // wrapping
    }

    #[test]
    fn int512_json() {
        let orig = Int512::from(1234567890987654321u128);
        let serialized = to_json_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Int512 = from_json(serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn int512_compare() {
        let a = Int512::from(12345u32);
        let b = Int512::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Int512::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int512_math() {
        let a = Int512::from(-12345i32);
        let b = Int512::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Int512::from(11111u32));
        assert_eq!(a + &b, Int512::from(11111u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Int512::from(35801u32));
        assert_eq!(b - &a, Int512::from(35801u32));

        // test += with owned and reference right hand side
        let mut c = Int512::from(300000u32);
        c += b;
        assert_eq!(c, Int512::from(323456u32));
        let mut d = Int512::from(300000u32);
        d += &b;
        assert_eq!(d, Int512::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Int512::from(300000u32);
        c -= b;
        assert_eq!(c, Int512::from(276544u32));
        let mut d = Int512::from(300000u32);
        d -= &b;
        assert_eq!(d, Int512::from(276544u32));

        // test - with negative result
        assert_eq!(a - b, Int512::from(-35801i32));
    }

    #[test]
    #[should_panic]
    fn int512_add_overflow_panics() {
        let _ = Int512::MAX + Int512::from(12u32);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int512_sub_works() {
        assert_eq!(Int512::from(2u32) - Int512::from(1u32), Int512::from(1u32));
        assert_eq!(Int512::from(2u32) - Int512::from(0u32), Int512::from(2u32));
        assert_eq!(Int512::from(2u32) - Int512::from(2u32), Int512::from(0u32));
        assert_eq!(Int512::from(2u32) - Int512::from(3u32), Int512::from(-1i32));

        // works for refs
        let a = Int512::from(10u32);
        let b = Int512::from(3u32);
        let expected = Int512::from(7u32);
        assert_eq!(a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn int512_sub_overflow_panics() {
        let _ = Int512::MIN + Int512::one() - Int512::from(2u32);
    }

    #[test]
    fn int512_sub_assign_works() {
        let mut a = Int512::from(14u32);
        a -= Int512::from(2u32);
        assert_eq!(a, Int512::from(12u32));

        // works for refs
        let mut a = Int512::from(10u32);
        let b = Int512::from(3u32);
        let expected = Int512::from(7u32);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int512_mul_works() {
        assert_eq!(Int512::from(2u32) * Int512::from(3u32), Int512::from(6u32));
        assert_eq!(Int512::from(2u32) * Int512::zero(), Int512::zero());

        // works for refs
        let a = Int512::from(11u32);
        let b = Int512::from(3u32);
        let expected = Int512::from(33u32);
        assert_eq!(a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn int512_mul_assign_works() {
        let mut a = Int512::from(14u32);
        a *= Int512::from(2u32);
        assert_eq!(a, Int512::from(28u32));

        // works for refs
        let mut a = Int512::from(10u32);
        let b = Int512::from(3u32);
        a *= &b;
        assert_eq!(a, Int512::from(30u32));
    }

    #[test]
    fn int512_pow_works() {
        assert_eq!(Int512::from(2u32).pow(2), Int512::from(4u32));
        assert_eq!(Int512::from(2u32).pow(10), Int512::from(1024u32));
    }

    #[test]
    #[should_panic]
    fn int512_pow_overflow_panics() {
        _ = Int512::MAX.pow(2u32);
    }

    #[test]
    fn int512_shr_works() {
        let original = Int512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]);

        let shifted = Int512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn int512_shr_overflow_panics() {
        let _ = Int512::from(1u32) >> 512u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Int512::from(17u32),
            Int512::from(123u32),
            Int512::from(540u32),
            Int512::from(82u32),
        ];
        let expected = Int512::from(762u32);

        let sum_as_ref: Int512 = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned: Int512 = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn int512_methods() {
        // checked_*
        assert!(matches!(
            Int512::MAX.checked_add(Int512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int512::from(1u32).checked_add(Int512::from(1u32)),
            Ok(Int512::from(2u32)),
        );
        assert!(matches!(
            Int512::MIN.checked_sub(Int512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int512::from(2u32).checked_sub(Int512::from(1u32)),
            Ok(Int512::from(1u32)),
        );
        assert!(matches!(
            Int512::MAX.checked_mul(Int512::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Int512::from(2u32).checked_mul(Int512::from(2u32)),
            Ok(Int512::from(4u32)),
        );
        assert!(matches!(
            Int512::MAX.checked_pow(2u32),
            Err(OverflowError { .. })
        ));
        assert_eq!(Int512::from(2u32).checked_pow(3u32), Ok(Int512::from(8u32)),);
        assert_eq!(
            Int512::MAX.checked_div(Int512::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int512::from(6u32).checked_div(Int512::from(2u32)),
            Ok(Int512::from(3u32)),
        );
        assert_eq!(
            Int512::MAX.checked_div_euclid(Int512::from(0u32)),
            Err(DivisionError::DivideByZero)
        );
        assert_eq!(
            Int512::from(6u32).checked_div_euclid(Int512::from(2u32)),
            Ok(Int512::from(3u32)),
        );
        assert_eq!(
            Int512::from(7u32).checked_div_euclid(Int512::from(2u32)),
            Ok(Int512::from(3u32)),
        );
        assert!(matches!(
            Int512::MAX.checked_rem(Int512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        // checked_* with negative numbers
        assert_eq!(
            Int512::from(-12i32).checked_div(Int512::from(10i32)),
            Ok(Int512::from(-1i32)),
        );
        assert_eq!(
            Int512::from(-2i32).checked_pow(3u32),
            Ok(Int512::from(-8i32)),
        );
        assert_eq!(
            Int512::from(-6i32).checked_mul(Int512::from(-7i32)),
            Ok(Int512::from(42i32)),
        );
        assert_eq!(
            Int512::from(-2i32).checked_add(Int512::from(3i32)),
            Ok(Int512::from(1i32)),
        );
        assert_eq!(
            Int512::from(-1i32).checked_div_euclid(Int512::from(-2i32)),
            Ok(Int512::from(1u32)),
        );

        // saturating_*
        assert_eq!(Int512::MAX.saturating_add(Int512::from(1u32)), Int512::MAX);
        assert_eq!(Int512::MIN.saturating_sub(Int512::from(1u32)), Int512::MIN);
        assert_eq!(Int512::MAX.saturating_mul(Int512::from(2u32)), Int512::MAX);
        assert_eq!(Int512::from(4u32).saturating_pow(2u32), Int512::from(16u32));
        assert_eq!(Int512::MAX.saturating_pow(2u32), Int512::MAX);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn int512_implements_rem() {
        let a = Int512::from(10u32);
        assert_eq!(a % Int512::from(10u32), Int512::zero());
        assert_eq!(a % Int512::from(2u32), Int512::zero());
        assert_eq!(a % Int512::from(1u32), Int512::zero());
        assert_eq!(a % Int512::from(3u32), Int512::from(1u32));
        assert_eq!(a % Int512::from(4u32), Int512::from(2u32));

        assert_eq!(
            Int512::from(-12i32) % Int512::from(10i32),
            Int512::from(-2i32)
        );
        assert_eq!(
            Int512::from(12i32) % Int512::from(-10i32),
            Int512::from(2i32)
        );
        assert_eq!(
            Int512::from(-12i32) % Int512::from(-10i32),
            Int512::from(-2i32)
        );

        // works for refs
        let a = Int512::from(10u32);
        let b = Int512::from(3u32);
        let expected = Int512::from(1u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn int512_rem_panics_for_zero() {
        let _ = Int512::from(10u32) % Int512::zero();
    }

    #[test]
    fn int512_rem_assign_works() {
        let mut a = Int512::from(30u32);
        a %= Int512::from(4u32);
        assert_eq!(a, Int512::from(2u32));

        // works for refs
        let mut a = Int512::from(25u32);
        let b = Int512::from(6u32);
        a %= &b;
        assert_eq!(a, Int512::from(1u32));
    }

    #[test]
    fn int512_shr() {
        let x: Int512 = 0x8000_0000_0000_0000_0000_0000_0000_0000u128.into();
        assert_eq!(x >> 0, x); // right shift by 0 should be no-op
        assert_eq!(
            x >> 1,
            Int512::from(0x4000_0000_0000_0000_0000_0000_0000_0000u128)
        );
        assert_eq!(
            x >> 4,
            Int512::from(0x0800_0000_0000_0000_0000_0000_0000_0000u128)
        );
        // right shift of MIN value by the maximum shift value should result in -1 (filled with 1s)
        assert_eq!(
            Int512::MIN >> (core::mem::size_of::<Int512>() as u32 * 8 - 1),
            -Int512::one()
        );
    }

    #[test]
    fn int512_shl() {
        let x: Int512 = 0x0800_0000_0000_0000_0000_0000_0000_0000u128.into();
        assert_eq!(x << 0, x); // left shift by 0 should be no-op
        assert_eq!(
            x << 1,
            Int512::from(0x1000_0000_0000_0000_0000_0000_0000_0000u128)
        );
        assert_eq!(
            x << 4,
            Int512::from(0x8000_0000_0000_0000_0000_0000_0000_0000u128)
        );
        // left shift by by the maximum shift value should result in MIN
        assert_eq!(
            Int512::one() << (core::mem::size_of::<Int512>() as u32 * 8 - 1),
            Int512::MIN
        );
    }

    #[test]
    fn int512_abs_diff_works() {
        let a = Int512::from(42u32);
        let b = Int512::from(5u32);
        let expected = Uint512::from(37u32);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let c = Int512::from(-5i32);
        assert_eq!(b.abs_diff(c), Uint512::from(10u32));
        assert_eq!(c.abs_diff(b), Uint512::from(10u32));
    }

    #[test]
    fn int512_abs_works() {
        let a = Int512::from(42i32);
        assert_eq!(a.abs(), a);

        let b = Int512::from(-42i32);
        assert_eq!(b.abs(), a);

        assert_eq!(Int512::zero().abs(), Int512::zero());
        assert_eq!((Int512::MIN + Int512::one()).abs(), Int512::MAX);
    }

    #[test]
    fn int512_unsigned_abs_works() {
        assert_eq!(Int512::zero().unsigned_abs(), Uint512::zero());
        assert_eq!(Int512::one().unsigned_abs(), Uint512::one());
        assert_eq!(
            Int512::MIN.unsigned_abs(),
            Uint512::from_be_bytes(Int512::MAX.to_be_bytes()) + Uint512::one()
        );

        let v = Int512::from(-42i32);
        assert_eq!(v.unsigned_abs(), v.abs_diff(Int512::zero()));
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int512_abs_min_panics() {
        _ = Int512::MIN.abs();
    }

    #[test]
    #[should_panic = "attempt to negate with overflow"]
    fn int512_neg_min_panics() {
        _ = -Int512::MIN;
    }

    #[test]
    fn int512_partial_eq() {
        let test_cases = [(1, 1, true), (42, 42, true), (42, 24, false), (0, 0, true)]
            .into_iter()
            .map(|(lhs, rhs, expected): (u64, u64, bool)| {
                (Int512::from(lhs), Int512::from(rhs), expected)
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
