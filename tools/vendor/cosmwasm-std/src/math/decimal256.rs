use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::ops::{Add, AddAssign, Div, DivAssign, Mul, MulAssign, Rem, RemAssign, Sub, SubAssign};
use core::str::FromStr;
use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyRatioError, DivideByZeroError, OverflowError,
    OverflowOperation, RoundUpOverflowError, StdError,
};
use crate::{forward_ref_partial_eq, Decimal, SignedDecimal, SignedDecimal256, Uint512};

use super::Fraction;
use super::Isqrt;
use super::Uint256;

/// A fixed-point decimal value with 18 fractional digits, i.e. Decimal256(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is
/// 115792089237316195423570985008687907853269984665640564039457.584007913129639935
/// (which is (2^256 - 1) / 10^18)
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct Decimal256(#[schemars(with = "String")] Uint256);

forward_ref_partial_eq!(Decimal256, Decimal256);

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Decimal256 range exceeded")]
pub struct Decimal256RangeExceeded;

impl Decimal256 {
    const DECIMAL_FRACTIONAL: Uint256 = // 1*10**18
        Uint256::from_u128(1_000_000_000_000_000_000);
    const DECIMAL_FRACTIONAL_SQUARED: Uint256 = // 1*10**36
        Uint256::from_u128(1_000_000_000_000_000_000_000_000_000_000_000_000);

    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18;
    /// The largest value that can be represented by this decimal type.
    pub const MAX: Self = Self(Uint256::MAX);
    /// The smallest value that can be represented by this decimal type.
    pub const MIN: Self = Self(Uint256::MIN);

    /// Creates a Decimal256 from Uint256
    /// This is equivalent to `Decimal256::from_atomics(value, 18)` but usable in a const context.
    pub const fn new(value: Uint256) -> Self {
        Self(value)
    }

    /// Creates a Decimal256 from u128
    /// This is equivalent to `Decimal256::from_atomics(value, 18)` but usable in a const context.
    pub const fn raw(value: u128) -> Self {
        Self(Uint256::from_u128(value))
    }

    /// Create a 1.0 Decimal256
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a 0.0 Decimal256
    #[inline]
    pub const fn zero() -> Self {
        Self(Uint256::zero())
    }

    /// Convert x% into Decimal256
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal256;
    /// const HALF: Decimal256 = Decimal256::percent(50);
    ///
    /// assert_eq!(HALF, Decimal256::from_str("0.5").unwrap());
    /// ```
    pub const fn percent(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**16 is well in u128 range
        let atomics = (x as u128) * 10_000_000_000_000_000;
        Self(Uint256::from_u128(atomics))
    }

    /// Convert permille (x/1000) into Decimal256
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal256;
    /// const HALF: Decimal256 = Decimal256::permille(500);
    ///
    /// assert_eq!(HALF, Decimal256::from_str("0.5").unwrap());
    /// ```
    pub const fn permille(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**15 is well in u128 range
        let atomics = (x as u128) * 1_000_000_000_000_000;
        Self(Uint256::from_u128(atomics))
    }

    /// Convert basis points (x/10000) into Decimal256
    ///
    /// ## Examples
    ///
    /// ```
    /// # use std::str::FromStr;
    /// # use cosmwasm_std::Decimal256;
    /// const TWO_BPS: Decimal256 = Decimal256::bps(2);
    /// const HALF: Decimal256 = Decimal256::bps(5000);
    ///
    /// assert_eq!(TWO_BPS, Decimal256::from_str("0.0002").unwrap());
    /// assert_eq!(HALF, Decimal256::from_str("0.5").unwrap());
    /// ```
    pub const fn bps(x: u64) -> Self {
        // multiplication does not overflow since `u64::MAX` * 10**14 is well in u128 range
        let atomics = (x as u128) * 100_000_000_000_000;
        Self(Uint256::from_u128(atomics))
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
    /// # use cosmwasm_std::{Decimal256, Uint256};
    /// let a = Decimal256::from_atomics(1234u64, 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = Decimal256::from_atomics(1234u128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = Decimal256::from_atomics(1u64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    ///
    /// let a = Decimal256::from_atomics(Uint256::MAX, 18).unwrap();
    /// assert_eq!(a, Decimal256::MAX);
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Uint256>,
        decimal_places: u32,
    ) -> Result<Self, Decimal256RangeExceeded> {
        let atomics = atomics.into();
        const TEN: Uint256 = Uint256::from_be_bytes([
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 10,
        ]);
        Ok(match decimal_places.cmp(&Self::DECIMAL_PLACES) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = TEN.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| Decimal256RangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = TEN.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Uint256 range.
                    // Any Uint256 `x` divided by `factor` with `factor > Uint256::MAX` is 0.
                    // Try e.g. Python3: `(2**256-1) // 2**256`
                    Self(Uint256::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a Decimal256
    pub fn from_ratio(numerator: impl Into<Uint256>, denominator: impl Into<Uint256>) -> Self {
        match Decimal256::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a Decimal256
    pub fn checked_from_ratio(
        numerator: impl Into<Uint256>,
        denominator: impl Into<Uint256>,
    ) -> Result<Self, CheckedFromRatioError> {
        let numerator: Uint256 = numerator.into();
        let denominator: Uint256 = denominator.into();
        match numerator.checked_multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator) {
            Ok(ratio) => {
                // numerator * DECIMAL_FRACTIONAL / denominator
                Ok(Self(ratio))
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
    /// # use cosmwasm_std::{Decimal256, Uint256};
    /// # use core::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = Decimal256::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Uint256::from(1234000000000000000u128));
    ///
    /// // Smallest possible value
    /// let b = Decimal256::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Uint256::from(1u128));
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> Uint256 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`Decimal256::atomics()`].
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
                .checked_add(Decimal256::one())
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

    /// Multiplies one `Decimal256` by another, returning an `OverflowError` if an overflow occurred.
    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        let result_as_uint512 = self.numerator().full_mul(other.numerator())
            / Uint512::from_uint256(Self::DECIMAL_FRACTIONAL); // from_uint128 is a const method and should be "free"
        result_as_uint512
            .try_into()
            .map(Self)
            .map_err(|_| OverflowError {
                operation: OverflowOperation::Mul,
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

        fn inner(mut x: Decimal256, mut n: u32) -> Result<Decimal256, OverflowError> {
            if n == 0 {
                return Ok(Decimal256::one());
            }

            let mut y = Decimal256::one();

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
            operation: OverflowOperation::Pow,
            operand1: self.to_string(),
            operand2: exp.to_string(),
        })
    }

    pub fn checked_div(self, other: Self) -> Result<Self, CheckedFromRatioError> {
        Decimal256::checked_from_ratio(self.numerator(), other.numerator())
    }

    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .map_err(|_| DivideByZeroError::new(self))
    }

    /// Returns the approximate square root as a Decimal256.
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
        // The max precision is something like `18 - log10(self.0) / 2`.
        (0..=Self::DECIMAL_PLACES / 2)
            .rev()
            .find_map(|i| self.sqrt_with_precision(i))
            // The last step (i = 0) is guaranteed to succeed because `isqrt(Uint256::MAX) * 10^9` does not overflow
            .unwrap()
    }

    /// Lower precision means more aggressive rounding, but less risk of overflow.
    /// Precision *must* be a number between 0 and 9 (inclusive).
    ///
    /// Returns `None` if the internal multiplication overflows.
    #[must_use = "this returns the result of the operation, without modifying the original"]
    fn sqrt_with_precision(&self, precision: u32) -> Option<Self> {
        let inner_mul = Uint256::from(100u128).pow(precision);
        self.0.checked_mul(inner_mul).ok().map(|inner| {
            let outer_mul = Uint256::from(10u128).pow(Self::DECIMAL_PLACES / 2 - precision);
            Self(inner.isqrt().checked_mul(outer_mul).unwrap())
        })
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn abs_diff(self, other: Self) -> Self {
        if self < other {
            other - self
        } else {
            self - other
        }
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
    /// use core::str::FromStr;
    /// use cosmwasm_std::{Decimal256, Uint256};
    ///
    /// let d = Decimal256::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint256::from(12u64));
    ///
    /// let d = Decimal256::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint256::from(12u64));
    ///
    /// let d = Decimal256::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_floor(), Uint256::from(75u64));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_uint_floor(self) -> Uint256 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Converts this decimal to an unsigned integer by rounting up
    /// to the next integer, e.g. 22.3 becomes 23.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{Decimal256, Uint256};
    ///
    /// let d = Decimal256::from_str("12.345").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint256::from(13u64));
    ///
    /// let d = Decimal256::from_str("12.999").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint256::from(13u64));
    ///
    /// let d = Decimal256::from_str("75.0").unwrap();
    /// assert_eq!(d.to_uint_ceil(), Uint256::from(75u64));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_uint_ceil(self) -> Uint256 {
        // Using `q = 1 + ((x - 1) / y); // if x != 0` with unsigned integers x, y, q
        // from https://stackoverflow.com/a/2745086/2013738. We know `x + y` CAN overflow.
        let x = self.0;
        let y = Self::DECIMAL_FRACTIONAL;
        if x.is_zero() {
            Uint256::zero()
        } else {
            Uint256::one() + ((x - Uint256::one()) / y)
        }
    }
}

impl Fraction<Uint256> for Decimal256 {
    #[inline]
    fn numerator(&self) -> Uint256 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Uint256 {
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
            Some(Self(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl From<Decimal> for Decimal256 {
    fn from(input: Decimal) -> Self {
        // Unwrap is safe because Decimal256 and Decimal have the same decimal places.
        // Every Decimal value can be stored in Decimal256.
        Decimal256::from_atomics(input.atomics(), input.decimal_places()).unwrap()
    }
}

impl TryFrom<SignedDecimal> for Decimal256 {
    type Error = Decimal256RangeExceeded;

    fn try_from(value: SignedDecimal) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(Decimal256)
            .map_err(|_| Decimal256RangeExceeded)
    }
}

impl TryFrom<SignedDecimal256> for Decimal256 {
    type Error = Decimal256RangeExceeded;

    fn try_from(value: SignedDecimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(Decimal256)
            .map_err(|_| Decimal256RangeExceeded)
    }
}

impl FromStr for Decimal256 {
    type Err = StdError;

    /// Converts the decimal string to a Decimal256
    /// Possible inputs: "1.23", "1", "000012", "1.123000000"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let whole = whole_part
            .parse::<Uint256>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<Uint256>()
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
            let fractional_factor = Uint256::from(10u128).pow(exp);
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

        Ok(Self(atomics))
    }
}

impl fmt::Display for Decimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::DECIMAL_FRACTIONAL;
        let fractional = (self.0).checked_rem(Self::DECIMAL_FRACTIONAL).unwrap();

        if fractional.is_zero() {
            write!(f, "{whole}")
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

impl fmt::Debug for Decimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Decimal256({self})")
    }
}

impl Add for Decimal256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self(self.0 + other.0)
    }
}
forward_ref_binop!(impl Add, add for Decimal256, Decimal256);

impl AddAssign for Decimal256 {
    fn add_assign(&mut self, rhs: Decimal256) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for Decimal256, Decimal256);

impl Sub for Decimal256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        Self(self.0 - other.0)
    }
}
forward_ref_binop!(impl Sub, sub for Decimal256, Decimal256);

impl SubAssign for Decimal256 {
    fn sub_assign(&mut self, rhs: Decimal256) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for Decimal256, Decimal256);

impl Mul for Decimal256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // Decimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_uint512 = self.numerator().full_mul(other.numerator())
            / Uint512::from_uint256(Self::DECIMAL_FRACTIONAL); // from_uint256 is a const method and should be "free"
        match result_as_uint512.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}
forward_ref_binop!(impl Mul, mul for Decimal256, Decimal256);

impl MulAssign for Decimal256 {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for Decimal256, Decimal256);

/// Both d*u and u*d with d: Decimal256 and u: Uint256 returns an Uint256. There is no
/// specific reason for this decision other than the initial use cases we have. If you
/// need a Decimal256 result for the same calculation, use Decimal256(d*u) or Decimal256(u*d).
impl Mul<Decimal256> for Uint256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, rhs: Decimal256) -> Self::Output {
        // 0*a and b*0 is always 0
        if self.is_zero() || rhs.is_zero() {
            return Uint256::zero();
        }
        self.multiply_ratio(rhs.0, Decimal256::DECIMAL_FRACTIONAL)
    }
}

impl Mul<Uint256> for Decimal256 {
    type Output = Uint256;

    fn mul(self, rhs: Uint256) -> Self::Output {
        rhs * self
    }
}

impl Div for Decimal256 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match Decimal256::checked_from_ratio(self.numerator(), other.numerator()) {
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
forward_ref_binop!(impl Div, div for Decimal256, Decimal256);

impl DivAssign for Decimal256 {
    fn div_assign(&mut self, rhs: Decimal256) {
        *self = *self / rhs;
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for Decimal256, Decimal256);

impl Div<Uint256> for Decimal256 {
    type Output = Self;

    fn div(self, rhs: Uint256) -> Self::Output {
        Self(self.0 / rhs)
    }
}

impl DivAssign<Uint256> for Decimal256 {
    fn div_assign(&mut self, rhs: Uint256) {
        self.0 /= rhs;
    }
}

impl Rem for Decimal256 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for Decimal256, Decimal256);

impl RemAssign<Decimal256> for Decimal256 {
    fn rem_assign(&mut self, rhs: Decimal256) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for Decimal256, Decimal256);

impl<A> core::iter::Sum<A> for Decimal256
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for Decimal256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for Decimal256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(Decimal256Visitor)
    }
}

struct Decimal256Visitor;

impl<'de> de::Visitor<'de> for Decimal256Visitor {
    type Value = Decimal256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match Self::Value::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{v}': {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::StdError;
    use crate::{from_json, to_json_vec};

    fn dec(input: &str) -> Decimal256 {
        Decimal256::from_str(input).unwrap()
    }

    #[test]
    fn decimal256_new() {
        let expected = Uint256::from(300u128);
        assert_eq!(Decimal256::new(expected).0, expected);
    }

    #[test]
    fn decimal256_raw() {
        let value = 300u128;
        let expected = Uint256::from(value);
        assert_eq!(Decimal256::raw(value).0, expected);
    }

    #[test]
    fn decimal256_one() {
        let value = Decimal256::one();
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn decimal256_zero() {
        let value = Decimal256::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn decimal256_percent() {
        let value = Decimal256::percent(50);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(2u8));
    }

    #[test]
    fn decimal256_permille() {
        let value = Decimal256::permille(125);
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(8u8));
    }

    #[test]
    fn decimal256_bps() {
        let value = Decimal256::bps(125);
        assert_eq!(
            value.0,
            Decimal256::DECIMAL_FRACTIONAL / Uint256::from(80u8)
        );
    }

    #[test]
    fn decimal256_from_atomics_works() {
        let one = Decimal256::one();
        let two = one + one;

        assert_eq!(Decimal256::from_atomics(1u128, 0).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(10u128, 1).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(100u128, 2).unwrap(), one);
        assert_eq!(Decimal256::from_atomics(1000u128, 3).unwrap(), one);
        assert_eq!(
            Decimal256::from_atomics(1000000000000000000u128, 18).unwrap(),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(10000000000000000000u128, 19).unwrap(),
            one
        );
        assert_eq!(
            Decimal256::from_atomics(100000000000000000000u128, 20).unwrap(),
            one
        );

        assert_eq!(Decimal256::from_atomics(2u128, 0).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(20u128, 1).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(200u128, 2).unwrap(), two);
        assert_eq!(Decimal256::from_atomics(2000u128, 3).unwrap(), two);
        assert_eq!(
            Decimal256::from_atomics(2000000000000000000u128, 18).unwrap(),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(20000000000000000000u128, 19).unwrap(),
            two
        );
        assert_eq!(
            Decimal256::from_atomics(200000000000000000000u128, 20).unwrap(),
            two
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            Decimal256::from_atomics(4321u128, 20).unwrap(),
            Decimal256::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(6789u128, 20).unwrap(),
            Decimal256::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 38).unwrap(),
            Decimal256::from_str("3.402823669209384634").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 39).unwrap(),
            Decimal256::from_str("0.340282366920938463").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 45).unwrap(),
            Decimal256::from_str("0.000000340282366920").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 51).unwrap(),
            Decimal256::from_str("0.000000000000340282").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 56).unwrap(),
            Decimal256::from_str("0.000000000000000003").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, 57).unwrap(),
            Decimal256::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            Decimal256::from_atomics(u128::MAX, u32::MAX).unwrap(),
            Decimal256::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = Decimal256::MAX;
        assert_eq!(
            Decimal256::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Overflow is only possible with digits < 18
        let result = Decimal256::from_atomics(Uint256::MAX, 17);
        assert_eq!(result.unwrap_err(), Decimal256RangeExceeded);
    }

    #[test]
    fn decimal256_from_ratio_works() {
        // 1.0
        assert_eq!(Decimal256::from_ratio(1u128, 1u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(53u128, 53u128), Decimal256::one());
        assert_eq!(Decimal256::from_ratio(125u128, 125u128), Decimal256::one());

        // 1.5
        assert_eq!(
            Decimal256::from_ratio(3u128, 2u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(150u128, 100u128),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_ratio(333u128, 222u128),
            Decimal256::percent(150)
        );

        // 0.125
        assert_eq!(
            Decimal256::from_ratio(1u64, 8u64),
            Decimal256::permille(125)
        );
        assert_eq!(
            Decimal256::from_ratio(125u64, 1000u64),
            Decimal256::permille(125)
        );

        // 1/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(1u64, 3u64),
            Decimal256(Uint256::from_str("333333333333333333").unwrap())
        );

        // 2/3 (result floored)
        assert_eq!(
            Decimal256::from_ratio(2u64, 3u64),
            Decimal256(Uint256::from_str("666666666666666666").unwrap())
        );

        // large inputs
        assert_eq!(Decimal256::from_ratio(0u128, u128::MAX), Decimal256::zero());
        assert_eq!(
            Decimal256::from_ratio(u128::MAX, u128::MAX),
            Decimal256::one()
        );
        // 340282366920938463463 is the largest integer <= Decimal256::MAX
        assert_eq!(
            Decimal256::from_ratio(340282366920938463463u128, 1u128),
            Decimal256::from_str("340282366920938463463").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn decimal256_from_ratio_panics_for_zero_denominator() {
        Decimal256::from_ratio(1u128, 0u128);
    }

    #[test]
    #[should_panic(expected = "Multiplication overflow")]
    fn decimal256_from_ratio_panics_for_mul_overflow() {
        Decimal256::from_ratio(Uint256::MAX, 1u128);
    }

    #[test]
    fn decimal256_checked_from_ratio_does_not_panic() {
        assert_eq!(
            Decimal256::checked_from_ratio(1u128, 0u128),
            Err(CheckedFromRatioError::DivideByZero)
        );

        assert_eq!(
            Decimal256::checked_from_ratio(Uint256::MAX, 1u128),
            Err(CheckedFromRatioError::Overflow)
        );
    }

    #[test]
    fn decimal256_implements_fraction() {
        let fraction = Decimal256::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Uint256::from_str("1234567000000000000000").unwrap()
        );
        assert_eq!(
            fraction.denominator(),
            Uint256::from_str("1000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal256_implements_from_decimal() {
        let a = Decimal::from_str("123.456").unwrap();
        let b = Decimal256::from(a);
        assert_eq!(b.to_string(), "123.456");

        let a = Decimal::from_str("0").unwrap();
        let b = Decimal256::from(a);
        assert_eq!(b.to_string(), "0");

        let a = Decimal::MAX;
        let b = Decimal256::from(a);
        assert_eq!(b.to_string(), "340282366920938463463.374607431768211455");
    }

    #[test]
    fn decimal256_from_str_works() {
        // Integers
        assert_eq!(Decimal256::from_str("0").unwrap(), Decimal256::percent(0));
        assert_eq!(Decimal256::from_str("1").unwrap(), Decimal256::percent(100));
        assert_eq!(Decimal256::from_str("5").unwrap(), Decimal256::percent(500));
        assert_eq!(
            Decimal256::from_str("42").unwrap(),
            Decimal256::percent(4200)
        );
        assert_eq!(Decimal256::from_str("000").unwrap(), Decimal256::percent(0));
        assert_eq!(
            Decimal256::from_str("001").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("005").unwrap(),
            Decimal256::percent(500)
        );
        assert_eq!(
            Decimal256::from_str("0042").unwrap(),
            Decimal256::percent(4200)
        );

        // Decimals
        assert_eq!(
            Decimal256::from_str("1.0").unwrap(),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::from_str("1.5").unwrap(),
            Decimal256::percent(150)
        );
        assert_eq!(
            Decimal256::from_str("0.5").unwrap(),
            Decimal256::percent(50)
        );
        assert_eq!(
            Decimal256::from_str("0.123").unwrap(),
            Decimal256::permille(123)
        );

        assert_eq!(
            Decimal256::from_str("40.00").unwrap(),
            Decimal256::percent(4000)
        );
        assert_eq!(
            Decimal256::from_str("04.00").unwrap(),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::from_str("00.40").unwrap(),
            Decimal256::percent(40)
        );
        assert_eq!(
            Decimal256::from_str("00.04").unwrap(),
            Decimal256::percent(4)
        );

        // Can handle 18 fractional digits
        assert_eq!(
            Decimal256::from_str("7.123456789012345678").unwrap(),
            Decimal256(Uint256::from(7123456789012345678u128))
        );
        assert_eq!(
            Decimal256::from_str("7.999999999999999999").unwrap(),
            Decimal256(Uint256::from(7999999999999999999u128))
        );

        // Works for documented max value
        assert_eq!(
            Decimal256::from_str(
                "115792089237316195423570985008687907853269984665640564039457.584007913129639935"
            )
            .unwrap(),
            Decimal256::MAX
        );
    }

    #[test]
    fn decimal256_from_str_errors_for_broken_whole_part() {
        match Decimal256::from_str("").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str(" ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str("-1").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing whole"),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_broken_fractional_part() {
        match Decimal256::from_str("1.").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str("1. ").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str("1.e").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str("1.2e3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Error parsing fractional"),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_more_than_36_fractional_digits() {
        match Decimal256::from_str("7.1234567890123456789").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {e:?}"),
        }

        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        match Decimal256::from_str("7.1230000000000000000").unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Cannot parse more than 18 fractional digits")
            }
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_invalid_number_of_dots() {
        match Decimal256::from_str("1.2.3").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {e:?}"),
        }

        match Decimal256::from_str("1.2.3.4").unwrap_err() {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Unexpected number of dots"),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn decimal256_from_str_errors_for_more_than_max_value() {
        // Integer
        match Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458")
            .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {e:?}"),
        }

        // Decimal
        match Decimal256::from_str("115792089237316195423570985008687907853269984665640564039458.0")
            .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {e:?}"),
        }
        match Decimal256::from_str(
            "115792089237316195423570985008687907853269984665640564039457.584007913129639936",
        )
        .unwrap_err()
        {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "Value too big"),
            e => panic!("Unexpected error: {e:?}"),
        }
    }

    #[test]
    fn decimal256_atomics_works() {
        let zero = Decimal256::zero();
        let one = Decimal256::one();
        let half = Decimal256::percent(50);
        let two = Decimal256::percent(200);
        let max = Decimal256::MAX;

        assert_eq!(zero.atomics(), Uint256::from(0u128));
        assert_eq!(one.atomics(), Uint256::from(1000000000000000000u128));
        assert_eq!(half.atomics(), Uint256::from(500000000000000000u128));
        assert_eq!(two.atomics(), Uint256::from(2000000000000000000u128));
        assert_eq!(max.atomics(), Uint256::MAX);
    }

    #[test]
    fn decimal256_decimal_places_works() {
        let zero = Decimal256::zero();
        let one = Decimal256::one();
        let half = Decimal256::percent(50);
        let two = Decimal256::percent(200);
        let max = Decimal256::MAX;

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
    }

    #[test]
    fn decimal256_is_zero_works() {
        assert!(Decimal256::zero().is_zero());
        assert!(Decimal256::percent(0).is_zero());
        assert!(Decimal256::permille(0).is_zero());

        assert!(!Decimal256::one().is_zero());
        assert!(!Decimal256::percent(123).is_zero());
        assert!(!Decimal256::permille(1234).is_zero());
    }

    #[test]
    fn decimal256_inv_works() {
        // d = 0
        assert_eq!(Decimal256::zero().inv(), None);

        // d == 1
        assert_eq!(Decimal256::one().inv(), Some(Decimal256::one()));

        // d > 1 exact
        assert_eq!(
            Decimal256::from_str("2").unwrap().inv(),
            Some(Decimal256::from_str("0.5").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("20").unwrap().inv(),
            Some(Decimal256::from_str("0.05").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("200").unwrap().inv(),
            Some(Decimal256::from_str("0.005").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("2000").unwrap().inv(),
            Some(Decimal256::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            Decimal256::from_str("3").unwrap().inv(),
            Some(Decimal256::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("6").unwrap().inv(),
            Some(Decimal256::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            Decimal256::from_str("0.5").unwrap().inv(),
            Some(Decimal256::from_str("2").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.05").unwrap().inv(),
            Some(Decimal256::from_str("20").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.005").unwrap().inv(),
            Some(Decimal256::from_str("200").unwrap())
        );
        assert_eq!(
            Decimal256::from_str("0.0005").unwrap().inv(),
            Some(Decimal256::from_str("2000").unwrap())
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal256_add_works() {
        let value = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(
            value.0,
            Decimal256::DECIMAL_FRACTIONAL * Uint256::from(3u8) / Uint256::from(2u8)
        );

        assert_eq!(
            Decimal256::percent(5) + Decimal256::percent(4),
            Decimal256::percent(9)
        );
        assert_eq!(
            Decimal256::percent(5) + Decimal256::zero(),
            Decimal256::percent(5)
        );
        assert_eq!(Decimal256::zero() + Decimal256::zero(), Decimal256::zero());

        // works for refs
        let a = Decimal256::percent(15);
        let b = Decimal256::percent(25);
        let expected = Decimal256::percent(40);
        assert_eq!(a + b, expected);
        assert_eq!(&a + b, expected);
        assert_eq!(a + &b, expected);
        assert_eq!(&a + &b, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to add with overflow")]
    fn decimal256_add_overflow_panics() {
        let _value = Decimal256::MAX + Decimal256::percent(50);
    }

    #[test]
    fn decimal256_add_assign_works() {
        let mut a = Decimal256::percent(30);
        a += Decimal256::percent(20);
        assert_eq!(a, Decimal256::percent(50));

        // works for refs
        let mut a = Decimal256::percent(15);
        let b = Decimal256::percent(3);
        let expected = Decimal256::percent(18);
        a += &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal256_sub_works() {
        let value = Decimal256::one() - Decimal256::percent(50); // 0.5
        assert_eq!(value.0, Decimal256::DECIMAL_FRACTIONAL / Uint256::from(2u8));

        assert_eq!(
            Decimal256::percent(9) - Decimal256::percent(4),
            Decimal256::percent(5)
        );
        assert_eq!(
            Decimal256::percent(16) - Decimal256::zero(),
            Decimal256::percent(16)
        );
        assert_eq!(
            Decimal256::percent(16) - Decimal256::percent(16),
            Decimal256::zero()
        );
        assert_eq!(Decimal256::zero() - Decimal256::zero(), Decimal256::zero());

        // works for refs
        let a = Decimal256::percent(13);
        let b = Decimal256::percent(6);
        let expected = Decimal256::percent(7);
        assert_eq!(a - b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn decimal256_sub_overflow_panics() {
        let _value = Decimal256::zero() - Decimal256::percent(50);
    }

    #[test]
    fn decimal256_sub_assign_works() {
        let mut a = Decimal256::percent(20);
        a -= Decimal256::percent(2);
        assert_eq!(a, Decimal256::percent(18));

        // works for refs
        let mut a = Decimal256::percent(33);
        let b = Decimal256::percent(13);
        let expected = Decimal256::percent(20);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal256_implements_mul() {
        let one = Decimal256::one();
        let two = one + one;
        let half = Decimal256::percent(50);

        // 1*x and x*1
        assert_eq!(one * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(one * Decimal256::percent(1), Decimal256::percent(1));
        assert_eq!(one * Decimal256::percent(10), Decimal256::percent(10));
        assert_eq!(one * Decimal256::percent(100), Decimal256::percent(100));
        assert_eq!(one * Decimal256::percent(1000), Decimal256::percent(1000));
        assert_eq!(one * Decimal256::MAX, Decimal256::MAX);
        assert_eq!(Decimal256::percent(0) * one, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * one, Decimal256::percent(1));
        assert_eq!(Decimal256::percent(10) * one, Decimal256::percent(10));
        assert_eq!(Decimal256::percent(100) * one, Decimal256::percent(100));
        assert_eq!(Decimal256::percent(1000) * one, Decimal256::percent(1000));
        assert_eq!(Decimal256::MAX * one, Decimal256::MAX);

        // double
        assert_eq!(two * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(two * Decimal256::percent(1), Decimal256::percent(2));
        assert_eq!(two * Decimal256::percent(10), Decimal256::percent(20));
        assert_eq!(two * Decimal256::percent(100), Decimal256::percent(200));
        assert_eq!(two * Decimal256::percent(1000), Decimal256::percent(2000));
        assert_eq!(Decimal256::percent(0) * two, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * two, Decimal256::percent(2));
        assert_eq!(Decimal256::percent(10) * two, Decimal256::percent(20));
        assert_eq!(Decimal256::percent(100) * two, Decimal256::percent(200));
        assert_eq!(Decimal256::percent(1000) * two, Decimal256::percent(2000));

        // half
        assert_eq!(half * Decimal256::percent(0), Decimal256::percent(0));
        assert_eq!(half * Decimal256::percent(1), Decimal256::permille(5));
        assert_eq!(half * Decimal256::percent(10), Decimal256::percent(5));
        assert_eq!(half * Decimal256::percent(100), Decimal256::percent(50));
        assert_eq!(half * Decimal256::percent(1000), Decimal256::percent(500));
        assert_eq!(Decimal256::percent(0) * half, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) * half, Decimal256::permille(5));
        assert_eq!(Decimal256::percent(10) * half, Decimal256::percent(5));
        assert_eq!(Decimal256::percent(100) * half, Decimal256::percent(50));
        assert_eq!(Decimal256::percent(1000) * half, Decimal256::percent(500));

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
        let max = Decimal256::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("115792089237316195423570985008687907853269984665640564039457.584007913129639935")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("11579208923731619542357098500868790785326998466564056403945.758400791312963993")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("1157920892373161954235709850086879078532699846656405640394.575840079131296399")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("115792089237316195423570985008687907853269984665640564039.457584007913129639")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("115792089237316195423570985008687907853269984665640564.039457584007913129")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("115792089237316195423570985008687907853269984665640.564039457584007913")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("115792089237316195423570985008687907853269984665.640564039457584007")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("115792089237316195423570985008687907853269984.665640564039457584")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("115792089237316195423570985008687907853269.984665640564039457")
        );

        // works for refs
        let a = Decimal256::percent(20);
        let b = Decimal256::percent(30);
        let expected = Decimal256::percent(6);
        assert_eq!(a * b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn decimal256_mul_assign_works() {
        let mut a = Decimal256::percent(15);
        a *= Decimal256::percent(60);
        assert_eq!(a, Decimal256::percent(9));

        // works for refs
        let mut a = Decimal256::percent(50);
        let b = Decimal256::percent(20);
        a *= &b;
        assert_eq!(a, Decimal256::percent(10));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn decimal256_mul_overflow_panics() {
        let _value = Decimal256::MAX * Decimal256::percent(101);
    }

    #[test]
    fn decimal256_checked_mul() {
        let test_data = [
            (Decimal256::zero(), Decimal256::zero()),
            (Decimal256::zero(), Decimal256::one()),
            (Decimal256::one(), Decimal256::zero()),
            (Decimal256::percent(10), Decimal256::zero()),
            (Decimal256::percent(10), Decimal256::percent(5)),
            (Decimal256::MAX, Decimal256::one()),
            (
                Decimal256::MAX / Uint256::from_uint128(2u128.into()),
                Decimal256::percent(200),
            ),
            (Decimal256::permille(6), Decimal256::permille(13)),
        ];

        // The regular core::ops::Mul is our source of truth for these tests.
        for (x, y) in test_data.into_iter() {
            assert_eq!(x * y, x.checked_mul(y).unwrap());
        }
    }

    #[test]
    fn decimal256_checked_mul_overflow() {
        assert_eq!(
            Decimal256::MAX.checked_mul(Decimal256::percent(200)),
            Err(OverflowError {
                operation: OverflowOperation::Mul,
                operand1: Decimal256::MAX.to_string(),
                operand2: Decimal256::percent(200).to_string(),
            })
        );
    }

    #[test]
    // in this test the Decimal256 is on the right
    fn uint128_decimal_multiply() {
        // a*b
        let left = Uint256::from(300u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(450u32));

        // a*0
        let left = Uint256::from(300u128);
        let right = Decimal256::zero();
        assert_eq!(left * right, Uint256::from(0u128));

        // 0*a
        let left = Uint256::from(0u128);
        let right = Decimal256::one() + Decimal256::percent(50); // 1.5
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    // in this test the Decimal256 is on the left
    fn decimal256_uint128_multiply() {
        // a*b
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(450u128));

        // 0*a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left * right, Uint256::from(0u128));

        // a*0
        let left = Decimal256::one() + Decimal256::percent(50); // 1.5
        let right = Uint256::from(0u128);
        assert_eq!(left * right, Uint256::from(0u128));
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal256_implements_div() {
        let one = Decimal256::one();
        let two = one + one;
        let half = Decimal256::percent(50);

        // 1/x and x/1
        assert_eq!(one / Decimal256::percent(1), Decimal256::percent(10_000));
        assert_eq!(one / Decimal256::percent(10), Decimal256::percent(1_000));
        assert_eq!(one / Decimal256::percent(100), Decimal256::percent(100));
        assert_eq!(one / Decimal256::percent(1000), Decimal256::percent(10));
        assert_eq!(Decimal256::percent(0) / one, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) / one, Decimal256::percent(1));
        assert_eq!(Decimal256::percent(10) / one, Decimal256::percent(10));
        assert_eq!(Decimal256::percent(100) / one, Decimal256::percent(100));
        assert_eq!(Decimal256::percent(1000) / one, Decimal256::percent(1000));

        // double
        assert_eq!(two / Decimal256::percent(1), Decimal256::percent(20_000));
        assert_eq!(two / Decimal256::percent(10), Decimal256::percent(2_000));
        assert_eq!(two / Decimal256::percent(100), Decimal256::percent(200));
        assert_eq!(two / Decimal256::percent(1000), Decimal256::percent(20));
        assert_eq!(Decimal256::percent(0) / two, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) / two, dec("0.005"));
        assert_eq!(Decimal256::percent(10) / two, Decimal256::percent(5));
        assert_eq!(Decimal256::percent(100) / two, Decimal256::percent(50));
        assert_eq!(Decimal256::percent(1000) / two, Decimal256::percent(500));

        // half
        assert_eq!(half / Decimal256::percent(1), Decimal256::percent(5_000));
        assert_eq!(half / Decimal256::percent(10), Decimal256::percent(500));
        assert_eq!(half / Decimal256::percent(100), Decimal256::percent(50));
        assert_eq!(half / Decimal256::percent(1000), Decimal256::percent(5));
        assert_eq!(Decimal256::percent(0) / half, Decimal256::percent(0));
        assert_eq!(Decimal256::percent(1) / half, Decimal256::percent(2));
        assert_eq!(Decimal256::percent(10) / half, Decimal256::percent(20));
        assert_eq!(Decimal256::percent(100) / half, Decimal256::percent(200));
        assert_eq!(Decimal256::percent(1000) / half, Decimal256::percent(2000));

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
            Decimal256::percent(15) / Decimal256::percent(60),
            Decimal256::percent(25)
        );

        // works for refs
        let a = Decimal256::percent(100);
        let b = Decimal256::percent(20);
        let expected = Decimal256::percent(500);
        assert_eq!(a / b, expected);
        assert_eq!(&a / b, expected);
        assert_eq!(a / &b, expected);
        assert_eq!(&a / &b, expected);
    }

    #[test]
    fn decimal256_div_assign_works() {
        let mut a = Decimal256::percent(15);
        a /= Decimal256::percent(20);
        assert_eq!(a, Decimal256::percent(75));

        // works for refs
        let mut a = Decimal256::percent(50);
        let b = Decimal256::percent(20);
        a /= &b;
        assert_eq!(a, Decimal256::percent(250));
    }

    #[test]
    #[should_panic(expected = "Division failed - multiplication overflow")]
    fn decimal256_div_overflow_panics() {
        let _value = Decimal256::MAX / Decimal256::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn decimal256_div_by_zero_panics() {
        let _value = Decimal256::one() / Decimal256::zero();
    }

    #[test]
    fn decimal256_uint128_division() {
        // a/b
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(3u128);
        assert_eq!(left / right, Decimal256::percent(50));

        // 0/a
        let left = Decimal256::zero();
        let right = Uint256::from(300u128);
        assert_eq!(left / right, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal256_uint128_divide_by_zero() {
        let left = Decimal256::percent(150); // 1.5
        let right = Uint256::from(0u128);
        let _result = left / right;
    }

    #[test]
    fn decimal256_uint128_div_assign() {
        // a/b
        let mut dec = Decimal256::percent(150); // 1.5
        dec /= Uint256::from(3u128);
        assert_eq!(dec, Decimal256::percent(50));

        // 0/a
        let mut dec = Decimal256::zero();
        dec /= Uint256::from(300u128);
        assert_eq!(dec, Decimal256::zero());
    }

    #[test]
    #[should_panic(expected = "attempt to divide by zero")]
    fn decimal256_uint128_div_assign_by_zero() {
        // a/0
        let mut dec = Decimal256::percent(50);
        dec /= Uint256::from(0u128);
    }

    #[test]
    fn decimal256_uint128_sqrt() {
        assert_eq!(Decimal256::percent(900).sqrt(), Decimal256::percent(300));

        assert!(Decimal256::percent(316) < Decimal256::percent(1000).sqrt());
        assert!(Decimal256::percent(1000).sqrt() < Decimal256::percent(317));
    }

    /// sqrt(2) is an irrational number, i.e. all 36 decimal places should be used.
    #[test]
    fn decimal256_uint128_sqrt_is_precise() {
        assert_eq!(
            Decimal256::from_str("2").unwrap().sqrt(),
            Decimal256::from_str("1.414213562373095048").unwrap() // https://www.wolframalpha.com/input/?i=sqrt%282%29
        );
    }

    #[test]
    fn decimal256_uint128_sqrt_does_not_overflow() {
        assert_eq!(
            Decimal256::from_str("40000000000000000000000000000000000000000000000000000000000")
                .unwrap()
                .sqrt(),
            Decimal256::from_str("200000000000000000000000000000").unwrap()
        );
    }

    #[test]
    fn decimal256_uint128_sqrt_intermediate_precision_used() {
        assert_eq!(
            Decimal256::from_str("40000000000000000000000000000000000000000000000001")
                .unwrap()
                .sqrt(),
            // The last few digits (39110) are truncated below due to the algorithm
            // we use. Larger numbers will cause less precision.
            // https://www.wolframalpha.com/input/?i=sqrt%2840000000000000000000000000000000000000000000000001%29
            Decimal256::from_str("6324555320336758663997787.088865437067400000").unwrap()
        );
    }

    #[test]
    fn decimal256_checked_pow() {
        for exp in 0..10 {
            assert_eq!(
                Decimal256::one().checked_pow(exp).unwrap(),
                Decimal256::one()
            );
        }

        // This case is mathematically undefined but we ensure consistency with Rust standard types
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
        assert_eq!(
            Decimal256::zero().checked_pow(0).unwrap(),
            Decimal256::one()
        );

        for exp in 1..10 {
            assert_eq!(
                Decimal256::zero().checked_pow(exp).unwrap(),
                Decimal256::zero()
            );
        }

        for num in &[
            Decimal256::percent(50),
            Decimal256::percent(99),
            Decimal256::percent(200),
        ] {
            assert_eq!(num.checked_pow(0).unwrap(), Decimal256::one())
        }

        assert_eq!(
            Decimal256::percent(20).checked_pow(2).unwrap(),
            Decimal256::percent(4)
        );

        assert_eq!(
            Decimal256::percent(20).checked_pow(3).unwrap(),
            Decimal256::permille(8)
        );

        assert_eq!(
            Decimal256::percent(200).checked_pow(4).unwrap(),
            Decimal256::percent(1600)
        );

        assert_eq!(
            Decimal256::percent(200).checked_pow(4).unwrap(),
            Decimal256::percent(1600)
        );

        assert_eq!(
            Decimal256::percent(700).checked_pow(5).unwrap(),
            Decimal256::percent(1680700)
        );

        assert_eq!(
            Decimal256::percent(700).checked_pow(8).unwrap(),
            Decimal256::percent(576480100)
        );

        assert_eq!(
            Decimal256::percent(700).checked_pow(10).unwrap(),
            Decimal256::percent(28247524900)
        );

        assert_eq!(
            Decimal256::percent(120).checked_pow(123).unwrap(),
            Decimal256(5486473221892422150877397607u128.into())
        );

        assert_eq!(
            Decimal256::percent(10).checked_pow(2).unwrap(),
            Decimal256(10000000000000000u128.into())
        );

        assert_eq!(
            Decimal256::percent(10).checked_pow(18).unwrap(),
            Decimal256(1u128.into())
        );
    }

    #[test]
    fn decimal256_checked_pow_overflow() {
        assert_eq!(
            Decimal256::MAX.checked_pow(2),
            Err(OverflowError {
                operation: OverflowOperation::Pow,
                operand1: Decimal256::MAX.to_string(),
                operand2: "2".to_string(),
            })
        );
    }

    #[test]
    fn decimal256_to_string() {
        // Integers
        assert_eq!(Decimal256::zero().to_string(), "0");
        assert_eq!(Decimal256::one().to_string(), "1");
        assert_eq!(Decimal256::percent(500).to_string(), "5");

        // Decimals
        assert_eq!(Decimal256::percent(125).to_string(), "1.25");
        assert_eq!(Decimal256::percent(42638).to_string(), "426.38");
        assert_eq!(Decimal256::percent(3).to_string(), "0.03");
        assert_eq!(Decimal256::permille(987).to_string(), "0.987");

        assert_eq!(
            Decimal256(Uint256::from(1u128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10u128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100u128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000u128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000u128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000u128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000u128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000u128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000u128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000u128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000u128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000u128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000u128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000u128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            Decimal256(Uint256::from(1000000000000000u128)).to_string(),
            "0.001"
        );
        assert_eq!(
            Decimal256(Uint256::from(10000000000000000u128)).to_string(),
            "0.01"
        );
        assert_eq!(
            Decimal256(Uint256::from(100000000000000000u128)).to_string(),
            "0.1"
        );
    }

    #[test]
    fn decimal256_iter_sum() {
        let items = vec![
            Decimal256::zero(),
            Decimal256::from_str("2").unwrap(),
            Decimal256::from_str("2").unwrap(),
        ];
        assert_eq!(
            items.iter().sum::<Decimal256>(),
            Decimal256::from_str("4").unwrap()
        );
        assert_eq!(
            items.into_iter().sum::<Decimal256>(),
            Decimal256::from_str("4").unwrap()
        );

        let empty: Vec<Decimal256> = vec![];
        assert_eq!(Decimal256::zero(), empty.iter().sum::<Decimal256>());
    }

    #[test]
    fn decimal256_serialize() {
        assert_eq!(to_json_vec(&Decimal256::zero()).unwrap(), br#""0""#);
        assert_eq!(to_json_vec(&Decimal256::one()).unwrap(), br#""1""#);
        assert_eq!(to_json_vec(&Decimal256::percent(8)).unwrap(), br#""0.08""#);
        assert_eq!(to_json_vec(&Decimal256::percent(87)).unwrap(), br#""0.87""#);
        assert_eq!(
            to_json_vec(&Decimal256::percent(876)).unwrap(),
            br#""8.76""#
        );
        assert_eq!(
            to_json_vec(&Decimal256::percent(8765)).unwrap(),
            br#""87.65""#
        );
    }

    #[test]
    fn decimal256_deserialize() {
        assert_eq!(
            from_json::<Decimal256>(br#""0""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_json::<Decimal256>(br#""1""#).unwrap(),
            Decimal256::one()
        );
        assert_eq!(
            from_json::<Decimal256>(br#""000""#).unwrap(),
            Decimal256::zero()
        );
        assert_eq!(
            from_json::<Decimal256>(br#""001""#).unwrap(),
            Decimal256::one()
        );

        assert_eq!(
            from_json::<Decimal256>(br#""0.08""#).unwrap(),
            Decimal256::percent(8)
        );
        assert_eq!(
            from_json::<Decimal256>(br#""0.87""#).unwrap(),
            Decimal256::percent(87)
        );
        assert_eq!(
            from_json::<Decimal256>(br#""8.76""#).unwrap(),
            Decimal256::percent(876)
        );
        assert_eq!(
            from_json::<Decimal256>(br#""87.65""#).unwrap(),
            Decimal256::percent(8765)
        );
    }

    #[test]
    fn decimal256_abs_diff_works() {
        let a = Decimal256::percent(285);
        let b = Decimal256::percent(200);
        let expected = Decimal256::percent(85);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn decimal256_rem_works() {
        // 4.02 % 1.11 = 0.69
        assert_eq!(
            Decimal256::percent(402) % Decimal256::percent(111),
            Decimal256::percent(69)
        );

        // 15.25 % 4 = 3.25
        assert_eq!(
            Decimal256::percent(1525) % Decimal256::percent(400),
            Decimal256::percent(325)
        );

        let a = Decimal256::percent(318);
        let b = Decimal256::percent(317);
        let expected = Decimal256::percent(1);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn decimal_rem_assign_works() {
        let mut a = Decimal256::percent(17673);
        a %= Decimal256::percent(2362);
        assert_eq!(a, Decimal256::percent(1139)); // 176.73 % 23.62 = 11.39

        let mut a = Decimal256::percent(4262);
        let b = Decimal256::percent(1270);
        a %= &b;
        assert_eq!(a, Decimal256::percent(452)); // 42.62 % 12.7 = 4.52
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn decimal256_rem_panics_for_zero() {
        let _ = Decimal256::percent(777) % Decimal256::zero();
    }

    #[test]
    fn decimal256_checked_methods() {
        // checked add
        assert_eq!(
            Decimal256::percent(402)
                .checked_add(Decimal256::percent(111))
                .unwrap(),
            Decimal256::percent(513)
        );
        assert!(matches!(
            Decimal256::MAX.checked_add(Decimal256::percent(1)),
            Err(OverflowError { .. })
        ));

        // checked sub
        assert_eq!(
            Decimal256::percent(1111)
                .checked_sub(Decimal256::percent(111))
                .unwrap(),
            Decimal256::percent(1000)
        );
        assert!(matches!(
            Decimal256::zero().checked_sub(Decimal256::percent(1)),
            Err(OverflowError { .. })
        ));

        // checked div
        assert_eq!(
            Decimal256::percent(30)
                .checked_div(Decimal256::percent(200))
                .unwrap(),
            Decimal256::percent(15)
        );
        assert_eq!(
            Decimal256::percent(88)
                .checked_div(Decimal256::percent(20))
                .unwrap(),
            Decimal256::percent(440)
        );
        assert!(matches!(
            Decimal256::MAX.checked_div(Decimal256::zero()),
            Err(CheckedFromRatioError::DivideByZero { .. })
        ));
        assert!(matches!(
            Decimal256::MAX.checked_div(Decimal256::percent(1)),
            Err(CheckedFromRatioError::Overflow { .. })
        ));

        // checked rem
        assert_eq!(
            Decimal256::percent(402)
                .checked_rem(Decimal256::percent(111))
                .unwrap(),
            Decimal256::percent(69)
        );
        assert_eq!(
            Decimal256::percent(1525)
                .checked_rem(Decimal256::percent(400))
                .unwrap(),
            Decimal256::percent(325)
        );
        assert!(matches!(
            Decimal256::MAX.checked_rem(Decimal256::zero()),
            Err(DivideByZeroError { .. })
        ));
    }

    #[test]
    fn decimal256_pow_works() {
        assert_eq!(Decimal256::percent(200).pow(2), Decimal256::percent(400));
        assert_eq!(
            Decimal256::percent(200).pow(10),
            Decimal256::percent(102400)
        );
    }

    #[test]
    #[should_panic]
    fn decimal256_pow_overflow_panics() {
        _ = Decimal256::MAX.pow(2u32);
    }

    #[test]
    fn decimal256_saturating_works() {
        assert_eq!(
            Decimal256::percent(200).saturating_add(Decimal256::percent(200)),
            Decimal256::percent(400)
        );
        assert_eq!(
            Decimal256::MAX.saturating_add(Decimal256::percent(200)),
            Decimal256::MAX
        );
        assert_eq!(
            Decimal256::percent(200).saturating_sub(Decimal256::percent(100)),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::zero().saturating_sub(Decimal256::percent(200)),
            Decimal256::zero()
        );
        assert_eq!(
            Decimal256::percent(200).saturating_mul(Decimal256::percent(50)),
            Decimal256::percent(100)
        );
        assert_eq!(
            Decimal256::MAX.saturating_mul(Decimal256::percent(200)),
            Decimal256::MAX
        );
        assert_eq!(
            Decimal256::percent(400).saturating_pow(2u32),
            Decimal256::percent(1600)
        );
        assert_eq!(Decimal256::MAX.saturating_pow(2u32), Decimal256::MAX);
    }

    #[test]
    fn decimal256_rounding() {
        assert_eq!(Decimal256::one().floor(), Decimal256::one());
        assert_eq!(Decimal256::percent(150).floor(), Decimal256::one());
        assert_eq!(Decimal256::percent(199).floor(), Decimal256::one());
        assert_eq!(Decimal256::percent(200).floor(), Decimal256::percent(200));
        assert_eq!(Decimal256::percent(99).floor(), Decimal256::zero());

        assert_eq!(Decimal256::one().ceil(), Decimal256::one());
        assert_eq!(Decimal256::percent(150).ceil(), Decimal256::percent(200));
        assert_eq!(Decimal256::percent(199).ceil(), Decimal256::percent(200));
        assert_eq!(Decimal256::percent(99).ceil(), Decimal256::one());
        assert_eq!(Decimal256(Uint256::from(1u128)).ceil(), Decimal256::one());
    }

    #[test]
    #[should_panic(expected = "attempt to ceil with overflow")]
    fn decimal256_ceil_panics() {
        let _ = Decimal256::MAX.ceil();
    }

    #[test]
    fn decimal256_checked_ceil() {
        assert_eq!(
            Decimal256::percent(199).checked_ceil(),
            Ok(Decimal256::percent(200))
        );
        assert_eq!(Decimal256::MAX.checked_ceil(), Err(RoundUpOverflowError));
    }

    #[test]
    fn decimal256_to_uint_floor_works() {
        let d = Decimal256::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(12));
        let d = Decimal256::from_str("12.345").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(12));
        let d = Decimal256::from_str("12.999").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(12));
        let d = Decimal256::from_str("0.98451384").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(0));

        let d = Decimal256::from_str("75.0").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(75));
        let d = Decimal256::from_str("0.0").unwrap();
        assert_eq!(d.to_uint_floor(), Uint256::from_u128(0));

        let d = Decimal256::MAX;
        assert_eq!(
            d.to_uint_floor(),
            Uint256::from_str("115792089237316195423570985008687907853269984665640564039457")
                .unwrap()
        );

        // Does the same as the old workaround `Uint256::one() * my_decimal`.
        // This block can be deleted as part of https://github.com/CosmWasm/cosmwasm/issues/1485.
        let tests = vec![
            Decimal256::from_str("12.345").unwrap(),
            Decimal256::from_str("0.98451384").unwrap(),
            Decimal256::from_str("178.0").unwrap(),
            Decimal256::MIN,
            Decimal256::MAX,
        ];
        for my_decimal in tests.into_iter() {
            assert_eq!(my_decimal.to_uint_floor(), Uint256::one() * my_decimal);
        }
    }

    #[test]
    fn decimal256_to_uint_ceil_works() {
        let d = Decimal256::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint256::from_u128(13));
        let d = Decimal256::from_str("12.345").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint256::from_u128(13));
        let d = Decimal256::from_str("12.999").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint256::from_u128(13));

        let d = Decimal256::from_str("75.0").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint256::from_u128(75));
        let d = Decimal256::from_str("0.0").unwrap();
        assert_eq!(d.to_uint_ceil(), Uint256::from_u128(0));

        let d = Decimal256::MAX;
        assert_eq!(
            d.to_uint_ceil(),
            Uint256::from_str("115792089237316195423570985008687907853269984665640564039458")
                .unwrap()
        );
    }

    #[test]
    fn decimal256_partial_eq() {
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
    fn decimal256_implements_debug() {
        let decimal = Decimal256::from_str("123.45").unwrap();
        assert_eq!(format!("{decimal:?}"), "Decimal256(123.45)");

        let test_cases = ["5", "5.01", "42", "0", "2"];
        for s in test_cases {
            let decimal256 = Decimal256::from_str(s).unwrap();
            let expected = format!("Decimal256({s})");
            assert_eq!(format!("{decimal256:?}"), expected);
        }
    }
}
