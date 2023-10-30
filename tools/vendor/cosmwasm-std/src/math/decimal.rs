use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use std::cmp::Ordering;
use std::fmt::{self, Write};
use std::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use std::str::FromStr;
use thiserror::Error;

use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyRatioError, DivideByZeroError, OverflowError,
    OverflowOperation, RoundUpOverflowError, StdError,
};
use crate::forward_ref_partial_eq;

use super::Fraction;
use super::Isqrt;
use super::{Uint128, Uint256};

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal(#[schemars(with = "String")] Uint128);

forward_ref_partial_eq!(Decimal, Decimal);

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Decimal range exceeded")]
pub struct DecimalRangeExceeded;

impl Decimal {
    const DECIMAL_FRACTIONAL: Uint128 = Uint128::new(1_000_000_000_000_000_000u128); // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: Uint128 =
        Uint128::new(1_000_000_000_000_000_000_000_000_000_000_000_000u128); // (1*10**18)**2 = 1*10**36

    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18; // This needs to be an even number.
    /// The largest value that can be represented by this decimal type.
    pub const MAX: Self = Self(Uint128::MAX);
    /// The smallest value that can be represented by this decimal type.
    pub const MIN: Self = Self(Uint128::MIN);

    /// Creates a Decimal(value)
    /// This is equivalent to `Decimal::from_atomics(value, 18)` but usable in a const context.
    pub const fn new(value: Uint128) -> Self {
        Self(value)
    }

    /// Creates a Decimal(Uint128(value))
    /// This is equivalent to `Decimal::from_atomics(value, 18)` but usable in a const context.
    pub const fn raw(value: u128) -> Self {
        Self(Uint128::new(value))
    }

    /// Create a 1.0 Decimal
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal
    #[inline]
    pub const fn zero() -> Self {
        Self(Uint128::zero())
    }

    /// Convert x% into Decimal
    pub fn percent(x: u64) -> Self {
        Self(((x as u128) * 10_000_000_000_000_000).into())
    }

    /// Convert permille (x/1000) into Decimal
    pub fn permille(x: u64) -> Self {
        Self(((x as u128) * 1_000_000_000_000_000).into())
    }

    /// Convert basis points (x/10000) into Decimal
    pub fn bps(x: u64) -> Self {
        Self(((x as u128) * 100_000_000_000_000).into())
    }

    /// Creates a decimal from a number of atomic units and the number
    /// of decimal places. The inputs will be converted internally to form
    /// a decimal with 18 decimal places. So the input 123 and 2 will create
    /// the decimal 1.23.
    ///
    /// Using 18 decimal places is slightly more efficient than other values
    /// as no internal conversion is necessary.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Decimal, Uint128};
    /// let a = Decimal::from_atomics(Uint128::new(1234), 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = Decimal::from_atomics(1234u128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = Decimal::from_atomics(1u64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint128>,
        decimal_places: u32,
    ) -> Result<Self, DecimalRangeExceeded> {
        let atomics = atomics.into();
        const TEN: Uint128 = Uint128::new(10);
        Ok(match decimal_places.cmp(&(Self::DECIMAL_PLACES)) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = TEN.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| DecimalRangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = TEN.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Uint128 range.
                    // Any Uint128 `x` divided by `factor` with `factor > Uint128::MAX` is 0.
                    // Try e.g. Python3: `(2**128-1) // 2**128`
                    Self(Uint128::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn from_ratio(numerator: impl Into<Uint128>, denominator: impl Into<Uint128>) -> Self {
        match Decimal::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal
    pub fn checked_from_ratio(
        numerator: impl Into<Uint128>,
        denominator: impl Into<Uint128>,
    ) -> Result<Self, CheckedFromRatioError> {
        let numerator: Uint128 = numerator.into();
        let denominator: Uint128 = denominator.into();
        match numerator.checked_multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator) {
            Ok(ratio) => {
                // numerator * DECIMAL_FRACTIONAL / denominator
                Ok(Decimal(ratio))
            }
            Err(CheckedMultiplyRatioError::Overflow) => Err(CheckedFromRatioError::Overflow),
            Err(CheckedMultiplyRatioError::DivideByZero) => {
                Err(CheckedFromRatioError::DivideByZero)
            }
        }
    }

    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{Decimal, Uint128};
    /// # use std::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = Decimal::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Uint128::new(1234000000000000000));
    ///
    /// // Smallest possible value
    /// let b = Decimal::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Uint128::new(1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> Uint128 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal::atomics()`].
    #[must_use]
    #[inline]
    pub const fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES
    }

    /// Rounds value down after decimal places.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn floor(&self) -> Self {
        Self((self.0 / Self::DECIMAL_FRACTIONAL) * Self::DECIMAL_FRACTIONAL)
    }

    /// Rounds value up after decimal places. Panics on overflow.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn ceil(&self) -> Self {
        match self.checked_ceil() {
            Ok(value) => value,
            Err(_) => panic!("attempt to ceil with overflow"),
        }
    }

    /// Rounds value up after decimal places. Returns OverflowError on overflow.
    pub fn checked_ceil(&self) -> Result<Self, RoundUpOverflowError> {
        let floor = self.floor();
        if floor == self {
            Ok(floor)
        } else {
            floor
                .checked_add(Decimal::one())
                .map_err(|_| RoundUpOverflowError)
        }
    }

    pub fn checked_add(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Add, self, other))
    }

    pub fn checked_sub(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Sub, self, other))
    }

    /// Multiplies one `Decimal` by another, returning an `OverflowError` if an overflow occurred.
    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        let result_as_uint256 = self.numerator().full_mul(other.numerator())
            / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        result_as_uint256
            .try_into()
            .map(Self)
            .map_err(|_| OverflowError {
                operation: crate::OverflowOperation::Mul,
                operand1: self.to_string(),
                operand2: other.to_string(),
            })
    }

    /// Raises a value to the power of `exp`, panics if an overflow occurred.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn pow(self, exp: u32) -> Self {
        match self.checked_pow(exp) {
            Ok(value) => value,
            Err(_) => panic!("Multiplication overflow"),
        }
    }

    /// Raises a value to the power of `exp`, returning an `OverflowError` if an overflow occurred.
    pub fn checked_pow(self, exp: u32) -> Result<Self, OverflowError> {
        // This uses the exponentiation by squaring algorithm:
        // https://en.wikipedia.org/wiki/Exponentiation_by_squaring#Basic_method

        fn inner(mut x: Decimal, mut n: u32) -> Result<Decimal, OverflowError> {
            if n == 0 {
                return Ok(Decimal::one());
            }

            let mut y = Decimal::one();

            while n > 1 {
                if n % 2 == 0 {
                    x = x.checked_mul(x)?;
                    n /= 2;
                } else {
                    y = x.checked_mul(y)?;
                    x = x.checked_mul(x)?;
                    n = (n - 1) / 2;
                }
            }

            Ok(x * y)
        }

        inner(self, exp).map_err(|_| OverflowError {
            operation: crate::OverflowOperation::Pow,
            operand1: self.to_string(),
            operand2: exp.to_string(),
        })
    }

    pub fn checked_div(self, other: Self) -> Result<Self, CheckedFromRatioError> {
        Decimal::checked_from_ratio(self.numerator(), other.numerator())
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .map_err(|_| DivideByZeroError::new(self))
    }

    /// Returns the approximate square root as a Decimal.
    ///
    /// This should not overflow or panic.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn sqrt(&self) -> Self {
        // Algorithm described in https://hackmd.io/@webmaster128/SJThlukj_
        // We start with the highest precision possible and lower it until
        // there's no overflow.
        //
        // TODO: This could be made more efficient once log10 is in:
        // https://github.com/rust-lang/rust/issues/70887
        // The max precision is something like `9 - log10(self.0) / 2`.
        (0..=Self::DECIMAL_PLACES / 2)
            .rev()
            .find_map(|i| self.sqrt_with_precision(i))
            // The last step (i = 0) is guaranteed to succeed because `isqrt(u128::MAX) * 10^9` does not overflow
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 9 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn sqrt_with_precision(&self, precision: u32) -> Option<Self> {
        let inner_mul = 100u128.pow(precision);
        self.0.checked_mul(inner_mul.into()).ok().map(|inner| {
            let outer_mul = 10u128.pow(Self::DECIMAL_PLACES / 2 - precision);
            Decimal(inner.isqrt().checked_mul(Uint128::from(outer_mul)).unwrap())
        })
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs_diff(self, other: Self) -> Self {
        Self(self.0.abs_diff(other.0))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_add(self, other: Self) -> Self {
        match self.checked_add(other) {
            Ok(value) => value,
            Err(_) => Self::MAX,
        }
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_sub(self, other: Self) -> Self {
        match self.checked_sub(other) {
            Ok(value) => value,
            Err(_) => Self::zero(),
        }
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_mul(self, other: Self) -> Self {
        match self.checked_mul(other) {
            Ok(value) => value,
            Err(_) => Self::MAX,
        }
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_pow(self, exp: u32) -> Self {
        match self.checked_pow(exp) {
            Ok(value) => value,
            Err(_) => Self::MAX,
        }
    }

    /// Converts this decimal to an unsigned integer by truncating
    /// the fractional part, e.g. 22.5 becomes 22.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use cosmwasm_std::{Decimal, Uint128};
    ///
    /// let d = Decimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(12));
    ///
    /// let d = Decimal::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(12));
    ///
    /// let d = Decimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint128::new(75));
    /// ```
    #[must_use]
    pub fn to_uint_floor(self) -> Uint128 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Converts this decimal to an unsigned integer by rounting up
    /// to the next integer, e.g. 22.3 becomes 23.
    ///
    /// ## Examples
    ///
    /// ```
    /// use std::str::FromStr;
    /// use cosmwasm_std::{Decimal, Uint128};
    ///
    /// let d = Decimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    ///
    /// let d = Decimal::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(13));
    ///
    /// let d = Decimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint128::new(75));
    /// ```
    #[must_use]
    pub fn to_uint_ceil(self) -> Uint128 {
        // Using `q = 1 + ((x - 1) / y); // if x != 0` with unsigned integers x, y, q
        // from https://stackoverflow.com/a/2745086/2013738. We know `x + y` CAN overflow.
        let x = self.0;
        let y = Self::DECIMAL_FRACTIONAL;
        if x.is_zero() {
            Uint128::zero()
        } else {
            Uint128::one() + ((x - Uint128::one()) / y)
        }
    }
}

impl Fraction<Uint128> for Decimal {
    #[inline]
    fn numerator(&self) -> Uint128 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint128 {
        Self::DECIMAL_FRACTIONAL
    }

    /// Returns the multiplicative inverse `1/d` for decimal `d`.
    ///
    /// If `d` is zero, none is returned.
    fn inv(&self) -> Option<Self> {
        if self.is_zero() {
            None
        } else {
            // Let self be p/q with p = self.0 and q = DECIMAL_FRACTIONAL.
            // Now we calculate the inverse a/b = q/p such that b = DECIMAL_FRACTIONAL. Then
            // `a = DECIMAL_FRACTIONAL*DECIMAL_FRACTIONAL / self.0`.
            Some(Decimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl FromStr for Decimal {
    type Err = StdError;

    /// Converts the decimal string to a Decimal
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<Uint128>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<Uint128>()
                .map_err(|_| StdError::generic_err("Error parsing fractional"))?;
            let exp = (Self::DECIMAL_PLACES.checked_sub(fractional_part.len() as u32)).ok_or_else(
                || {
                    StdError::generic_err(format!(
                        "Cannot parse more than {} fractional digits",
                        Self::DECIMAL_PLACES
                    ))
                },
            )?;
            debug_assert!(exp <= Self::DECIMAL_PLACES);
            let fractional_factor = Uint128::from(10u128.pow(exp));
            atomics = atomics
                .checked_add(
                    // The inner multiplication can't overflow because
                    // fractional < 10^DECIMAL_PLACES && fractional_factor <= 10^DECIMAL_PLACES
                    fractional.checked_mul(fractional_factor).unwrap(),
                )
                .map_err(|_| StdError::generic_err("Value too big"))?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::generic_err("Unexpected number of dots"));
        }

        Ok(Decimal(atomics))
    }
}

impl fmt::Display for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::DECIMAL_FRACTIONAL;
        let fractional = (self.0).checked_rem(Self::DECIMAL_FRACTIONAL).unwrap();

        if fractional.is_zero() {
            write!(f, "{}", whole)
        } else {
            let fractional_string = format!(
                "{:0>padding$}",
                fractional,
                padding = Self::DECIMAL_PLACES as usize
            );
            f.write_str(&whole.to_string())?;
            f.write_char('.')?;
            f.write_str(fractional_string.trim_end_matches('0'))?;
            Ok(())
        }
    }
}

impl fmt::Debug for Decimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decimal({})", self)
    }
}

impl Add for Decimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Decimal(self.0 + other.0)
    }
}
forward_ref_binop!(impl Add, add for Decimal, Decimal);

impl AddAssign for Decimal {
    fn add_assign(&mut self, rhs: Decimal) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Decimal, Decimal);

impl Sub for Decimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Decimal(self.0 - other.0)
    }
}
forward_ref_binop!(impl Sub, sub for Decimal, Decimal);

impl SubAssign for Decimal {
    fn sub_assign(&mut self, rhs: Decimal) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Decimal, Decimal);

impl Mul for Decimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_uint256 = self.numerator().full_mul(other.numerator())
            / Uint256::from_uint128(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        match result_as_uint256.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}
forward_ref_binop!(impl Mul, mul for Decimal, Decimal);

impl MulAssign for Decimal {
    fn mul_assign(&mut self, rhs: Decimal) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Decimal, Decimal);

/// Both d*u and u*d with d: Decimal and u: Uint128 returns an Uint128. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal result for the same calculation, use Decimal(d*u) or Decimal(u*d).
impl Mul<Decimal> for Uint128 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint128::zero();
        }
        self.multiply_ratio(rhs.0, Decimal::DECIMAL_FRACTIONAL)
    }
}

impl Mul<Uint128> for Decimal {
    type Output = Uint128;

    fn mul(self, rhs: Uint128) -> Self::Output {
        rhs * self
    }
}

impl Div for Decimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match Decimal::checked_from_ratio(self.numerator(), other.numerator()) {
            Ok(ratio) => ratio,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Division failed - denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => {
                panic!("Division failed - multiplication overflow")
            }
        }
    }
}
forward_ref_binop!(impl Div, div for Decimal, Decimal);

impl DivAssign for Decimal {
    fn div_assign(&mut self, rhs: Decimal) {
        *self = *self / rhs;
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Decimal, Decimal);

impl Div<Uint128> for Decimal {
    type Output = Self;

    fn div(self, rhs: Uint128) -> Self::Output {
        Decimal(self.0 / rhs)
    }
}

impl DivAssign<Uint128> for Decimal {
    fn div_assign(&mut self, rhs: Uint128) {
        self.0 /= rhs;
    }
}

impl Rem for Decimal {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Decimal, Decimal);

impl RemAssign<Decimal> for Decimal {
    fn rem_assign(&mut self, rhs: Decimal) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Decimal, Decimal);

impl<A> std::iter::Sum<A> for Decimal
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DecimalVisitor)
    }
}

struct DecimalVisitor;

impl<'de> de::Visitor<'de> for DecimalVisitor {
    type Value = Decimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Decimal::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{}': {}", v, e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_slice, to_vec};

    fn dec(input: &str) -> Decimal {
        Decimal::from_str(input).unwrap()
    }

    #[test]
    fn decimal_new() {
        let expected = Uint128::from(300u128);
        assert_eq!(Decimal::new(expected).0, expected);
    }

    #[test]
    fn decimal_raw() {
        let value = 300u128;
        assert_eq!(Decimal::raw(value).0.u128(), value);
    }

    #[test]
    fn decimal_one() {
        let value = Decimal::one();
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal_zero() {
        let value = Decimal::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn decimal_percent() {
        let value = Decimal::percent(50);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(2u8));
    }

    #[test]
    fn decimal_permille() {
        let value = Decimal::permille(125);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(8u8));
    }

    #[test]
    fn decimal_bps() {
        let value = Decimal::bps(125);
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(80u8));
    }

    #[test]
    fn decimal_from_atomics_works() {
        let one = Decimal::one();
        let two = one + one;

        assert_eq!(Decimal::from_atomics(1u128, 0).unwrap(), one);
        assert_eq!(Decimal::from_atomics(10u128, 1).unwrap(), one);
        assert_eq!(Decimal::from_atomics(100u128, 2).unwrap(), one);
        assert_eq!(Decimal::from_atomics(1000u128, 3).unwrap(), one);
        assert_eq!(
            Decimal::from_atomics(1000000000000000000u128, 18).unwrap(),
            one
        );
        assert_eq!(
            Decimal::from_atomics(10000000000000000000u128, 19).unwrap(),
            one
        );
        assert_eq!(
            Decimal::from_atomics(100000000000000000000u128, 20).unwrap(),
            one
        );

        assert_eq!(Decimal::from_atomics(2u128, 0).unwrap(), two);
        assert_eq!(Decimal::from_atomics(20u128, 1).unwrap(), two);
        assert_eq!(Decimal::from_atomics(200u128, 2).unwrap(), two);
        assert_eq!(Decimal::from_atomics(2000u128, 3).unwrap(), two);
        assert_eq!(
            Decimal::from_atomics(2000000000000000000u128, 18).unwrap(),
            two
        );
        assert_eq!(
            Decimal::from_atomics(20000000000000000000u128, 19).unwrap(),
            two
        );
        assert_eq!(
            Decimal::from_atomics(200000000000000000000u128, 20).unwrap(),
            two
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal::from_atomics(4321u128, 20).unwrap(),
            Decimal::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(6789u128, 20).unwrap(),
            Decimal::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 38).unwrap(),
            Decimal::from_str("3.402823669209384634").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 39).unwrap(),
            Decimal::from_str("0.340282366920938463").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 45).unwrap(),
            Decimal::from_str("0.000000340282366920").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 51).unwrap(),
            Decimal::from_str("0.000000000000340282").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 56).unwrap(),
            Decimal::from_str("0.000000000000000003").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, 57).unwrap(),
            Decimal::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            Decimal::from_atomics(u128::MAX, u32::MAX).unwrap(),
            Decimal::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = Decimal::MAX;
        assert_eq!(
            Decimal::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Overflow is only possible with digits < 18
        let result = Decimal::from_atomics(u128::MAX, 17);
        assert_eq!(result.unwrap_err(), DecimalRangeExceeded);
    }

    #[test]
    fn decimal_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal::from_ratio(1u128, 1u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(53u128, 53u128), Decimal::one());
        assert_eq!(Decimal::from_ratio(125u128, 125u128), Decimal::one());

        // 1.5
        assert_eq!(Decimal::from_ratio(3u128, 2u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(150u128, 100u128), Decimal::percent(150));
        assert_eq!(Decimal::from_ratio(333u128, 222u128), Decimal::percent(150));

        // 0.125
        assert_eq!(Decimal::from_ratio(1u64, 8u64), Decimal::permille(125));
        assert_eq!(Decimal::from_ratio(125u64, 1000u64), Decimal::permille(125));

        // 1/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(1u64, 3u64),
            Decimal(Uint128::from(333_333_333_333_333_333u128))
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal::from_ratio(2u64, 3u64),
            Decimal(Uint128::from(666_666_666_666_666_666u128))
        );

        // large inputs
        assert_eq!(Decimal::from_ratio(0u128, u128::MAX), Decimal::zero());
        assert_eq!(Decimal::from_ratio(u128::MAX, u128::MAX), Decimal::one());
        // 340282366920938463463 is the largest integer <= Decimal::MAX
        assert_eq!(
            Decimal::from_ratio(340282366920938463463u128, 1u128),
            Decimal::from_str("340282366920938463463").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal_from_ratio_panics_for_zero_denominator() {
        Decimal::from_ratio(1u128, 0u128);
    }

    #[test]
    #[should_panic(expected = "Multiplication overflow")]
    fn decimal_from_ratio_panics_for_mul_overflow() {
        Decimal::from_ratio(u128::MAX, 1u128);
    }

    #[test]
    fn decimal_checked_from_ratio_does_not_panic() {
        assert_eq!(
            Decimal::checked_from_ratio(1u128, 0u128),
            Err(CheckedFromRatioError::DivideByZero)
        );

        assert_eq!(
            Decimal::checked_from_ratio(u128::MAX, 1u128),
            Err(CheckedFromRatioError::Overflow)
        );
    }

    #[test]
    fn decimal_implements_fraction() {
        let fraction = Decimal::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Uint128::from(1_234_567_000_000_000_000_000u128)
        );
        assert_eq!(
            fraction.denominator(),
            Uint128::from(1_000_000_000_000_000_000u128)
        );
    }

    #[test]
    fn decimal_from_str_works() {
        // Integers
        assert_eq!(Decimal::from_str("0").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("1").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("5").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("42").unwrap(), Decimal::percent(4200));
        assert_eq!(Decimal::from_str("000").unwrap(), Decimal::percent(0));
        assert_eq!(Decimal::from_str("001").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("005").unwrap(), Decimal::percent(500));
        assert_eq!(Decimal::from_str("0042").unwrap(), Decimal::percent(4200));

        // Decimals
        assert_eq!(Decimal::from_str("1.0").unwrap(), Decimal::percent(100));
        assert_eq!(Decimal::from_str("1.5").unwrap(), Decimal::percent(150));
        assert_eq!(Decimal::from_str("0.5").unwrap(), Decimal::percent(50));
        assert_eq!(Decimal::from_str("0.123").unwrap(), Decimal::permille(123));

        assert_eq!(Decimal::from_str("40.00").unwrap(), Decimal::percent(4000));
        assert_eq!(Decimal::from_str("04.00").unwrap(), Decimal::percent(400));
        assert_eq!(Decimal::from_str("00.40").unwrap(), Decimal::percent(40));
        assert_eq!(Decimal::from_str("00.04").unwrap(), Decimal::percent(4));

        // Can handle DECIMAL_PLACES fractional digits
        assert_eq!(
            Decimal::from_str("7.123456789012345678").unwrap(),
            Decimal(Uint128::from(7123456789012345678u128))
        );
        assert_eq!(
            Decimal::from_str("7.999999999999999999").unwrap(),
            Decimal(Uint128::from(7999999999999999999u128))
        );

        // Works for documented max value
        assert_eq!(
            Decimal::from_str("340282366920938463463.374607431768211455").unwrap(),
            Decimal::MAX
        );
    }

    #[test]
    fn decimal_from_str_errors_for_broken_whole_part() {
        match Decimal::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_broken_fractinal_part() {
        match Decimal::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_18_fractional_digits() {
        match Decimal::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits",)
            }
            e => panic!("Unexpected error: {:?}", e),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_invalid_number_of_dots() {
        match Decimal::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }

        match Decimal::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal::from_str("340282366920938463464").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }

        // Decimal
        match Decimal::from_str("340282366920938463464.0").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
        match Decimal::from_str("340282366920938463463.374607431768211456").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {:?}", e),
        }
    }

    #[test]
    fn decimal_atomics_works() {
        let zero = Decimal::zero();
        let one = Decimal::one();
        let half = Decimal::percent(50);
        let two = Decimal::percent(200);
        let max = Decimal::MAX;

        assert_eq!(zero.atomics(), Uint128::new(0));
        assert_eq!(one.atomics(), Uint128::new(1000000000000000000));
        assert_eq!(half.atomics(), Uint128::new(500000000000000000));
        assert_eq!(two.atomics(), Uint128::new(2000000000000000000));
        assert_eq!(max.atomics(), Uint128::MAX);
    }

    #[test]
    fn decimal_decimal_places_works() {
        let zero = Decimal::zero();
        let one = Decimal::one();
        let half = Decimal::percent(50);
        let two = Decimal::percent(200);
        let max = Decimal::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal_is_zero_works() {
        assert!(Decimal::zero().is_zero());
        assert!(Decimal::percent(0).is_zero());
        assert!(Decimal::permille(0).is_zero());

        assert!(!Decimal::one().is_zero());
        assert!(!Decimal::percent(123).is_zero());
        assert!(!Decimal::permille(1234).is_zero());
    }

    #[test]
    fn decimal_inv_works() {
        // d = 0
        assert_eq!(Decimal::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal::one().inv(), Some(Decimal::one()));

        // d > 1 exact
        assert_eq!(
            Decimal::from_str("2").unwrap().inv(),
            Some(Decimal::from_str("0.5").unwrap())
        );
        assert_eq!(
            Decimal::from_str("20").unwrap().inv(),
            Some(Decimal::from_str("0.05").unwrap())
        );
        assert_eq!(
            Decimal::from_str("200").unwrap().inv(),
            Some(Decimal::from_str("0.005").unwrap())
        );
        assert_eq!(
            Decimal::from_str("2000").unwrap().inv(),
            Some(Decimal::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            Decimal::from_str("3").unwrap().inv(),
            Some(Decimal::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            Decimal::from_str("6").unwrap().inv(),
            Some(Decimal::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            Decimal::from_str("0.5").unwrap().inv(),
            Some(Decimal::from_str("2").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.05").unwrap().inv(),
            Some(Decimal::from_str("20").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.005").unwrap().inv(),
            Some(Decimal::from_str("200").unwrap())
        );
        assert_eq!(
            Decimal::from_str("0.0005").unwrap().inv(),
            Some(Decimal::from_str("2000").unwrap())
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal_add_works() {
        let value = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal::DECIMAL_FRACTIONAL * Uint128::from(3u8) / Uint128::from(2u8)
        );

        assert_eq!(
            Decimal::percent(5) + Decimal::percent(4),
            Decimal::percent(9)
        );
        assert_eq!(Decimal::percent(5) + Decimal::zero(), Decimal::percent(5));
        assert_eq!(Decimal::zero() + Decimal::zero(), Decimal::zero());

        // works for refs
        let a = Decimal::percent(15);
        let b = Decimal::percent(25);
        let expected = Decimal::percent(40);
        assert_eq!(a + b, expected);
        assert_eq!(&a + b, expected);
        assert_eq!(a + &b, expected);
        assert_eq!(&a + &b, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal_add_overflow_panics() {
        let _value = Decimal::MAX + Decimal::percent(50);
    }

    #[test]
    fn decimal_add_assign_works() {
        let mut a = Decimal::percent(30);
        a += Decimal::percent(20);
        assert_eq!(a, Decimal::percent(50));

        // works for refs
        let mut a = Decimal::percent(15);
        let b = Decimal::percent(3);
        let expected = Decimal::percent(18);
        a += &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal_sub_works() {
        let value = Decimal::one() - Decimal::percent(50); // 0.5
        assert_eq!(value.0, Decimal::DECIMAL_FRACTIONAL / Uint128::from(2u8));

        assert_eq!(
            Decimal::percent(9) - Decimal::percent(4),
            Decimal::percent(5)
        );
        assert_eq!(Decimal::percent(16) - Decimal::zero(), Decimal::percent(16));
        assert_eq!(Decimal::percent(16) - Decimal::percent(16), Decimal::zero());
        assert_eq!(Decimal::zero() - Decimal::zero(), Decimal::zero());

        // works for refs
        let a = Decimal::percent(13);
        let b = Decimal::percent(6);
        let expected = Decimal::percent(7);
        assert_eq!(a - b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn decimal_sub_overflow_panics() {
        let _value = Decimal::zero() - Decimal::percent(50);
    }

    #[test]
    fn decimal_sub_assign_works() {
        let mut a = Decimal::percent(20);
        a -= Decimal::percent(2);
        assert_eq!(a, Decimal::percent(18));

        // works for refs
        let mut a = Decimal::percent(33);
        let b = Decimal::percent(13);
        let expected = Decimal::percent(20);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal_implements_mul() {
        let one = Decimal::one();
        let two = one + one;
        let half = Decimal::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(one * Decimal::percent(1), Decimal::percent(1));
        assert_eq!(one * Decimal::percent(10), Decimal::percent(10));
        assert_eq!(one * Decimal::percent(100), Decimal::percent(100));
        assert_eq!(one * Decimal::percent(1000), Decimal::percent(1000));
        assert_eq!(one * Decimal::MAX, Decimal::MAX);
        assert_eq!(Decimal::percent(0) * one, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * one, Decimal::percent(1));
        assert_eq!(Decimal::percent(10) * one, Decimal::percent(10));
        assert_eq!(Decimal::percent(100) * one, Decimal::percent(100));
        assert_eq!(Decimal::percent(1000) * one, Decimal::percent(1000));
        assert_eq!(Decimal::MAX * one, Decimal::MAX);

        // double
        assert_eq!(two * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(two * Decimal::percent(1), Decimal::percent(2));
        assert_eq!(two * Decimal::percent(10), Decimal::percent(20));
        assert_eq!(two * Decimal::percent(100), Decimal::percent(200));
        assert_eq!(two * Decimal::percent(1000), Decimal::percent(2000));
        assert_eq!(Decimal::percent(0) * two, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * two, Decimal::percent(2));
        assert_eq!(Decimal::percent(10) * two, Decimal::percent(20));
        assert_eq!(Decimal::percent(100) * two, Decimal::percent(200));
        assert_eq!(Decimal::percent(1000) * two, Decimal::percent(2000));

        // half
        assert_eq!(half * Decimal::percent(0), Decimal::percent(0));
        assert_eq!(half * Decimal::percent(1), Decimal::permille(5));
        assert_eq!(half * Decimal::percent(10), Decimal::percent(5));
        assert_eq!(half * Decimal::percent(100), Decimal::percent(50));
        assert_eq!(half * Decimal::percent(1000), Decimal::percent(500));
        assert_eq!(Decimal::percent(0) * half, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) * half, Decimal::permille(5));
        assert_eq!(Decimal::percent(10) * half, Decimal::percent(5));
        assert_eq!(Decimal::percent(100) * half, Decimal::percent(50));
        assert_eq!(Decimal::percent(1000) * half, Decimal::percent(500));

        // Move left
        let a = dec("123.127726548762582");
        assert_eq!(a * dec("1"), dec("123.127726548762582"));
        assert_eq!(a * dec("10"), dec("1231.27726548762582"));
        assert_eq!(a * dec("100"), dec("12312.7726548762582"));
        assert_eq!(a * dec("1000"), dec("123127.726548762582"));
        assert_eq!(a * dec("1000000"), dec("123127726.548762582"));
        assert_eq!(a * dec("1000000000"), dec("123127726548.762582"));
        assert_eq!(a * dec("1000000000000"), dec("123127726548762.582"));
        assert_eq!(a * dec("1000000000000000"), dec("123127726548762582"));
        assert_eq!(a * dec("1000000000000000000"), dec("123127726548762582000"));
        assert_eq!(dec("1") * a, dec("123.127726548762582"));
        assert_eq!(dec("10") * a, dec("1231.27726548762582"));
        assert_eq!(dec("100") * a, dec("12312.7726548762582"));
        assert_eq!(dec("1000") * a, dec("123127.726548762582"));
        assert_eq!(dec("1000000") * a, dec("123127726.548762582"));
        assert_eq!(dec("1000000000") * a, dec("123127726548.762582"));
        assert_eq!(dec("1000000000000") * a, dec("123127726548762.582"));
        assert_eq!(dec("1000000000000000") * a, dec("123127726548762582"));
        assert_eq!(dec("1000000000000000000") * a, dec("123127726548762582000"));

        // Move right
        let max = Decimal::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("340282366920938463463.374607431768211455")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("34028236692093846346.337460743176821145")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("3402823669209384634.633746074317682114")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("340282366920938463.463374607431768211")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("340282366920938.463463374607431768")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("340282366920.938463463374607431")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("340282366.920938463463374607")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("340282.366920938463463374")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("340.282366920938463463")
        );

        // works for refs
        let a = Decimal::percent(20);
        let b = Decimal::percent(30);
        let expected = Decimal::percent(6);
        assert_eq!(a * b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn decimal_mul_assign_works() {
        let mut a = Decimal::percent(15);
        a *= Decimal::percent(60);
        assert_eq!(a, Decimal::percent(9));

        // works for refs
        let mut a = Decimal::percent(50);
        let b = Decimal::percent(20);
        a *= &b;
        assert_eq!(a, Decimal::percent(10));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal_mul_overflow_panics() {
        let _value = Decimal::MAX * Decimal::percent(101);
    }

    #[test]
    fn decimal_checked_mul() {
        let test_data = [
            (Decimal::zero(), Decimal::zero()),
            (Decimal::zero(), Decimal::one()),
            (Decimal::one(), Decimal::zero()),
            (Decimal::percent(10), Decimal::zero()),
            (Decimal::percent(10), Decimal::percent(5)),
            (Decimal::MAX, Decimal::one()),
            (Decimal::MAX / Uint128::new(2), Decimal::percent(200)),
            (Decimal::permille(6), Decimal::permille(13)),
        ];

        // The regular std::ops::Mul is our source of truth for these tests.
        for (x, y) in test_data.into_iter() {
            assert_eq!(x * y, x.checked_mul(y).unwrap());
        }
    }

    #[test]
    fn decimal_checked_mul_overflow() {
        assert_eq!(
            Decimal::MAX.checked_mul(Decimal::percent(200)),
            Err(OverflowError {
                operation: crate::OverflowOperation::Mul,
                operand1: Decimal::MAX.to_string(),
                operand2: Decimal::percent(200).to_string(),
            })
        );
    }

    #[test]
    // in this test the Decimal is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint128::new(300);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128::new(450));

        // a*0
        let left = Uint128::new(300);
        let right = Decimal::zero();
        assert_eq!(left * right, Uint128::new(0));

        // 0*a
        let left = Uint128::new(0);
        let right = Decimal::one() + Decimal::percent(50); // 1.5
        assert_eq!(left * right, Uint128::new(0));
    }

    #[test]
    // in this test the Decimal is on the left
    fn decimal_uint128_multiply() {
        // a*b
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128::new(300);
        assert_eq!(left * right, Uint128::new(450));

        // 0*a
        let left = Decimal::zero();
        let right = Uint128::new(300);
        assert_eq!(left * right, Uint128::new(0));

        // a*0
        let left = Decimal::one() + Decimal::percent(50); // 1.5
        let right = Uint128::new(0);
        assert_eq!(left * right, Uint128::new(0));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal_implements_div() {
        let one = Decimal::one();
        let two = one + one;
        let half = Decimal::percent(50);

        // 1/x and x/1
        assert_eq!(one / Decimal::percent(1), Decimal::percent(10_000));
        assert_eq!(one / Decimal::percent(10), Decimal::percent(1_000));
        assert_eq!(one / Decimal::percent(100), Decimal::percent(100));
        assert_eq!(one / Decimal::percent(1000), Decimal::percent(10));
        assert_eq!(Decimal::percent(0) / one, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) / one, Decimal::percent(1));
        assert_eq!(Decimal::percent(10) / one, Decimal::percent(10));
        assert_eq!(Decimal::percent(100) / one, Decimal::percent(100));
        assert_eq!(Decimal::percent(1000) / one, Decimal::percent(1000));

        // double
        assert_eq!(two / Decimal::percent(1), Decimal::percent(20_000));
        assert_eq!(two / Decimal::percent(10), Decimal::percent(2_000));
        assert_eq!(two / Decimal::percent(100), Decimal::percent(200));
        assert_eq!(two / Decimal::percent(1000), Decimal::percent(20));
        assert_eq!(Decimal::percent(0) / two, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) / two, dec("0.005"));
        assert_eq!(Decimal::percent(10) / two, Decimal::percent(5));
        assert_eq!(Decimal::percent(100) / two, Decimal::percent(50));
        assert_eq!(Decimal::percent(1000) / two, Decimal::percent(500));

        // half
        assert_eq!(half / Decimal::percent(1), Decimal::percent(5_000));
        assert_eq!(half / Decimal::percent(10), Decimal::percent(500));
        assert_eq!(half / Decimal::percent(100), Decimal::percent(50));
        assert_eq!(half / Decimal::percent(1000), Decimal::percent(5));
        assert_eq!(Decimal::percent(0) / half, Decimal::percent(0));
        assert_eq!(Decimal::percent(1) / half, Decimal::percent(2));
        assert_eq!(Decimal::percent(10) / half, Decimal::percent(20));
        assert_eq!(Decimal::percent(100) / half, Decimal::percent(200));
        assert_eq!(Decimal::percent(1000) / half, Decimal::percent(2000));

        // Move right
        let a = dec("123127726548762582");
        assert_eq!(a / dec("1"), dec("123127726548762582"));
        assert_eq!(a / dec("10"), dec("12312772654876258.2"));
        assert_eq!(a / dec("100"), dec("1231277265487625.82"));
        assert_eq!(a / dec("1000"), dec("123127726548762.582"));
        assert_eq!(a / dec("1000000"), dec("123127726548.762582"));
        assert_eq!(a / dec("1000000000"), dec("123127726.548762582"));
        assert_eq!(a / dec("1000000000000"), dec("123127.726548762582"));
        assert_eq!(a / dec("1000000000000000"), dec("123.127726548762582"));
        assert_eq!(a / dec("1000000000000000000"), dec("0.123127726548762582"));
        assert_eq!(dec("1") / a, dec("0.000000000000000008"));
        assert_eq!(dec("10") / a, dec("0.000000000000000081"));
        assert_eq!(dec("100") / a, dec("0.000000000000000812"));
        assert_eq!(dec("1000") / a, dec("0.000000000000008121"));
        assert_eq!(dec("1000000") / a, dec("0.000000000008121647"));
        assert_eq!(dec("1000000000") / a, dec("0.000000008121647560"));
        assert_eq!(dec("1000000000000") / a, dec("0.000008121647560868"));
        assert_eq!(dec("1000000000000000") / a, dec("0.008121647560868164"));
        assert_eq!(dec("1000000000000000000") / a, dec("8.121647560868164773"));

        // Move left
        let a = dec("0.123127726548762582");
        assert_eq!(a / dec("1.0"), dec("0.123127726548762582"));
        assert_eq!(a / dec("0.1"), dec("1.23127726548762582"));
        assert_eq!(a / dec("0.01"), dec("12.3127726548762582"));
        assert_eq!(a / dec("0.001"), dec("123.127726548762582"));
        assert_eq!(a / dec("0.000001"), dec("123127.726548762582"));
        assert_eq!(a / dec("0.000000001"), dec("123127726.548762582"));
        assert_eq!(a / dec("0.000000000001"), dec("123127726548.762582"));
        assert_eq!(a / dec("0.000000000000001"), dec("123127726548762.582"));
        assert_eq!(a / dec("0.000000000000000001"), dec("123127726548762582"));

        assert_eq!(
            Decimal::percent(15) / Decimal::percent(60),
            Decimal::percent(25)
        );

        // works for refs
        let a = Decimal::percent(100);
        let b = Decimal::percent(20);
        let expected = Decimal::percent(500);
        assert_eq!(a / b, expected);
        assert_eq!(&a / b, expected);
        assert_eq!(a / &b, expected);
        assert_eq!(&a / &b, expected);
    }

    #[test]
    fn decimal_div_assign_works() {
        let mut a = Decimal::percent(15);
        a /= Decimal::percent(20);
        assert_eq!(a, Decimal::percent(75));

        // works for refs
        let mut a = Decimal::percent(50);
        let b = Decimal::percent(20);
        a /= &b;
        assert_eq!(a, Decimal::percent(250));
    }

    #[test]
    #[should_panic(expected = "Division failed - multiplication overflow")]
    fn decimal_div_overflow_panics() {
        let _value = Decimal::MAX / Decimal::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn decimal_div_by_zero_panics() {
        let _value = Decimal::one() / Decimal::zero();
    }

    #[test]
    fn decimal_uint128_division() {
        // a/b
        let left = Decimal::percent(150); // 1.5
        let right = Uint128::new(3);
        assert_eq!(left / right, Decimal::percent(50));

        // 0/a
        let left = Decimal::zero();
        let right = Uint128::new(300);
        assert_eq!(left / right, Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_divide_by_zero() {
        let left = Decimal::percent(150); // 1.5
        let right = Uint128::new(0);
        let _result = left / right;
    }

    #[test]
    fn decimal_uint128_div_assign() {
        // a/b
        let mut dec = Decimal::percent(150); // 1.5
        dec /= Uint128::new(3);
        assert_eq!(dec, Decimal::percent(50));

        // 0/a
        let mut dec = Decimal::zero();
        dec /= Uint128::new(300);
        assert_eq!(dec, Decimal::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal_uint128_div_assign_by_zero() {
        // a/0
        let mut dec = Decimal::percent(50);
        dec /= Uint128::new(0);
    }

    #[test]
    fn decimal_uint128_sqrt() {
        assert_eq!(Decimal::percent(900).sqrt(), Decimal::percent(300));

        assert!(Decimal::percent(316) < Decimal::percent(1000).sqrt());
        assert!(Decimal::percent(1000).sqrt() < Decimal::percent(317));
    }

    /// sqrt(2) is an irrational number, i.e. all 18 decimal places should be used.
    #[test]
    fn decimal_uint128_sqrt_is_precise() {
        assert_eq!(
            Decimal::from_str("2").unwrap().sqrt(),
            Decimal::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
        );
    }

    #[test]
    fn decimal_uint128_sqrt_does_not_overflow() {
        assert_eq!(
            Decimal::from_str("400").unwrap().sqrt(),
            Decimal::from_str("20").unwrap()
        );
    }

    #[test]
    fn decimal_uint128_sqrt_intermediate_precision_used() {
        assert_eq!(
            Decimal::from_str("400001").unwrap().sqrt(),
            // The last two digits (27) are truncated below due to the algorithm
            // we use. Larger numbers will cause less precision.
            // https://www.wolframalpha.com/input/?i=sqrt%28400001%29
            Decimal::from_str("632.456322602596803200").unwrap()
        );
    }

    #[test]
    fn decimal_checked_pow() {
        for exp in 0..10 {
            assert_eq!(Decimal::one().checked_pow(exp).unwrap(), Decimal::one());
        }

        // This case is mathematically undefined but we ensure consistency with Rust stdandard types
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
        assert_eq!(Decimal::zero().checked_pow(0).unwrap(), Decimal::one());

        for exp in 1..10 {
            assert_eq!(Decimal::zero().checked_pow(exp).unwrap(), Decimal::zero());
        }

        for num in &[
            Decimal::percent(50),
            Decimal::percent(99),
            Decimal::percent(200),
        ] {
            assert_eq!(num.checked_pow(0).unwrap(), Decimal::one())
        }

        assert_eq!(
            Decimal::percent(20).checked_pow(2).unwrap(),
            Decimal::percent(4)
        );

        assert_eq!(
            Decimal::percent(20).checked_pow(3).unwrap(),
            Decimal::permille(8)
        );

        assert_eq!(
            Decimal::percent(200).checked_pow(4).unwrap(),
            Decimal::percent(1600)
        );

        assert_eq!(
            Decimal::percent(200).checked_pow(4).unwrap(),
            Decimal::percent(1600)
        );

        assert_eq!(
            Decimal::percent(700).checked_pow(5).unwrap(),
            Decimal::percent(1680700)
        );

        assert_eq!(
            Decimal::percent(700).checked_pow(8).unwrap(),
            Decimal::percent(576480100)
        );

        assert_eq!(
            Decimal::percent(700).checked_pow(10).unwrap(),
            Decimal::percent(28247524900)
        );

        assert_eq!(
            Decimal::percent(120).checked_pow(123).unwrap(),
            Decimal(5486473221892422150877397607u128.into())
        );

        assert_eq!(
            Decimal::percent(10).checked_pow(2).unwrap(),
            Decimal(10000000000000000u128.into())
        );

        assert_eq!(
            Decimal::percent(10).checked_pow(18).unwrap(),
            Decimal(1u128.into())
        );
    }

    #[test]
    fn decimal_checked_pow_overflow() {
        assert_eq!(
            Decimal::MAX.checked_pow(2),
            Err(OverflowError {
                operation: crate::OverflowOperation::Pow,
                operand1: Decimal::MAX.to_string(),
                operand2: "2".to_string(),
            })
        );
    }

    #[test]
    fn decimal_to_string() {
        // Integers
        assert_eq!(Decimal::zero().to_string(), "0");
        assert_eq!(Decimal::one().to_string(), "1");
        assert_eq!(Decimal::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal::percent(125).to_string(), "1.25");
        assert_eq!(Decimal::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal::percent(3).to_string(), "0.03");
        assert_eq!(Decimal::permille(987).to_string(), "0.987");

        assert_eq!(
            Decimal(Uint128::from(1u128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10u128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100u128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000u128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000u128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000u128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000u128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000u128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000u128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000000u128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000u128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000u128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000000u128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000000u128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            Decimal(Uint128::from(1000000000000000u128)).to_string(),
            "0.001"
        );
        assert_eq!(
            Decimal(Uint128::from(10000000000000000u128)).to_string(),
            "0.01"
        );
        assert_eq!(
            Decimal(Uint128::from(100000000000000000u128)).to_string(),
            "0.1"
        );
    }

    #[test]
    fn decimal_iter_sum() {
        let items = vec![
            Decimal::zero(),
            Decimal(Uint128::from(2u128)),
            Decimal(Uint128::from(2u128)),
        ];
        assert_eq!(items.iter().sum::<Decimal>(), Decimal(Uint128::from(4u128)));
        assert_eq!(
            items.into_iter().sum::<Decimal>(),
            Decimal(Uint128::from(4u128))
        );

        let empty: Vec<Decimal> = vec![];
        assert_eq!(Decimal::zero(), empty.iter().sum::<Decimal>());
    }

    #[test]
    fn decimal_serialize() {
        assert_eq!(to_vec(&Decimal::zero()).unwrap(), br#""0""#);
        assert_eq!(to_vec(&Decimal::one()).unwrap(), br#""1""#);
        assert_eq!(to_vec(&Decimal::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_vec(&Decimal::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(to_vec(&Decimal::percent(876)).unwrap(), br#""8.76""#);
        assert_eq!(to_vec(&Decimal::percent(8765)).unwrap(), br#""87.65""#);
    }

    #[test]
    fn decimal_deserialize() {
        assert_eq!(from_slice::<Decimal>(br#""0""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""1""#).unwrap(), Decimal::one());
        assert_eq!(from_slice::<Decimal>(br#""000""#).unwrap(), Decimal::zero());
        assert_eq!(from_slice::<Decimal>(br#""001""#).unwrap(), Decimal::one());

        assert_eq!(
            from_slice::<Decimal>(br#""0.08""#).unwrap(),
            Decimal::percent(8)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""0.87""#).unwrap(),
            Decimal::percent(87)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""8.76""#).unwrap(),
            Decimal::percent(876)
        );
        assert_eq!(
            from_slice::<Decimal>(br#""87.65""#).unwrap(),
            Decimal::percent(8765)
        );
    }

    #[test]
    fn decimal_abs_diff_works() {
        let a = Decimal::percent(285);
        let b = Decimal::percent(200);
        let expected = Decimal::percent(85);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal_rem_works() {
        // 4.02 % 1.11 = 0.69
        assert_eq!(
            Decimal::percent(402) % Decimal::percent(111),
            Decimal::percent(69)
        );

        // 15.25 % 4 = 3.25
        assert_eq!(
            Decimal::percent(1525) % Decimal::percent(400),
            Decimal::percent(325)
        );

        let a = Decimal::percent(318);
        let b = Decimal::percent(317);
        let expected = Decimal::percent(1);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn decimal_rem_assign_works() {
        let mut a = Decimal::percent(17673);
        a %= Decimal::percent(2362);
        assert_eq!(a, Decimal::percent(1139)); // 176.73 % 23.62 = 11.39

        let mut a = Decimal::percent(4262);
        let b = Decimal::percent(1270);
        a %= &b;
        assert_eq!(a, Decimal::percent(452)); // 42.62 % 12.7 = 4.52
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn decimal_rem_panics_for_zero() {
        let _ = Decimal::percent(777) % Decimal::zero();
    }

    #[test]
    fn decimal_checked_methods() {
        // checked add
        assert_eq!(
            Decimal::percent(402)
                .checked_add(Decimal::percent(111))
                .unwrap(),
            Decimal::percent(513)
        );
        assert!(matches!(
            Decimal::MAX.checked_add(Decimal::percent(1)),
            Err(OverflowError { .. })
        ));

        // checked sub
        assert_eq!(
            Decimal::percent(1111)
                .checked_sub(Decimal::percent(111))
                .unwrap(),
            Decimal::percent(1000)
        );
        assert!(matches!(
            Decimal::zero().checked_sub(Decimal::percent(1)),
            Err(OverflowError { .. })
        ));

        // checked div
        assert_eq!(
            Decimal::percent(30)
                .checked_div(Decimal::percent(200))
                .unwrap(),
            Decimal::percent(15)
        );
        assert_eq!(
            Decimal::percent(88)
                .checked_div(Decimal::percent(20))
                .unwrap(),
            Decimal::percent(440)
        );
        assert!(matches!(
            Decimal::MAX.checked_div(Decimal::zero()),
            Err(CheckedFromRatioError::DivideByZero {})
        ));
        assert!(matches!(
            Decimal::MAX.checked_div(Decimal::percent(1)),
            Err(CheckedFromRatioError::Overflow {})
        ));

        // checked rem
        assert_eq!(
            Decimal::percent(402)
                .checked_rem(Decimal::percent(111))
                .unwrap(),
            Decimal::percent(69)
        );
        assert_eq!(
            Decimal::percent(1525)
                .checked_rem(Decimal::percent(400))
                .unwrap(),
            Decimal::percent(325)
        );
        assert!(matches!(
            Decimal::MAX.checked_rem(Decimal::zero()),
            Err(DivideByZeroError { .. })
        ));
    }

    #[test]
    fn decimal_pow_works() {
        assert_eq!(Decimal::percent(200).pow(2), Decimal::percent(400));
        assert_eq!(Decimal::percent(200).pow(10), Decimal::percent(102400));
    }

    #[test]
    #[should_panic]
    fn decimal_pow_overflow_panics() {
        _ = Decimal::MAX.pow(2u32);
    }

    #[test]
    fn decimal_saturating_works() {
        assert_eq!(
            Decimal::percent(200).saturating_add(Decimal::percent(200)),
            Decimal::percent(400)
        );
        assert_eq!(
            Decimal::MAX.saturating_add(Decimal::percent(200)),
            Decimal::MAX
        );
        assert_eq!(
            Decimal::percent(200).saturating_sub(Decimal::percent(100)),
            Decimal::percent(100)
        );
        assert_eq!(
            Decimal::zero().saturating_sub(Decimal::percent(200)),
            Decimal::zero()
        );
        assert_eq!(
            Decimal::percent(200).saturating_mul(Decimal::percent(50)),
            Decimal::percent(100)
        );
        assert_eq!(
            Decimal::MAX.saturating_mul(Decimal::percent(200)),
            Decimal::MAX
        );
        assert_eq!(
            Decimal::percent(400).saturating_pow(2u32),
            Decimal::percent(1600)
        );
        assert_eq!(Decimal::MAX.saturating_pow(2u32), Decimal::MAX);
    }

    #[test]
    fn decimal_rounding() {
        assert_eq!(Decimal::one().floor(), Decimal::one());
        assert_eq!(Decimal::percent(150).floor(), Decimal::one());
        assert_eq!(Decimal::percent(199).floor(), Decimal::one());
        assert_eq!(Decimal::percent(200).floor(), Decimal::percent(200));
        assert_eq!(Decimal::percent(99).floor(), Decimal::zero());

        assert_eq!(Decimal::one().ceil(), Decimal::one());
        assert_eq!(Decimal::percent(150).ceil(), Decimal::percent(200));
        assert_eq!(Decimal::percent(199).ceil(), Decimal::percent(200));
        assert_eq!(Decimal::percent(99).ceil(), Decimal::one());
        assert_eq!(Decimal(Uint128::from(1u128)).ceil(), Decimal::one());
    }

    #[test]
    #[should_panic(expected = "attempt to ceil with overflow")]
    fn decimal_ceil_panics() {
        let _ = Decimal::MAX.ceil();
    }

    #[test]
    fn decimal_checked_ceil() {
        assert_eq!(
            Decimal::percent(199).checked_ceil(),
            Ok(Decimal::percent(200))
        );
        assert!(matches!(
            Decimal::MAX.checked_ceil(),
            Err(RoundUpOverflowError { .. })
        ));
    }

    #[test]
    fn decimal_to_uint_floor_works() {
        let d = Decimal::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(12));
        let d = Decimal::from_str("12.345").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(12));
        let d = Decimal::from_str("12.999").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(12));
        let d = Decimal::from_str("0.98451384").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(0));

        let d = Decimal::from_str("75.0").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(75));
        let d = Decimal::from_str("0.0").unwrap();
        assert_eq!(d.to_uint_floor(), Uint128::new(0));

        let d = Decimal::MAX;
        assert_eq!(d.to_uint_floor(), Uint128::new(340282366920938463463));

        // Does the same as the old workaround `Uint128::one() * my_decimal`.
        // This block can be deleted as part of https://github.com/CosmWasm/cosmwasm/issues/1485.
        let tests = vec![
            Decimal::from_str("12.345").unwrap(),
            Decimal::from_str("0.98451384").unwrap(),
            Decimal::from_str("178.0").unwrap(),
            Decimal::MIN,
            Decimal::MAX,
        ];
        for my_decimal in tests.into_iter() {
            assert_eq!(my_decimal.to_uint_floor(), Uint128::one() * my_decimal);
        }
    }

    #[test]
    fn decimal_to_uint_ceil_works() {
        let d = Decimal::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint128::new(13));
        let d = Decimal::from_str("12.345").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint128::new(13));
        let d = Decimal::from_str("12.999").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint128::new(13));

        let d = Decimal::from_str("75.0").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint128::new(75));
        let d = Decimal::from_str("0.0").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint128::new(0));

        let d = Decimal::MAX;
        assert_eq!(d.to_uint_ceil(), Uint128::new(340282366920938463464));
    }

    #[test]
    fn decimal_partial_eq() {
        let test_cases = [
            ("1", "1", true),
            ("0.5", "0.5", true),
            ("0.5", "0.51", false),
            ("0", "0.00000", true),
        ]
        .into_iter()
        .map(|(lhs, rhs, expected)| (dec(lhs), dec(rhs), expected));

        #[allow(clippy::op_ref)]
        for (lhs, rhs, expected) in test_cases {
            assert_eq!(lhs == rhs, expected);
            assert_eq!(&lhs == rhs, expected);
            assert_eq!(lhs == &rhs, expected);
            assert_eq!(&lhs == &rhs, expected);
        }
    }

    #[test]
    fn decimal_implements_debug() {
        let decimal = Decimal::from_str("123.45").unwrap();
        assert_eq!(format!("{:?}", decimal), "Decimal(123.45)");

        let test_cases = ["5", "5.01", "42", "0", "2"];
        for s in test_cases {
            let decimal = Decimal::from_str(s).unwrap();
            let expected = format!("Decimal({})", s);
            assert_eq!(format!("{:?}", decimal), expected);
        }
    }
}
