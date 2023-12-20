use core::fmt;
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr,
    ShrAssign, Sub, SubAssign,
};
use core::str::FromStr;
use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};

use crate::errors::{
    ConversionOverflowError, DivideByZeroError, OverflowError, OverflowOperation, StdError,
};
use crate::{forward_ref_partial_eq, Int128, Int256, Int512, Int64, Uint128, Uint256, Uint64};

/// Used internally - we don't want to leak this type since we might change
/// the implementation in the future.
use bnum::types::U512;

use super::conversion::{forward_try_from, try_from_int_to_uint};
use super::num_consts::NumConsts;

/// An implementation of u512 that is using strings for JSON encoding/decoding,
/// such that the full u512 range can be used for clients that convert JSON numbers to floats,
/// like JavaScript and jq.
///
/// # Examples
///
/// Use `from` to create instances out of primitive uint types or `new` to provide big
/// endian bytes:
///
/// ```
/// # use cosmwasm_std::Uint512;
/// let a = Uint512::from(258u128);
/// let b = Uint512::new([
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
pub struct Uint512(#[schemars(with = "String")] pub(crate) U512);

forward_ref_partial_eq!(Uint512, Uint512);

impl Uint512 {
    pub const MAX: Uint512 = Uint512(U512::MAX);
    pub const MIN: Uint512 = Uint512(U512::ZERO);

    /// Creates a Uint512(value) from a big endian representation. It's just an alias for
    /// `from_be_bytes`.
    pub const fn new(value: [u8; 64]) -> Self {
        Self::from_be_bytes(value)
    }

    /// Creates a Uint512(0)
    #[inline]
    pub const fn zero() -> Self {
        Uint512(U512::ZERO)
    }

    /// Creates a Uint512(1)
    #[inline]
    pub const fn one() -> Self {
        Self(U512::ONE)
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
        Self(U512::from_digits(words))
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
        Self(U512::from_digits(words))
    }

    /// A conversion from `Uint256` that, unlike the one provided by the `From` trait,
    /// can be used in a `const` context.
    #[must_use]
    pub const fn from_uint256(num: Uint256) -> Self {
        let bytes = num.to_le_bytes();
        Self::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
            bytes[16], bytes[17], bytes[18], bytes[19], bytes[20], bytes[21], bytes[22], bytes[23],
            bytes[24], bytes[25], bytes[26], bytes[27], bytes[28], bytes[29], bytes[30], bytes[31],
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ])
    }

    /// Returns a copy of the number as big endian bytes.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn to_be_bytes(self) -> [u8; 64] {
        let words = self.0.digits();
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
        let words = self.0.digits();
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

    pub fn checked_div(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_div(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_div_euclid(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.checked_div(other)
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .ok_or_else(|| DivideByZeroError::new(self))
    }

    pub fn checked_shr(self, other: u32) -> Result<Self, OverflowError> {
        self.0
            .checked_shr(other)
            .map(Self)
            .ok_or_else(|| OverflowError::new(OverflowOperation::Shr, self, other))
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
    pub const fn abs_diff(self, other: Self) -> Self {
        Self(self.0.abs_diff(other.0))
    }
}

impl NumConsts for Uint512 {
    const ZERO: Self = Self::zero();
    const ONE: Self = Self::one();
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;
}

impl From<Uint256> for Uint512 {
    fn from(val: Uint256) -> Self {
        let mut bytes = [0u8; 64];
        bytes[32..].copy_from_slice(&val.to_be_bytes());

        Self::from_be_bytes(bytes)
    }
}

impl From<Uint128> for Uint512 {
    fn from(val: Uint128) -> Self {
        val.u128().into()
    }
}

impl From<Uint64> for Uint512 {
    fn from(val: Uint64) -> Self {
        val.u64().into()
    }
}

impl From<u128> for Uint512 {
    fn from(val: u128) -> Self {
        Uint512(val.into())
    }
}

impl From<u64> for Uint512 {
    fn from(val: u64) -> Self {
        Uint512(val.into())
    }
}

impl From<u32> for Uint512 {
    fn from(val: u32) -> Self {
        Uint512(val.into())
    }
}

impl From<u16> for Uint512 {
    fn from(val: u16) -> Self {
        Uint512(val.into())
    }
}

impl From<u8> for Uint512 {
    fn from(val: u8) -> Self {
        Uint512(val.into())
    }
}

impl TryFrom<Uint512> for Uint256 {
    type Error = ConversionOverflowError;

    fn try_from(value: Uint512) -> Result<Self, Self::Error> {
        let bytes = value.to_be_bytes();
        let (first_bytes, last_bytes) = bytes.split_at(32);

        if first_bytes != [0u8; 32] {
            return Err(ConversionOverflowError::new(
                "Uint512",
                "Uint256",
                value.to_string(),
            ));
        }

        Ok(Self::from_be_bytes(last_bytes.try_into().unwrap()))
    }
}

forward_try_from!(Uint512, Uint128);
forward_try_from!(Uint512, Uint64);

// Int to Uint
try_from_int_to_uint!(Int64, Uint512);
try_from_int_to_uint!(Int128, Uint512);
try_from_int_to_uint!(Int256, Uint512);
try_from_int_to_uint!(Int512, Uint512);

impl TryFrom<&str> for Uint512 {
    type Error = StdError;

    fn try_from(val: &str) -> Result<Self, Self::Error> {
        Self::from_str(val)
    }
}

impl FromStr for Uint512 {
    type Err = StdError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match U512::from_str_radix(s, 10) {
            Ok(u) => Ok(Self(u)),
            Err(e) => Err(StdError::generic_err(format!("Parsing u512: {e}"))),
        }
    }
}

impl From<Uint512> for String {
    fn from(original: Uint512) -> Self {
        original.to_string()
    }
}

impl fmt::Display for Uint512 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl Add<Uint512> for Uint512 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Uint512(self.0.checked_add(rhs.0).unwrap())
    }
}

impl<'a> Add<&'a Uint512> for Uint512 {
    type Output = Self;

    fn add(self, rhs: &'a Uint512) -> Self {
        Uint512(self.0.checked_add(rhs.0).unwrap())
    }
}

impl Sub<Uint512> for Uint512 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Uint512(self.0.checked_sub(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Sub, sub for Uint512, Uint512);

impl SubAssign<Uint512> for Uint512 {
    fn sub_assign(&mut self, rhs: Uint512) {
        self.0 = self.0.checked_sub(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Uint512, Uint512);

impl Div<Uint512> for Uint512 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}

impl<'a> Div<&'a Uint512> for Uint512 {
    type Output = Self;

    fn div(self, rhs: &'a Uint512) -> Self::Output {
        Self(self.0.checked_div(rhs.0).unwrap())
    }
}

impl Rem for Uint512 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero.
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Uint512, Uint512);

impl Not for Uint512 {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self(!self.0)
    }
}

impl RemAssign<Uint512> for Uint512 {
    fn rem_assign(&mut self, rhs: Uint512) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Uint512, Uint512);

impl Mul<Uint512> for Uint512 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.checked_mul(rhs.0).unwrap())
    }
}
forward_ref_binop!(impl Mul, mul for Uint512, Uint512);

impl MulAssign<Uint512> for Uint512 {
    fn mul_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_mul(rhs.0).unwrap();
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Uint512, Uint512);

impl Shr<u32> for Uint512 {
    type Output = Self;

    fn shr(self, rhs: u32) -> Self::Output {
        self.checked_shr(rhs).unwrap_or_else(|_| {
            panic!(
                "right shift error: {rhs} is larger or equal than the number of bits in Uint512",
            )
        })
    }
}

impl<'a> Shr<&'a u32> for Uint512 {
    type Output = Self;

    fn shr(self, rhs: &'a u32) -> Self::Output {
        Shr::<u32>::shr(self, *rhs)
    }
}

impl Shl<u32> for Uint512 {
    type Output = Self;

    fn shl(self, rhs: u32) -> Self::Output {
        self.checked_shl(rhs)
            .expect("attempt to shift left with overflow")
    }
}

impl<'a> Shl<&'a u32> for Uint512 {
    type Output = Self;

    fn shl(self, rhs: &'a u32) -> Self::Output {
        self.shl(*rhs)
    }
}

impl AddAssign<Uint512> for Uint512 {
    fn add_assign(&mut self, rhs: Uint512) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}

impl<'a> AddAssign<&'a Uint512> for Uint512 {
    fn add_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_add(rhs.0).unwrap();
    }
}

impl DivAssign<Uint512> for Uint512 {
    fn div_assign(&mut self, rhs: Self) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}

impl<'a> DivAssign<&'a Uint512> for Uint512 {
    fn div_assign(&mut self, rhs: &'a Uint512) {
        self.0 = self.0.checked_div(rhs.0).unwrap();
    }
}

impl ShrAssign<u32> for Uint512 {
    fn shr_assign(&mut self, rhs: u32) {
        *self = Shr::<u32>::shr(*self, rhs);
    }
}

impl<'a> ShrAssign<&'a u32> for Uint512 {
    fn shr_assign(&mut self, rhs: &'a u32) {
        *self = Shr::<u32>::shr(*self, *rhs);
    }
}

impl ShlAssign<u32> for Uint512 {
    fn shl_assign(&mut self, rhs: u32) {
        *self = self.shl(rhs);
    }
}

impl<'a> ShlAssign<&'a u32> for Uint512 {
    fn shl_assign(&mut self, rhs: &'a u32) {
        *self = self.shl(*rhs);
    }
}

impl Serialize for Uint512 {
    /// Serializes as an integer string using base 10
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for Uint512 {
    /// Deserialized from an integer string using base 10
    fn deserialize<D>(deserializer: D) -> Result<Uint512, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Uint512Visitor)
    }
}

struct Uint512Visitor;

impl<'de> de::Visitor<'de> for Uint512Visitor {
    type Value = Uint512;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded integer")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        Uint512::try_from(v).map_err(|e| E::custom(format!("invalid Uint512 '{v}' - {e}")))
    }
}

impl<A> core::iter::Sum<A> for Uint512
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
    use crate::{from_json, math::conversion::test_try_from_int_to_uint, to_json_vec};

    #[test]
    fn size_of_works() {
        assert_eq!(core::mem::size_of::<Uint512>(), 64);
    }

    #[test]
    fn uint512_new_works() {
        let num = Uint512::new([1; 64]);
        let a: [u8; 64] = num.to_be_bytes();
        assert_eq!(a, [1; 64]);

        let be_bytes = [
            0u8, 222u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 1u8, 2u8, 3u8,
        ];
        let num = Uint512::new(be_bytes);
        let resulting_bytes: [u8; 64] = num.to_be_bytes();
        assert_eq!(be_bytes, resulting_bytes);
    }

    #[test]
    fn uint512_not_works() {
        let num = Uint512::new([1; 64]);
        let a = (!num).to_be_bytes();
        assert_eq!(a, [254; 64]);

        assert_eq!(!Uint512::MAX, Uint512::MIN);
        assert_eq!(!Uint512::MIN, Uint512::MAX);
    }

    #[test]
    fn uint512_zero_works() {
        let zero = Uint512::zero();
        assert_eq!(
            zero.to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn uin512_one_works() {
        let one = Uint512::one();
        assert_eq!(
            one.to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 1
            ]
        );
    }

    #[test]
    fn uint512_endianness() {
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
        let num1 = Uint512::new(be_bytes);
        let num2 = Uint512::from_be_bytes(be_bytes);
        let num3 = Uint512::from_le_bytes(le_bytes);
        assert_eq!(num1, Uint512::from(65536u32 + 512 + 3));
        assert_eq!(num1, num2);
        assert_eq!(num1, num3);
    }

    #[test]
    fn uint512_convert_from() {
        let a = Uint512::from(5u128);
        assert_eq!(a.0, U512::from(5u32));

        let a = Uint512::from(5u64);
        assert_eq!(a.0, U512::from(5u32));

        let a = Uint512::from(5u32);
        assert_eq!(a.0, U512::from(5u32));

        let a = Uint512::from(5u16);
        assert_eq!(a.0, U512::from(5u32));

        let a = Uint512::from(5u8);
        assert_eq!(a.0, U512::from(5u32));

        let result = Uint512::try_from("34567");
        assert_eq!(
            result.unwrap().0,
            U512::from_str_radix("34567", 10).unwrap()
        );

        let result = Uint512::try_from("1.23");
        assert!(result.is_err());
    }

    #[test]
    fn uint512_try_from_signed_works() {
        test_try_from_int_to_uint::<Int64, Uint512>("Int64", "Uint512");
        test_try_from_int_to_uint::<Int128, Uint512>("Int128", "Uint512");
        test_try_from_int_to_uint::<Int256, Uint512>("Int256", "Uint512");
        test_try_from_int_to_uint::<Int512, Uint512>("Int512", "Uint512");
    }

    #[test]
    fn uint512_try_into() {
        assert!(Uint64::try_from(Uint512::MAX).is_err());
        assert!(Uint128::try_from(Uint512::MAX).is_err());
        assert!(Uint256::try_from(Uint512::MAX).is_err());

        assert_eq!(Uint64::try_from(Uint512::zero()), Ok(Uint64::zero()));
        assert_eq!(Uint128::try_from(Uint512::zero()), Ok(Uint128::zero()));
        assert_eq!(Uint256::try_from(Uint512::zero()), Ok(Uint256::zero()));

        assert_eq!(
            Uint64::try_from(Uint512::from(42u64)),
            Ok(Uint64::from(42u64))
        );
        assert_eq!(
            Uint128::try_from(Uint512::from(42u128)),
            Ok(Uint128::from(42u128))
        );
        assert_eq!(
            Uint256::try_from(Uint512::from(42u128)),
            Ok(Uint256::from(42u128))
        );
    }

    #[test]
    fn uint512_convert_to_uint128() {
        let source = Uint512::from(42u128);
        let target = Uint128::try_from(source);
        assert_eq!(target, Ok(Uint128::new(42u128)));

        let source = Uint512::MAX;
        let target = Uint128::try_from(source);
        assert_eq!(
            target,
            Err(ConversionOverflowError::new(
                "Uint512",
                "Uint128",
                Uint512::MAX.to_string()
            ))
        );
    }

    #[test]
    fn uint512_from_uint256() {
        assert_eq!(
            Uint512::from_uint256(Uint256::from_str("123").unwrap()),
            Uint512::from_str("123").unwrap()
        );

        assert_eq!(
            Uint512::from_uint256(Uint256::from_str("9785746283745").unwrap()),
            Uint512::from_str("9785746283745").unwrap()
        );

        assert_eq!(
            Uint512::from_uint256(
                Uint256::from_str(
                    "97857462837575757832978493758398593853985452378423874623874628736482736487236"
                )
                .unwrap()
            ),
            Uint512::from_str(
                "97857462837575757832978493758398593853985452378423874623874628736482736487236"
            )
            .unwrap()
        );
    }

    #[test]
    fn uint512_implements_display() {
        let a = Uint512::from(12345u32);
        assert_eq!(format!("Embedded: {a}"), "Embedded: 12345");
        assert_eq!(a.to_string(), "12345");

        let a = Uint512::zero();
        assert_eq!(format!("Embedded: {a}"), "Embedded: 0");
        assert_eq!(a.to_string(), "0");
    }

    #[test]
    fn uint512_display_padding_works() {
        // width > natural representation
        let a = Uint512::from(123u64);
        assert_eq!(format!("Embedded: {a:05}"), "Embedded: 00123");

        // width < natural representation
        let a = Uint512::from(123u64);
        assert_eq!(format!("Embedded: {a:02}"), "Embedded: 123");
    }

    #[test]
    fn uint512_to_be_bytes_works() {
        assert_eq!(
            Uint512::zero().to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0,
            ]
        );
        assert_eq!(
            Uint512::MAX.to_be_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            ]
        );
        assert_eq!(
            Uint512::from(1u128).to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 1
            ]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(64, "big")]`
        assert_eq!(
            Uint512::from(240282366920938463463374607431768124608u128).to_be_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 180, 196, 179, 87, 165,
                121, 59, 133, 246, 117, 221, 191, 255, 254, 172, 192
            ]
        );
        assert_eq!(
            Uint512::from_be_bytes([
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
    fn uint512_to_le_bytes_works() {
        assert_eq!(
            Uint512::zero().to_le_bytes(),
            [
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Uint512::MAX.to_le_bytes(),
            [
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
                0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff
            ]
        );
        assert_eq!(
            Uint512::from(1u128).to_le_bytes(),
            [
                1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        // Python: `[b for b in (240282366920938463463374607431768124608).to_bytes(64, "little")]`
        assert_eq!(
            Uint512::from(240282366920938463463374607431768124608u128).to_le_bytes(),
            [
                192, 172, 254, 255, 191, 221, 117, 246, 133, 59, 121, 165, 87, 179, 196, 180, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
            ]
        );
        assert_eq!(
            Uint512::from_be_bytes([
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
    fn uint512_is_zero_works() {
        assert!(Uint512::zero().is_zero());
        assert!(Uint512(U512::from(0u32)).is_zero());

        assert!(!Uint512::from(1u32).is_zero());
        assert!(!Uint512::from(123u32).is_zero());
    }

    #[test]
    fn uint512_wrapping_methods() {
        // wrapping_add
        assert_eq!(
            Uint512::from(2u32).wrapping_add(Uint512::from(2u32)),
            Uint512::from(4u32)
        ); // non-wrapping
        assert_eq!(
            Uint512::MAX.wrapping_add(Uint512::from(1u32)),
            Uint512::from(0u32)
        ); // wrapping

        // wrapping_sub
        assert_eq!(
            Uint512::from(7u32).wrapping_sub(Uint512::from(5u32)),
            Uint512::from(2u32)
        ); // non-wrapping
        assert_eq!(
            Uint512::from(0u32).wrapping_sub(Uint512::from(1u32)),
            Uint512::MAX
        ); // wrapping

        // wrapping_mul
        assert_eq!(
            Uint512::from(3u32).wrapping_mul(Uint512::from(2u32)),
            Uint512::from(6u32)
        ); // non-wrapping
        assert_eq!(
            Uint512::MAX.wrapping_mul(Uint512::from(2u32)),
            Uint512::MAX - Uint512::one()
        ); // wrapping

        // wrapping_pow
        assert_eq!(Uint512::from(2u32).wrapping_pow(3), Uint512::from(8u32)); // non-wrapping
        assert_eq!(Uint512::MAX.wrapping_pow(2), Uint512::from(1u32)); // wrapping
    }

    #[test]
    fn uint512_json() {
        let orig = Uint512::from(1234567890987654321u128);
        let serialized = to_json_vec(&orig).unwrap();
        assert_eq!(serialized.as_slice(), b"\"1234567890987654321\"");
        let parsed: Uint512 = from_json(serialized).unwrap();
        assert_eq!(parsed, orig);
    }

    #[test]
    fn uint512_compare() {
        let a = Uint512::from(12345u32);
        let b = Uint512::from(23456u32);

        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, Uint512::from(12345u32));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_math() {
        let a = Uint512::from(12345u32);
        let b = Uint512::from(23456u32);

        // test + with owned and reference right hand side
        assert_eq!(a + b, Uint512::from(35801u32));
        assert_eq!(a + &b, Uint512::from(35801u32));

        // test - with owned and reference right hand side
        assert_eq!(b - a, Uint512::from(11111u32));
        assert_eq!(b - &a, Uint512::from(11111u32));

        // test += with owned and reference right hand side
        let mut c = Uint512::from(300000u32);
        c += b;
        assert_eq!(c, Uint512::from(323456u32));
        let mut d = Uint512::from(300000u32);
        d += &b;
        assert_eq!(d, Uint512::from(323456u32));

        // test -= with owned and reference right hand side
        let mut c = Uint512::from(300000u32);
        c -= b;
        assert_eq!(c, Uint512::from(276544u32));
        let mut d = Uint512::from(300000u32);
        d -= &b;
        assert_eq!(d, Uint512::from(276544u32));

        // error result on underflow (- would produce negative result)
        let underflow_result = a.checked_sub(b);
        let OverflowError {
            operand1, operand2, ..
        } = underflow_result.unwrap_err();
        assert_eq!((operand1, operand2), (a.to_string(), b.to_string()));
    }

    #[test]
    #[should_panic]
    fn uint512_add_overflow_panics() {
        let max = Uint512::new([255u8; 64]);
        let _ = max + Uint512::from(12u32);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_sub_works() {
        assert_eq!(
            Uint512::from(2u32) - Uint512::from(1u32),
            Uint512::from(1u32)
        );
        assert_eq!(
            Uint512::from(2u32) - Uint512::from(0u32),
            Uint512::from(2u32)
        );
        assert_eq!(
            Uint512::from(2u32) - Uint512::from(2u32),
            Uint512::from(0u32)
        );

        // works for refs
        let a = Uint512::from(10u32);
        let b = Uint512::from(3u32);
        let expected = Uint512::from(7u32);
        assert_eq!(a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn uint512_sub_overflow_panics() {
        let _ = Uint512::from(1u32) - Uint512::from(2u32);
    }

    #[test]
    fn uint512_sub_assign_works() {
        let mut a = Uint512::from(14u32);
        a -= Uint512::from(2u32);
        assert_eq!(a, Uint512::from(12u32));

        // works for refs
        let mut a = Uint512::from(10u32);
        let b = Uint512::from(3u32);
        let expected = Uint512::from(7u32);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_mul_works() {
        assert_eq!(
            Uint512::from(2u32) * Uint512::from(3u32),
            Uint512::from(6u32)
        );
        assert_eq!(Uint512::from(2u32) * Uint512::zero(), Uint512::zero());

        // works for refs
        let a = Uint512::from(11u32);
        let b = Uint512::from(3u32);
        let expected = Uint512::from(33u32);
        assert_eq!(a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn uint512_mul_assign_works() {
        let mut a = Uint512::from(14u32);
        a *= Uint512::from(2u32);
        assert_eq!(a, Uint512::from(28u32));

        // works for refs
        let mut a = Uint512::from(10u32);
        let b = Uint512::from(3u32);
        a *= &b;
        assert_eq!(a, Uint512::from(30u32));
    }

    #[test]
    fn uint512_pow_works() {
        assert_eq!(Uint512::from(2u32).pow(2), Uint512::from(4u32));
        assert_eq!(Uint512::from(2u32).pow(10), Uint512::from(1024u32));
    }

    #[test]
    #[should_panic]
    fn uint512_pow_overflow_panics() {
        _ = Uint512::MAX.pow(2u32);
    }

    #[test]
    fn uint512_shr_works() {
        let original = Uint512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 2u8, 0u8, 4u8, 2u8,
        ]);

        let shifted = Uint512::new([
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 128u8, 1u8, 0u8,
        ]);

        assert_eq!(original >> 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint512_shr_overflow_panics() {
        let _ = Uint512::from(1u32) >> 512u32;
    }

    #[test]
    fn uint512_shl_works() {
        let original = Uint512::new([
            64u8, 128u8, 1u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]);

        let shifted = Uint512::new([
            2u8, 0u8, 4u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
            0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8, 0u8,
        ]);

        assert_eq!(original << 2u32, shifted);
    }

    #[test]
    #[should_panic]
    fn uint512_shl_overflow_panics() {
        let _ = Uint512::from(1u32) << 512u32;
    }

    #[test]
    fn sum_works() {
        let nums = vec![
            Uint512::from(17u32),
            Uint512::from(123u32),
            Uint512::from(540u32),
            Uint512::from(82u32),
        ];
        let expected = Uint512::from(762u32);

        let sum_as_ref: Uint512 = nums.iter().sum();
        assert_eq!(expected, sum_as_ref);

        let sum_as_owned: Uint512 = nums.into_iter().sum();
        assert_eq!(expected, sum_as_owned);
    }

    #[test]
    fn uint512_methods() {
        // checked_*
        assert!(matches!(
            Uint512::MAX.checked_add(Uint512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Uint512::from(1u32).checked_add(Uint512::from(1u32)),
            Ok(Uint512::from(2u32)),
        );
        assert!(matches!(
            Uint512::from(0u32).checked_sub(Uint512::from(1u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Uint512::from(2u32).checked_sub(Uint512::from(1u32)),
            Ok(Uint512::from(1u32)),
        );
        assert!(matches!(
            Uint512::MAX.checked_mul(Uint512::from(2u32)),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Uint512::from(2u32).checked_mul(Uint512::from(2u32)),
            Ok(Uint512::from(4u32)),
        );
        assert!(matches!(
            Uint512::MAX.checked_pow(2u32),
            Err(OverflowError { .. })
        ));
        assert_eq!(
            Uint512::from(2u32).checked_pow(3u32),
            Ok(Uint512::from(8u32)),
        );
        assert!(matches!(
            Uint512::MAX.checked_div(Uint512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        assert_eq!(
            Uint512::from(6u32).checked_div(Uint512::from(2u32)),
            Ok(Uint512::from(3u32)),
        );
        assert!(matches!(
            Uint512::MAX.checked_div_euclid(Uint512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));
        assert_eq!(
            Uint512::from(6u32).checked_div_euclid(Uint512::from(2u32)),
            Ok(Uint512::from(3u32)),
        );
        assert_eq!(
            Uint512::from(7u32).checked_div_euclid(Uint512::from(2u32)),
            Ok(Uint512::from(3u32)),
        );
        assert!(matches!(
            Uint512::MAX.checked_rem(Uint512::from(0u32)),
            Err(DivideByZeroError { .. })
        ));

        // saturating_*
        assert_eq!(
            Uint512::MAX.saturating_add(Uint512::from(1u32)),
            Uint512::MAX
        );
        assert_eq!(
            Uint512::from(0u32).saturating_sub(Uint512::from(1u32)),
            Uint512::from(0u32)
        );
        assert_eq!(
            Uint512::MAX.saturating_mul(Uint512::from(2u32)),
            Uint512::MAX
        );
        assert_eq!(
            Uint512::from(4u32).saturating_pow(2u32),
            Uint512::from(16u32)
        );
        assert_eq!(Uint512::MAX.saturating_pow(2u32), Uint512::MAX);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_implements_rem() {
        let a = Uint512::from(10u32);
        assert_eq!(a % Uint512::from(10u32), Uint512::zero());
        assert_eq!(a % Uint512::from(2u32), Uint512::zero());
        assert_eq!(a % Uint512::from(1u32), Uint512::zero());
        assert_eq!(a % Uint512::from(3u32), Uint512::from(1u32));
        assert_eq!(a % Uint512::from(4u32), Uint512::from(2u32));

        // works for refs
        let a = Uint512::from(10u32);
        let b = Uint512::from(3u32);
        let expected = Uint512::from(1u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn uint512_rem_panics_for_zero() {
        let _ = Uint512::from(10u32) % Uint512::zero();
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn uint512_rem_works() {
        assert_eq!(
            Uint512::from(12u32) % Uint512::from(10u32),
            Uint512::from(2u32)
        );
        assert_eq!(Uint512::from(50u32) % Uint512::from(5u32), Uint512::zero());

        // works for refs
        let a = Uint512::from(42u32);
        let b = Uint512::from(5u32);
        let expected = Uint512::from(2u32);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn uint512_rem_assign_works() {
        let mut a = Uint512::from(30u32);
        a %= Uint512::from(4u32);
        assert_eq!(a, Uint512::from(2u32));

        // works for refs
        let mut a = Uint512::from(25u32);
        let b = Uint512::from(6u32);
        a %= &b;
        assert_eq!(a, Uint512::from(1u32));
    }

    #[test]
    fn uint512_abs_diff_works() {
        let a = Uint512::from(42u32);
        let b = Uint512::from(5u32);
        let expected = Uint512::from(37u32);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    fn uint512_partial_eq() {
        let test_cases = [(1, 1, true), (42, 42, true), (42, 24, false), (0, 0, true)]
            .into_iter()
            .map(|(lhs, rhs, expected): (u64, u64, bool)| {
                (Uint512::from(lhs), Uint512::from(rhs), expected)
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
