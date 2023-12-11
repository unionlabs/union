use core::cmp::Ordering;
use core::fmt::{self, Write};
use core::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Neg, Rem, RemAssign, Sub, SubAssign,
};
use core::str::FromStr;
use forward_ref::{forward_ref_binop, forward_ref_op_assign};
use schemars::JsonSchema;
use serde::{de, ser, Deserialize, Deserializer, Serialize};
use thiserror::Error;

use crate::errors::{
    CheckedFromRatioError, CheckedMultiplyRatioError, DivideByZeroError, OverflowError,
    OverflowOperation, RoundDownOverflowError, RoundUpOverflowError, StdError,
};
use crate::{forward_ref_partial_eq, Decimal, Decimal256, Int512, SignedDecimal};

use super::Fraction;
use super::Int256;

/// A signed fixed-point decimal value with 18 fractional digits,
/// i.e. SignedDecimal256(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is
/// 57896044618658097711785492504343953926634992332820282019728.792003956564819967
/// (which is (2^255 - 1) / 10^18)
/// and the smallest is
/// -57896044618658097711785492504343953926634992332820282019728.792003956564819968
/// (which is -2^255 / 10^18).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct SignedDecimal256(#[schemars(with = "String")] Int256);

forward_ref_partial_eq!(SignedDecimal256, SignedDecimal256);

#[derive(Error, Debug, PartialEq, Eq)]
#[error("SignedDecimal256 range exceeded")]
pub struct SignedDecimal256RangeExceeded;

impl SignedDecimal256 {
    const DECIMAL_FRACTIONAL: Int256 = // 1*10**18
        Int256::from_i128(1_000_000_000_000_000_000);
    const DECIMAL_FRACTIONAL_SQUARED: Int256 = // 1*10**36
        Int256::from_i128(1_000_000_000_000_000_000_000_000_000_000_000_000);

    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18; // This needs to be an even number.

    /// The largest value that can be represented by this signed decimal type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// assert_eq!(
    ///     SignedDecimal256::MAX.to_string(),
    ///     "57896044618658097711785492504343953926634992332820282019728.792003956564819967"
    /// );
    /// ```
    pub const MAX: Self = Self(Int256::MAX);

    /// The smallest value that can be represented by this signed decimal type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// assert_eq!(
    ///     SignedDecimal256::MIN.to_string(),
    ///     "-57896044618658097711785492504343953926634992332820282019728.792003956564819968"
    /// );
    /// ```
    pub const MIN: Self = Self(Int256::MIN);

    /// Creates a SignedDecimal256(value)
    /// This is equivalent to `SignedDecimal256::from_atomics(value, 18)` but usable in a const context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal256, Int256};
    /// assert_eq!(SignedDecimal256::new(Int256::one()).to_string(), "0.000000000000000001");
    /// ```
    pub const fn new(value: Int256) -> Self {
        Self(value)
    }

    /// Creates a SignedDecimal256(Int256(value))
    /// This is equivalent to `SignedDecimal256::from_atomics(value, 18)` but usable in a const context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// assert_eq!(SignedDecimal256::raw(1234i128).to_string(), "0.000000000000001234");
    /// ```
    pub const fn raw(value: i128) -> Self {
        Self(Int256::from_i128(value))
    }

    /// Create a 1.0 SignedDecimal256
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a -1.0 SignedDecimal256
    #[inline]
    pub const fn negative_one() -> Self {
        // -DECIMAL_FRATIONAL
        Self(Int256::from_i128(-1_000_000_000_000_000_000))
    }

    /// Create a 0.0 SignedDecimal256
    #[inline]
    pub const fn zero() -> Self {
        Self(Int256::zero())
    }

    /// Convert x% into SignedDecimal256
    pub fn percent(x: i64) -> Self {
        Self(((x as i128) * 10_000_000_000_000_000).into())
    }

    /// Convert permille (x/1000) into SignedDecimal256
    pub fn permille(x: i64) -> Self {
        Self(((x as i128) * 1_000_000_000_000_000).into())
    }

    /// Convert basis points (x/10000) into SignedDecimal256
    pub fn bps(x: i64) -> Self {
        Self(((x as i128) * 100_000_000_000_000).into())
    }

    /// Creates a signed decimal from a number of atomic units and the number
    /// of decimal places. The inputs will be converted internally to form
    /// a signed decimal with 18 decimal places. So the input 123 and 2 will create
    /// the decimal 1.23.
    ///
    /// Using 18 decimal places is slightly more efficient than other values
    /// as no internal conversion is necessary.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal256, Int256};
    /// let a = SignedDecimal256::from_atomics(Int256::from(1234), 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = SignedDecimal256::from_atomics(1234i128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = SignedDecimal256::from_atomics(1i64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    ///
    /// let a = SignedDecimal256::from_atomics(-1i64, 18).unwrap();
    /// assert_eq!(a.to_string(), "-0.000000000000000001");
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Int256>,
        decimal_places: u32,
    ) -> Result<Self, SignedDecimal256RangeExceeded> {
        let atomics = atomics.into();
        let ten = Int256::from(10u64);
        Ok(match decimal_places.cmp(&(Self::DECIMAL_PLACES)) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = ten.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| SignedDecimal256RangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = ten.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Int256 range.
                    // Any Int256 `x` divided by `factor` with `factor > Int256::MAX` is 0.
                    // Try e.g. Python3: `(2**128-1) // 2**128`
                    Self(Int256::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a SignedDecimal256
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// assert_eq!(
    ///     SignedDecimal256::from_ratio(1, 3).to_string(),
    ///     "0.333333333333333333"
    /// );
    /// ```
    pub fn from_ratio(numerator: impl Into<Int256>, denominator: impl Into<Int256>) -> Self {
        match SignedDecimal256::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a SignedDecimal256
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal256, CheckedFromRatioError};
    /// assert_eq!(
    ///     SignedDecimal256::checked_from_ratio(1, 3).unwrap().to_string(),
    ///     "0.333333333333333333"
    /// );
    /// assert_eq!(
    ///     SignedDecimal256::checked_from_ratio(1, 0),
    ///     Err(CheckedFromRatioError::DivideByZero)
    /// );
    /// ```
    pub fn checked_from_ratio(
        numerator: impl Into<Int256>,
        denominator: impl Into<Int256>,
    ) -> Result<Self, CheckedFromRatioError> {
        let numerator: Int256 = numerator.into();
        let denominator: Int256 = denominator.into();
        match numerator.checked_multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator) {
            Ok(ratio) => {
                // numerator * DECIMAL_FRACTIONAL / denominator
                Ok(SignedDecimal256(ratio))
            }
            Err(CheckedMultiplyRatioError::Overflow) => Err(CheckedFromRatioError::Overflow),
            Err(CheckedMultiplyRatioError::DivideByZero) => {
                Err(CheckedFromRatioError::DivideByZero)
            }
        }
    }

    /// Returns `true` if the number is 0
    #[must_use]
    pub const fn is_zero(&self) -> bool {
        self.0.is_zero()
    }

    /// Returns `true` if the number is negative (< 0)
    #[must_use]
    pub const fn is_negative(&self) -> bool {
        self.0.is_negative()
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal256, Int256};
    /// # use core::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = SignedDecimal256::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Int256::from(1234000000000000000i128));
    ///
    /// // Smallest possible value
    /// let b = SignedDecimal256::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Int256::from(1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> Int256 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`SignedDecimal256::atomics()`].
    #[must_use]
    #[inline]
    pub const fn decimal_places(&self) -> u32 {
        Self::DECIMAL_PLACES
    }

    /// Rounds value by truncating the decimal places.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// # use core::str::FromStr;
    /// assert!(SignedDecimal256::from_str("0.6").unwrap().trunc().is_zero());
    /// assert_eq!(SignedDecimal256::from_str("-5.8").unwrap().trunc().to_string(), "-5");
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn trunc(&self) -> Self {
        Self((self.0 / Self::DECIMAL_FRACTIONAL) * Self::DECIMAL_FRACTIONAL)
    }

    /// Rounds value down after decimal places. Panics on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// # use core::str::FromStr;
    /// assert!(SignedDecimal256::from_str("0.6").unwrap().floor().is_zero());
    /// assert_eq!(SignedDecimal256::from_str("-5.2").unwrap().floor().to_string(), "-6");
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn floor(&self) -> Self {
        match self.checked_floor() {
            Ok(value) => value,
            Err(_) => panic!("attempt to floor with overflow"),
        }
    }

    /// Rounds value down after decimal places.
    pub fn checked_floor(&self) -> Result<Self, RoundDownOverflowError> {
        if self.is_negative() {
            let truncated = self.trunc();

            if truncated != self {
                truncated
                    .checked_sub(SignedDecimal256::one())
                    .map_err(|_| RoundDownOverflowError)
            } else {
                Ok(truncated)
            }
        } else {
            Ok(self.trunc())
        }
    }

    /// Rounds value up after decimal places. Panics on overflow.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal256;
    /// # use core::str::FromStr;
    /// assert_eq!(SignedDecimal256::from_str("0.2").unwrap().ceil(), SignedDecimal256::one());
    /// assert_eq!(SignedDecimal256::from_str("-5.8").unwrap().ceil().to_string(), "-5");
    /// ```
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
                .checked_add(SignedDecimal256::one())
                .map_err(|_| RoundUpOverflowError)
        }
    }

    /// Computes `self + other`, returning an `OverflowError` if an overflow occurred.
    pub fn checked_add(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_add(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Add, self, other))
    }

    /// Computes `self - other`, returning an `OverflowError` if an overflow occurred.
    pub fn checked_sub(self, other: Self) -> Result<Self, OverflowError> {
        self.0
            .checked_sub(other.0)
            .map(Self)
            .map_err(|_| OverflowError::new(OverflowOperation::Sub, self, other))
    }

    /// Multiplies one `SignedDecimal256` by another, returning an `OverflowError` if an overflow occurred.
    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        let result_as_int512 =
            self.numerator().full_mul(other.numerator()) / Int512::from(Self::DECIMAL_FRACTIONAL);
        result_as_int512
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

        fn inner(mut x: SignedDecimal256, mut n: u32) -> Result<SignedDecimal256, OverflowError> {
            if n == 0 {
                return Ok(SignedDecimal256::one());
            }

            let mut y = SignedDecimal256::one();

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
        SignedDecimal256::checked_from_ratio(self.numerator(), other.numerator())
    }

    /// Computes `self % other`, returning an `DivideByZeroError` if `other == 0`.
    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .map_err(|_| DivideByZeroError::new(self))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs_diff(self, other: Self) -> Decimal256 {
        Decimal256::new(self.0.abs_diff(other.0))
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
        match self.checked_mul(other) {
            Ok(value) => value,
            Err(_) => {
                // both negative or both positive results in positive number, otherwise negative
                if self.is_negative() == other.is_negative() {
                    Self::MAX
                } else {
                    Self::MIN
                }
            }
        }
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn saturating_pow(self, exp: u32) -> Self {
        match self.checked_pow(exp) {
            Ok(value) => value,
            Err(_) => {
                // odd exponent of negative number results in negative number
                // everything else results in positive number
                if self.is_negative() && exp % 2 == 1 {
                    Self::MIN
                } else {
                    Self::MAX
                }
            }
        }
    }

    /// Converts this decimal to a signed integer by rounding down
    /// to the next integer, e.g. 22.5 becomes 22 and -1.2 becomes -2.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{SignedDecimal256, Int256};
    ///
    /// let d = SignedDecimal256::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_floor(), Int256::from(12));
    ///
    /// let d = SignedDecimal256::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_floor(), Int256::from(-13));
    ///
    /// let d = SignedDecimal256::from_str("-0.05").unwrap();
    /// assert_eq!(d.to_int_floor(), Int256::from(-1));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_floor(self) -> Int256 {
        if self.is_negative() {
            // Using `x.to_int_floor() = -(-x).to_int_ceil()` for a negative `x`,
            // but avoiding overflow by implementing the formula from `to_int_ceil` directly.
            let x = self.0;
            let y = Self::DECIMAL_FRACTIONAL;
            // making sure not to negate `x`, as this would overflow
            -Int256::one() - ((-Int256::one() - x) / y)
        } else {
            self.to_int_trunc()
        }
    }

    /// Converts this decimal to a signed integer by truncating
    /// the fractional part, e.g. 22.5 becomes 22.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{SignedDecimal256, Int256};
    ///
    /// let d = SignedDecimal256::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int256::from(12));
    ///
    /// let d = SignedDecimal256::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int256::from(-12));
    ///
    /// let d = SignedDecimal256::from_str("75.0").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int256::from(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_trunc(self) -> Int256 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Converts this decimal to a signed integer by rounding up
    /// to the next integer, e.g. 22.3 becomes 23 and -1.2 becomes -1.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{SignedDecimal256, Int256};
    ///
    /// let d = SignedDecimal256::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int256::from(13));
    ///
    /// let d = SignedDecimal256::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int256::from(-12));
    ///
    /// let d = SignedDecimal256::from_str("75.0").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int256::from(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_ceil(self) -> Int256 {
        if self.is_negative() {
            self.to_int_trunc()
        } else {
            // Using `q = 1 + ((x - 1) / y); // if x != 0` with unsigned integers x, y, q
            // from https://stackoverflow.com/a/2745086/2013738. We know `x + y` CAN overflow.
            let x = self.0;
            let y = Self::DECIMAL_FRACTIONAL;
            if x.is_zero() {
                Int256::zero()
            } else {
                Int256::one() + ((x - Int256::one()) / y)
            }
        }
    }
}

impl Fraction<Int256> for SignedDecimal256 {
    #[inline]
    fn numerator(&self) -> Int256 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Int256 {
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
            Some(SignedDecimal256(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl Neg for SignedDecimal256 {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl From<SignedDecimal> for SignedDecimal256 {
    fn from(value: SignedDecimal) -> Self {
        Self::new(value.atomics().into())
    }
}

impl From<Decimal> for SignedDecimal256 {
    fn from(value: Decimal) -> Self {
        Self::new(value.atomics().into())
    }
}

impl TryFrom<Decimal256> for SignedDecimal256 {
    type Error = SignedDecimal256RangeExceeded;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(SignedDecimal256)
            .map_err(|_| SignedDecimal256RangeExceeded)
    }
}

impl FromStr for SignedDecimal256 {
    type Err = StdError;

    /// Converts the decimal string to a SignedDecimal256
    /// Possible inputs: "1.23", "1", "000012", "1.123000000", "-1.12300"
    /// Disallowed: "", ".23"
    ///
    /// This never performs any kind of rounding.
    /// More than DECIMAL_PLACES fractional digits, even zeros, result in an error.
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parts_iter = input.split('.');

        let whole_part = parts_iter.next().unwrap(); // split always returns at least one element
        let is_neg = whole_part.starts_with('-');

        let whole = whole_part
            .parse::<Int256>()
            .map_err(|_| StdError::generic_err("Error parsing whole"))?;
        let mut atomics = whole
            .checked_mul(Self::DECIMAL_FRACTIONAL)
            .map_err(|_| StdError::generic_err("Value too big"))?;

        if let Some(fractional_part) = parts_iter.next() {
            let fractional = fractional_part
                .parse::<u64>() // u64 is enough for 18 decimal places
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
            let fractional_factor = Int256::from(10i128.pow(exp));

            // This multiplication can't overflow because
            // fractional < 10^DECIMAL_PLACES && fractional_factor <= 10^DECIMAL_PLACES
            let fractional_part = Int256::from(fractional)
                .checked_mul(fractional_factor)
                .unwrap();

            // for negative numbers, we need to subtract the fractional part
            atomics = if is_neg {
                atomics.checked_sub(fractional_part)
            } else {
                atomics.checked_add(fractional_part)
            }
            .map_err(|_| StdError::generic_err("Value too big"))?;
        }

        if parts_iter.next().is_some() {
            return Err(StdError::generic_err("Unexpected number of dots"));
        }

        Ok(SignedDecimal256(atomics))
    }
}

impl fmt::Display for SignedDecimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let whole = (self.0) / Self::DECIMAL_FRACTIONAL;
        let fractional = (self.0).checked_rem(Self::DECIMAL_FRACTIONAL).unwrap();

        if fractional.is_zero() {
            write!(f, "{whole}")
        } else {
            let fractional_string = format!(
                "{:0>padding$}",
                fractional.abs(), // fractional should always be printed as positive
                padding = Self::DECIMAL_PLACES as usize
            );
            if self.is_negative() {
                f.write_char('-')?;
            }
            write!(
                f,
                "{whole}.{fractional}",
                whole = whole.abs(),
                fractional = fractional_string.trim_end_matches('0')
            )
        }
    }
}

impl fmt::Debug for SignedDecimal256 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedDecimal256({self})")
    }
}

impl Add for SignedDecimal256 {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SignedDecimal256(self.0 + other.0)
    }
}
forward_ref_binop!(impl Add, add for SignedDecimal256, SignedDecimal256);

impl AddAssign for SignedDecimal256 {
    fn add_assign(&mut self, rhs: SignedDecimal256) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for SignedDecimal256, SignedDecimal256);

impl Sub for SignedDecimal256 {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SignedDecimal256(self.0 - other.0)
    }
}
forward_ref_binop!(impl Sub, sub for SignedDecimal256, SignedDecimal256);

impl SubAssign for SignedDecimal256 {
    fn sub_assign(&mut self, rhs: SignedDecimal256) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for SignedDecimal256, SignedDecimal256);

impl Mul for SignedDecimal256 {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // SignedDecimal256s are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_int512 =
            self.numerator().full_mul(other.numerator()) / Int512::from(Self::DECIMAL_FRACTIONAL);
        match result_as_int512.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}
forward_ref_binop!(impl Mul, mul for SignedDecimal256, SignedDecimal256);

impl MulAssign for SignedDecimal256 {
    fn mul_assign(&mut self, rhs: SignedDecimal256) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for SignedDecimal256, SignedDecimal256);

impl Div for SignedDecimal256 {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match SignedDecimal256::checked_from_ratio(self.numerator(), other.numerator()) {
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
forward_ref_binop!(impl Div, div for SignedDecimal256, SignedDecimal256);

impl DivAssign for SignedDecimal256 {
    fn div_assign(&mut self, rhs: SignedDecimal256) {
        *self = *self / rhs;
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for SignedDecimal256, SignedDecimal256);

impl Div<Int256> for SignedDecimal256 {
    type Output = Self;

    fn div(self, rhs: Int256) -> Self::Output {
        SignedDecimal256(self.0 / rhs)
    }
}

impl DivAssign<Int256> for SignedDecimal256 {
    fn div_assign(&mut self, rhs: Int256) {
        self.0 /= rhs;
    }
}

impl Rem for SignedDecimal256 {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for SignedDecimal256, SignedDecimal256);

impl RemAssign<SignedDecimal256> for SignedDecimal256 {
    fn rem_assign(&mut self, rhs: SignedDecimal256) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for SignedDecimal256, SignedDecimal256);

impl<A> core::iter::Sum<A> for SignedDecimal256
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for SignedDecimal256 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for SignedDecimal256 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SignedDecimal256Visitor)
    }
}

struct SignedDecimal256Visitor;

impl<'de> de::Visitor<'de> for SignedDecimal256Visitor {
    type Value = SignedDecimal256;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match SignedDecimal256::from_str(v) {
            Ok(d) => Ok(d),
            Err(e) => Err(E::custom(format!("Error parsing decimal '{v}': {e}"))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{from_json, to_json_vec};
    use schemars::schema_for;

    fn dec(input: &str) -> SignedDecimal256 {
        SignedDecimal256::from_str(input).unwrap()
    }

    #[test]
    fn signed_decimal_256_new() {
        let expected = Int256::from(300i128);
        assert_eq!(SignedDecimal256::new(expected).0, expected);

        let expected = Int256::from(-300i128);
        assert_eq!(SignedDecimal256::new(expected).0, expected);
    }

    #[test]
    fn signed_decimal_256_raw() {
        let value = 300i128;
        assert_eq!(SignedDecimal256::raw(value).0, Int256::from(value));

        let value = -300i128;
        assert_eq!(SignedDecimal256::raw(value).0, Int256::from(value));
    }

    #[test]
    fn signed_decimal_256_one() {
        let value = SignedDecimal256::one();
        assert_eq!(value.0, SignedDecimal256::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn signed_decimal_256_zero() {
        let value = SignedDecimal256::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn signed_decimal_256_percent() {
        let value = SignedDecimal256::percent(50);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(2u8)
        );

        let value = SignedDecimal256::percent(-50);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(-2i8)
        );
    }

    #[test]
    fn signed_decimal_256_permille() {
        let value = SignedDecimal256::permille(125);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(8u8)
        );

        let value = SignedDecimal256::permille(-125);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(-8i8)
        );
    }

    #[test]
    fn signed_decimal_256_bps() {
        let value = SignedDecimal256::bps(125);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(80u8)
        );

        let value = SignedDecimal256::bps(-125);
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(-80i8)
        );
    }

    #[test]
    fn signed_decimal_256_from_atomics_works() {
        let one = SignedDecimal256::one();
        let two = one + one;
        let neg_one = SignedDecimal256::negative_one();

        assert_eq!(SignedDecimal256::from_atomics(1i128, 0).unwrap(), one);
        assert_eq!(SignedDecimal256::from_atomics(10i128, 1).unwrap(), one);
        assert_eq!(SignedDecimal256::from_atomics(100i128, 2).unwrap(), one);
        assert_eq!(SignedDecimal256::from_atomics(1000i128, 3).unwrap(), one);
        assert_eq!(
            SignedDecimal256::from_atomics(1000000000000000000i128, 18).unwrap(),
            one
        );
        assert_eq!(
            SignedDecimal256::from_atomics(10000000000000000000i128, 19).unwrap(),
            one
        );
        assert_eq!(
            SignedDecimal256::from_atomics(100000000000000000000i128, 20).unwrap(),
            one
        );

        assert_eq!(SignedDecimal256::from_atomics(2i128, 0).unwrap(), two);
        assert_eq!(SignedDecimal256::from_atomics(20i128, 1).unwrap(), two);
        assert_eq!(SignedDecimal256::from_atomics(200i128, 2).unwrap(), two);
        assert_eq!(SignedDecimal256::from_atomics(2000i128, 3).unwrap(), two);
        assert_eq!(
            SignedDecimal256::from_atomics(2000000000000000000i128, 18).unwrap(),
            two
        );
        assert_eq!(
            SignedDecimal256::from_atomics(20000000000000000000i128, 19).unwrap(),
            two
        );
        assert_eq!(
            SignedDecimal256::from_atomics(200000000000000000000i128, 20).unwrap(),
            two
        );

        assert_eq!(SignedDecimal256::from_atomics(-1i128, 0).unwrap(), neg_one);
        assert_eq!(SignedDecimal256::from_atomics(-10i128, 1).unwrap(), neg_one);
        assert_eq!(
            SignedDecimal256::from_atomics(-100000000000000000000i128, 20).unwrap(),
            neg_one
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            SignedDecimal256::from_atomics(4321i128, 20).unwrap(),
            SignedDecimal256::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(-4321i128, 20).unwrap(),
            SignedDecimal256::from_str("-0.000000000000000043").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(6789i128, 20).unwrap(),
            SignedDecimal256::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 38).unwrap(),
            SignedDecimal256::from_str("1.701411834604692317").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 39).unwrap(),
            SignedDecimal256::from_str("0.170141183460469231").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 45).unwrap(),
            SignedDecimal256::from_str("0.000000170141183460").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 51).unwrap(),
            SignedDecimal256::from_str("0.000000000000170141").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 56).unwrap(),
            SignedDecimal256::from_str("0.000000000000000001").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, 57).unwrap(),
            SignedDecimal256::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            SignedDecimal256::from_atomics(i128::MAX, u32::MAX).unwrap(),
            SignedDecimal256::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = SignedDecimal256::MAX;
        assert_eq!(
            SignedDecimal256::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Can be used with min value
        let min = SignedDecimal256::MIN;
        assert_eq!(
            SignedDecimal256::from_atomics(min.atomics(), min.decimal_places()).unwrap(),
            min
        );

        // Overflow is only possible with digits < 18
        let result = SignedDecimal256::from_atomics(Int256::MAX, 17);
        assert_eq!(result.unwrap_err(), SignedDecimal256RangeExceeded);
    }

    #[test]
    fn signed_decimal_256_from_ratio_works() {
        // 1.0
        assert_eq!(
            SignedDecimal256::from_ratio(1i128, 1i128),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::from_ratio(53i128, 53i128),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::from_ratio(125i128, 125i128),
            SignedDecimal256::one()
        );

        // -1.0
        assert_eq!(
            SignedDecimal256::from_ratio(-1i128, 1i128),
            SignedDecimal256::negative_one()
        );
        assert_eq!(
            SignedDecimal256::from_ratio(-53i128, 53i128),
            SignedDecimal256::negative_one()
        );
        assert_eq!(
            SignedDecimal256::from_ratio(125i128, -125i128),
            SignedDecimal256::negative_one()
        );

        // 1.5
        assert_eq!(
            SignedDecimal256::from_ratio(3i128, 2i128),
            SignedDecimal256::percent(150)
        );
        assert_eq!(
            SignedDecimal256::from_ratio(150i128, 100i128),
            SignedDecimal256::percent(150)
        );
        assert_eq!(
            SignedDecimal256::from_ratio(333i128, 222i128),
            SignedDecimal256::percent(150)
        );

        // 0.125
        assert_eq!(
            SignedDecimal256::from_ratio(1i64, 8i64),
            SignedDecimal256::permille(125)
        );
        assert_eq!(
            SignedDecimal256::from_ratio(125i64, 1000i64),
            SignedDecimal256::permille(125)
        );

        // -0.125
        assert_eq!(
            SignedDecimal256::from_ratio(-1i64, 8i64),
            SignedDecimal256::permille(-125)
        );
        assert_eq!(
            SignedDecimal256::from_ratio(125i64, -1000i64),
            SignedDecimal256::permille(-125)
        );

        // 1/3 (result floored)
        assert_eq!(
            SignedDecimal256::from_ratio(1i64, 3i64),
            SignedDecimal256(Int256::from(333_333_333_333_333_333i128))
        );

        // 2/3 (result floored)
        assert_eq!(
            SignedDecimal256::from_ratio(2i64, 3i64),
            SignedDecimal256(Int256::from(666_666_666_666_666_666i128))
        );

        // large inputs
        assert_eq!(
            SignedDecimal256::from_ratio(0i128, i128::MAX),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::from_ratio(i128::MAX, i128::MAX),
            SignedDecimal256::one()
        );
        // 170141183460469231731 is the largest integer <= SignedDecimal256::MAX
        assert_eq!(
            SignedDecimal256::from_ratio(170141183460469231731i128, 1i128),
            SignedDecimal256::from_str("170141183460469231731").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn signed_decimal_256_from_ratio_panics_for_zero_denominator() {
        SignedDecimal256::from_ratio(1i128, 0i128);
    }

    #[test]
    #[should_panic(expected = "Multiplication overflow")]
    fn signed_decimal_256_from_ratio_panics_for_mul_overflow() {
        SignedDecimal256::from_ratio(Int256::MAX, 1i128);
    }

    #[test]
    fn signed_decimal_256_checked_from_ratio_does_not_panic() {
        assert_eq!(
            SignedDecimal256::checked_from_ratio(1i128, 0i128),
            Err(CheckedFromRatioError::DivideByZero)
        );

        assert_eq!(
            SignedDecimal256::checked_from_ratio(Int256::MAX, 1i128),
            Err(CheckedFromRatioError::Overflow)
        );
    }

    #[test]
    fn signed_decimal_256_implements_fraction() {
        let fraction = SignedDecimal256::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Int256::from(1_234_567_000_000_000_000_000i128)
        );
        assert_eq!(
            fraction.denominator(),
            Int256::from(1_000_000_000_000_000_000i128)
        );

        let fraction = SignedDecimal256::from_str("-1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Int256::from(-1_234_567_000_000_000_000_000i128)
        );
        assert_eq!(
            fraction.denominator(),
            Int256::from(1_000_000_000_000_000_000i128)
        );
    }

    #[test]
    fn signed_decimal_256_from_str_works() {
        // Integers
        assert_eq!(
            SignedDecimal256::from_str("0").unwrap(),
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::from_str("1").unwrap(),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::from_str("5").unwrap(),
            SignedDecimal256::percent(500)
        );
        assert_eq!(
            SignedDecimal256::from_str("42").unwrap(),
            SignedDecimal256::percent(4200)
        );
        assert_eq!(
            SignedDecimal256::from_str("000").unwrap(),
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::from_str("001").unwrap(),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::from_str("005").unwrap(),
            SignedDecimal256::percent(500)
        );
        assert_eq!(
            SignedDecimal256::from_str("0042").unwrap(),
            SignedDecimal256::percent(4200)
        );

        // Positive decimals
        assert_eq!(
            SignedDecimal256::from_str("1.0").unwrap(),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::from_str("1.5").unwrap(),
            SignedDecimal256::percent(150)
        );
        assert_eq!(
            SignedDecimal256::from_str("0.5").unwrap(),
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            SignedDecimal256::from_str("0.123").unwrap(),
            SignedDecimal256::permille(123)
        );

        assert_eq!(
            SignedDecimal256::from_str("40.00").unwrap(),
            SignedDecimal256::percent(4000)
        );
        assert_eq!(
            SignedDecimal256::from_str("04.00").unwrap(),
            SignedDecimal256::percent(400)
        );
        assert_eq!(
            SignedDecimal256::from_str("00.40").unwrap(),
            SignedDecimal256::percent(40)
        );
        assert_eq!(
            SignedDecimal256::from_str("00.04").unwrap(),
            SignedDecimal256::percent(4)
        );
        // Negative decimals
        assert_eq!(
            SignedDecimal256::from_str("-00.04").unwrap(),
            SignedDecimal256::percent(-4)
        );
        assert_eq!(
            SignedDecimal256::from_str("-00.40").unwrap(),
            SignedDecimal256::percent(-40)
        );
        assert_eq!(
            SignedDecimal256::from_str("-04.00").unwrap(),
            SignedDecimal256::percent(-400)
        );

        // Can handle DECIMAL_PLACES fractional digits
        assert_eq!(
            SignedDecimal256::from_str("7.123456789012345678").unwrap(),
            SignedDecimal256(Int256::from(7123456789012345678i128))
        );
        assert_eq!(
            SignedDecimal256::from_str("7.999999999999999999").unwrap(),
            SignedDecimal256(Int256::from(7999999999999999999i128))
        );

        // Works for documented max value
        assert_eq!(
            SignedDecimal256::from_str(
                "57896044618658097711785492504343953926634992332820282019728.792003956564819967"
            )
            .unwrap(),
            SignedDecimal256::MAX
        );
        // Works for documented min value
        assert_eq!(
            SignedDecimal256::from_str(
                "-57896044618658097711785492504343953926634992332820282019728.792003956564819968"
            )
            .unwrap(),
            SignedDecimal256::MIN
        );
        assert_eq!(
            SignedDecimal256::from_str("-1").unwrap(),
            SignedDecimal256::negative_one()
        );
    }

    #[test]
    fn signed_decimal_256_from_str_errors_for_broken_whole_part() {
        let expected_err = StdError::generic_err("Error parsing whole");
        assert_eq!(SignedDecimal256::from_str("").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal256::from_str(" ").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal256::from_str("-").unwrap_err(), expected_err);
    }

    #[test]
    fn signed_decimal_256_from_str_errors_for_broken_fractional_part() {
        let expected_err = StdError::generic_err("Error parsing fractional");
        assert_eq!(SignedDecimal256::from_str("1.").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal256::from_str("1. ").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal256::from_str("1.e").unwrap_err(), expected_err);
        assert_eq!(
            SignedDecimal256::from_str("1.2e3").unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal256::from_str("1.-2").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_256_from_str_errors_for_more_than_18_fractional_digits() {
        let expected_err = StdError::generic_err("Cannot parse more than 18 fractional digits");
        assert_eq!(
            SignedDecimal256::from_str("7.1234567890123456789").unwrap_err(),
            expected_err
        );
        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        assert_eq!(
            SignedDecimal256::from_str("7.1230000000000000000").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_256_from_str_errors_for_invalid_number_of_dots() {
        let expected_err = StdError::generic_err("Unexpected number of dots");
        assert_eq!(
            SignedDecimal256::from_str("1.2.3").unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal256::from_str("1.2.3.4").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_256_from_str_errors_for_more_than_max_value() {
        let expected_err = StdError::generic_err("Value too big");
        // Integer
        assert_eq!(
            SignedDecimal256::from_str(
                "57896044618658097711785492504343953926634992332820282019729",
            )
            .unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal256::from_str(
                "-57896044618658097711785492504343953926634992332820282019729",
            )
            .unwrap_err(),
            expected_err
        );

        // SignedDecimal256
        assert_eq!(
            SignedDecimal256::from_str(
                "57896044618658097711785492504343953926634992332820282019729.0",
            )
            .unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal256::from_str(
                "57896044618658097711785492504343953926634992332820282019728.792003956564819968",
            )
            .unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal256::from_str(
                "-57896044618658097711785492504343953926634992332820282019728.792003956564819969",
            )
            .unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_256_conversions_work() {
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::zero()),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::one()),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::percent(50)),
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::MAX),
            SignedDecimal256::new(Int256::from_i128(i128::MAX))
        );
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::percent(-50)),
            SignedDecimal256::percent(-50)
        );
        assert_eq!(
            SignedDecimal256::from(SignedDecimal::MIN),
            SignedDecimal256::new(Int256::from_i128(i128::MIN))
        );
    }

    #[test]
    fn signed_decimal_256_atomics_works() {
        let zero = SignedDecimal256::zero();
        let one = SignedDecimal256::one();
        let half = SignedDecimal256::percent(50);
        let two = SignedDecimal256::percent(200);
        let max = SignedDecimal256::MAX;
        let neg_half = SignedDecimal256::percent(-50);
        let neg_two = SignedDecimal256::percent(-200);
        let min = SignedDecimal256::MIN;

        assert_eq!(zero.atomics(), Int256::from(0));
        assert_eq!(one.atomics(), Int256::from(1000000000000000000i128));
        assert_eq!(half.atomics(), Int256::from(500000000000000000i128));
        assert_eq!(two.atomics(), Int256::from(2000000000000000000i128));
        assert_eq!(max.atomics(), Int256::MAX);
        assert_eq!(neg_half.atomics(), Int256::from(-500000000000000000i128));
        assert_eq!(neg_two.atomics(), Int256::from(-2000000000000000000i128));
        assert_eq!(min.atomics(), Int256::MIN);
    }

    #[test]
    fn signed_decimal_256_decimal_places_works() {
        let zero = SignedDecimal256::zero();
        let one = SignedDecimal256::one();
        let half = SignedDecimal256::percent(50);
        let two = SignedDecimal256::percent(200);
        let max = SignedDecimal256::MAX;
        let neg_one = SignedDecimal256::negative_one();

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
        assert_eq!(neg_one.decimal_places(), 18);
    }

    #[test]
    fn signed_decimal_256_is_zero_works() {
        assert!(SignedDecimal256::zero().is_zero());
        assert!(SignedDecimal256::percent(0).is_zero());
        assert!(SignedDecimal256::permille(0).is_zero());

        assert!(!SignedDecimal256::one().is_zero());
        assert!(!SignedDecimal256::percent(123).is_zero());
        assert!(!SignedDecimal256::permille(-1234).is_zero());
    }

    #[test]
    fn signed_decimal_256_inv_works() {
        // d = 0
        assert_eq!(SignedDecimal256::zero().inv(), None);

        // d == 1
        assert_eq!(SignedDecimal256::one().inv(), Some(SignedDecimal256::one()));

        // d == -1
        assert_eq!(
            SignedDecimal256::negative_one().inv(),
            Some(SignedDecimal256::negative_one())
        );

        // d > 1 exact
        assert_eq!(
            SignedDecimal256::from_str("2").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.5").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("20").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.05").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("200").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.005").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("2000").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            SignedDecimal256::from_str("3").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("6").unwrap().inv(),
            Some(SignedDecimal256::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            SignedDecimal256::from_str("0.5").unwrap().inv(),
            Some(SignedDecimal256::from_str("2").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("0.05").unwrap().inv(),
            Some(SignedDecimal256::from_str("20").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("0.005").unwrap().inv(),
            Some(SignedDecimal256::from_str("200").unwrap())
        );
        assert_eq!(
            SignedDecimal256::from_str("0.0005").unwrap().inv(),
            Some(SignedDecimal256::from_str("2000").unwrap())
        );

        // d < 0
        assert_eq!(
            SignedDecimal256::from_str("-0.5").unwrap().inv(),
            Some(SignedDecimal256::from_str("-2").unwrap())
        );
        // d < 0 rounded
        assert_eq!(
            SignedDecimal256::from_str("-3").unwrap().inv(),
            Some(SignedDecimal256::from_str("-0.333333333333333333").unwrap())
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_256_add_works() {
        let value = SignedDecimal256::one() + SignedDecimal256::percent(50); // 1.5
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL * Int256::from(3u8) / Int256::from(2u8)
        );

        assert_eq!(
            SignedDecimal256::percent(5) + SignedDecimal256::percent(4),
            SignedDecimal256::percent(9)
        );
        assert_eq!(
            SignedDecimal256::percent(5) + SignedDecimal256::zero(),
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            SignedDecimal256::zero() + SignedDecimal256::zero(),
            SignedDecimal256::zero()
        );
        // negative numbers
        assert_eq!(
            SignedDecimal256::percent(-5) + SignedDecimal256::percent(-4),
            SignedDecimal256::percent(-9)
        );
        assert_eq!(
            SignedDecimal256::percent(-5) + SignedDecimal256::percent(4),
            SignedDecimal256::percent(-1)
        );
        assert_eq!(
            SignedDecimal256::percent(5) + SignedDecimal256::percent(-4),
            SignedDecimal256::percent(1)
        );

        // works for refs
        let a = SignedDecimal256::percent(15);
        let b = SignedDecimal256::percent(25);
        let expected = SignedDecimal256::percent(40);
        assert_eq!(a + b, expected);
        assert_eq!(&a + b, expected);
        assert_eq!(a + &b, expected);
        assert_eq!(&a + &b, expected);
    }

    #[test]
    #[should_panic]
    fn signed_decimal_256_add_overflow_panics() {
        let _value = SignedDecimal256::MAX + SignedDecimal256::percent(50);
    }

    #[test]
    fn signed_decimal_256_add_assign_works() {
        let mut a = SignedDecimal256::percent(30);
        a += SignedDecimal256::percent(20);
        assert_eq!(a, SignedDecimal256::percent(50));

        // works for refs
        let mut a = SignedDecimal256::percent(15);
        let b = SignedDecimal256::percent(3);
        let expected = SignedDecimal256::percent(18);
        a += &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_256_sub_works() {
        let value = SignedDecimal256::one() - SignedDecimal256::percent(50); // 0.5
        assert_eq!(
            value.0,
            SignedDecimal256::DECIMAL_FRACTIONAL / Int256::from(2u8)
        );

        assert_eq!(
            SignedDecimal256::percent(9) - SignedDecimal256::percent(4),
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            SignedDecimal256::percent(16) - SignedDecimal256::zero(),
            SignedDecimal256::percent(16)
        );
        assert_eq!(
            SignedDecimal256::percent(16) - SignedDecimal256::percent(16),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::zero() - SignedDecimal256::zero(),
            SignedDecimal256::zero()
        );

        // negative numbers
        assert_eq!(
            SignedDecimal256::percent(-5) - SignedDecimal256::percent(-4),
            SignedDecimal256::percent(-1)
        );
        assert_eq!(
            SignedDecimal256::percent(-5) - SignedDecimal256::percent(4),
            SignedDecimal256::percent(-9)
        );
        assert_eq!(
            SignedDecimal256::percent(500) - SignedDecimal256::percent(-4),
            SignedDecimal256::percent(504)
        );

        // works for refs
        let a = SignedDecimal256::percent(13);
        let b = SignedDecimal256::percent(6);
        let expected = SignedDecimal256::percent(7);
        assert_eq!(a - b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn signed_decimal_256_sub_overflow_panics() {
        let _value = SignedDecimal256::MIN - SignedDecimal256::percent(50);
    }

    #[test]
    fn signed_decimal_256_sub_assign_works() {
        let mut a = SignedDecimal256::percent(20);
        a -= SignedDecimal256::percent(2);
        assert_eq!(a, SignedDecimal256::percent(18));

        // works for refs
        let mut a = SignedDecimal256::percent(33);
        let b = SignedDecimal256::percent(13);
        let expected = SignedDecimal256::percent(20);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_256_implements_mul() {
        let one = SignedDecimal256::one();
        let two = one + one;
        let half = SignedDecimal256::percent(50);

        // 1*x and x*1
        assert_eq!(
            one * SignedDecimal256::percent(0),
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            one * SignedDecimal256::percent(1),
            SignedDecimal256::percent(1)
        );
        assert_eq!(
            one * SignedDecimal256::percent(10),
            SignedDecimal256::percent(10)
        );
        assert_eq!(
            one * SignedDecimal256::percent(100),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            one * SignedDecimal256::percent(1000),
            SignedDecimal256::percent(1000)
        );
        assert_eq!(one * SignedDecimal256::MAX, SignedDecimal256::MAX);
        assert_eq!(
            SignedDecimal256::percent(0) * one,
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::percent(1) * one,
            SignedDecimal256::percent(1)
        );
        assert_eq!(
            SignedDecimal256::percent(10) * one,
            SignedDecimal256::percent(10)
        );
        assert_eq!(
            SignedDecimal256::percent(100) * one,
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) * one,
            SignedDecimal256::percent(1000)
        );
        assert_eq!(SignedDecimal256::MAX * one, SignedDecimal256::MAX);
        assert_eq!(
            SignedDecimal256::percent(-1) * one,
            SignedDecimal256::percent(-1)
        );
        assert_eq!(
            one * SignedDecimal256::percent(-10),
            SignedDecimal256::percent(-10)
        );

        // double
        assert_eq!(
            two * SignedDecimal256::percent(0),
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            two * SignedDecimal256::percent(1),
            SignedDecimal256::percent(2)
        );
        assert_eq!(
            two * SignedDecimal256::percent(10),
            SignedDecimal256::percent(20)
        );
        assert_eq!(
            two * SignedDecimal256::percent(100),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            two * SignedDecimal256::percent(1000),
            SignedDecimal256::percent(2000)
        );
        assert_eq!(
            SignedDecimal256::percent(0) * two,
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::percent(1) * two,
            SignedDecimal256::percent(2)
        );
        assert_eq!(
            SignedDecimal256::percent(10) * two,
            SignedDecimal256::percent(20)
        );
        assert_eq!(
            SignedDecimal256::percent(100) * two,
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) * two,
            SignedDecimal256::percent(2000)
        );
        assert_eq!(
            SignedDecimal256::percent(-1) * two,
            SignedDecimal256::percent(-2)
        );
        assert_eq!(
            two * SignedDecimal256::new(Int256::MIN / Int256::from(2)),
            SignedDecimal256::MIN
        );

        // half
        assert_eq!(
            half * SignedDecimal256::percent(0),
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            half * SignedDecimal256::percent(1),
            SignedDecimal256::permille(5)
        );
        assert_eq!(
            half * SignedDecimal256::percent(10),
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            half * SignedDecimal256::percent(100),
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            half * SignedDecimal256::percent(1000),
            SignedDecimal256::percent(500)
        );
        assert_eq!(
            SignedDecimal256::percent(0) * half,
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::percent(1) * half,
            SignedDecimal256::permille(5)
        );
        assert_eq!(
            SignedDecimal256::percent(10) * half,
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            SignedDecimal256::percent(100) * half,
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) * half,
            SignedDecimal256::percent(500)
        );

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
        assert_eq!(
            dec("-1000000000000000000") * a,
            dec("-123127726548762582000")
        );

        // Move right
        let max = SignedDecimal256::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("57896044618658097711785492504343953926634992332820282019728.792003956564819967")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("5789604461865809771178549250434395392663499233282028201972.879200395656481996")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("578960446186580977117854925043439539266349923328202820197.287920039565648199")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("57896044618658097711785492504343953926634992332820282019.728792003956564819")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("57896044618658097711785492504343953926634992332820282.019728792003956564")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("57896044618658097711785492504343953926634992332820.282019728792003956")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("57896044618658097711785492504343953926634992332.820282019728792003")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("57896044618658097711785492504343953926634992.332820282019728792")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("57896044618658097711785492504343953926634.992332820282019728")
        );

        // works for refs
        let a = SignedDecimal256::percent(20);
        let b = SignedDecimal256::percent(30);
        let expected = SignedDecimal256::percent(6);
        assert_eq!(a * b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn signed_decimal_256_mul_assign_works() {
        let mut a = SignedDecimal256::percent(15);
        a *= SignedDecimal256::percent(60);
        assert_eq!(a, SignedDecimal256::percent(9));

        // works for refs
        let mut a = SignedDecimal256::percent(50);
        let b = SignedDecimal256::percent(20);
        a *= &b;
        assert_eq!(a, SignedDecimal256::percent(10));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn signed_decimal_256_mul_overflow_panics() {
        let _value = SignedDecimal256::MAX * SignedDecimal256::percent(101);
    }

    #[test]
    fn signed_decimal_256_checked_mul() {
        let test_data = [
            (SignedDecimal256::zero(), SignedDecimal256::zero()),
            (SignedDecimal256::zero(), SignedDecimal256::one()),
            (SignedDecimal256::one(), SignedDecimal256::zero()),
            (SignedDecimal256::percent(10), SignedDecimal256::zero()),
            (SignedDecimal256::percent(10), SignedDecimal256::percent(5)),
            (SignedDecimal256::MAX, SignedDecimal256::one()),
            (
                SignedDecimal256::MAX / Int256::from(2),
                SignedDecimal256::percent(200),
            ),
            (
                SignedDecimal256::permille(6),
                SignedDecimal256::permille(13),
            ),
            (
                SignedDecimal256::permille(-6),
                SignedDecimal256::permille(0),
            ),
            (SignedDecimal256::MAX, SignedDecimal256::negative_one()),
        ];

        // The regular core::ops::Mul is our source of truth for these tests.
        for (x, y) in test_data.into_iter() {
            assert_eq!(x * y, x.checked_mul(y).unwrap());
        }
    }

    #[test]
    fn signed_decimal_256_checked_mul_overflow() {
        assert_eq!(
            SignedDecimal256::MAX.checked_mul(SignedDecimal256::percent(200)),
            Err(OverflowError {
                operation: OverflowOperation::Mul,
                operand1: SignedDecimal256::MAX.to_string(),
                operand2: SignedDecimal256::percent(200).to_string(),
            })
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_256_implements_div() {
        let one = SignedDecimal256::one();
        let two = one + one;
        let half = SignedDecimal256::percent(50);

        // 1/x and x/1
        assert_eq!(
            one / SignedDecimal256::percent(1),
            SignedDecimal256::percent(10_000)
        );
        assert_eq!(
            one / SignedDecimal256::percent(10),
            SignedDecimal256::percent(1_000)
        );
        assert_eq!(
            one / SignedDecimal256::percent(100),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            one / SignedDecimal256::percent(1000),
            SignedDecimal256::percent(10)
        );
        assert_eq!(
            SignedDecimal256::percent(0) / one,
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::percent(1) / one,
            SignedDecimal256::percent(1)
        );
        assert_eq!(
            SignedDecimal256::percent(10) / one,
            SignedDecimal256::percent(10)
        );
        assert_eq!(
            SignedDecimal256::percent(100) / one,
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) / one,
            SignedDecimal256::percent(1000)
        );
        assert_eq!(
            one / SignedDecimal256::percent(-1),
            SignedDecimal256::percent(-10_000)
        );
        assert_eq!(
            one / SignedDecimal256::percent(-10),
            SignedDecimal256::percent(-1_000)
        );

        // double
        assert_eq!(
            two / SignedDecimal256::percent(1),
            SignedDecimal256::percent(20_000)
        );
        assert_eq!(
            two / SignedDecimal256::percent(10),
            SignedDecimal256::percent(2_000)
        );
        assert_eq!(
            two / SignedDecimal256::percent(100),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            two / SignedDecimal256::percent(1000),
            SignedDecimal256::percent(20)
        );
        assert_eq!(
            SignedDecimal256::percent(0) / two,
            SignedDecimal256::percent(0)
        );
        assert_eq!(SignedDecimal256::percent(1) / two, dec("0.005"));
        assert_eq!(
            SignedDecimal256::percent(10) / two,
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            SignedDecimal256::percent(100) / two,
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) / two,
            SignedDecimal256::percent(500)
        );
        assert_eq!(
            two / SignedDecimal256::percent(-1),
            SignedDecimal256::percent(-20_000)
        );
        assert_eq!(
            SignedDecimal256::percent(-10000) / two,
            SignedDecimal256::percent(-5000)
        );

        // half
        assert_eq!(
            half / SignedDecimal256::percent(1),
            SignedDecimal256::percent(5_000)
        );
        assert_eq!(
            half / SignedDecimal256::percent(10),
            SignedDecimal256::percent(500)
        );
        assert_eq!(
            half / SignedDecimal256::percent(100),
            SignedDecimal256::percent(50)
        );
        assert_eq!(
            half / SignedDecimal256::percent(1000),
            SignedDecimal256::percent(5)
        );
        assert_eq!(
            SignedDecimal256::percent(0) / half,
            SignedDecimal256::percent(0)
        );
        assert_eq!(
            SignedDecimal256::percent(1) / half,
            SignedDecimal256::percent(2)
        );
        assert_eq!(
            SignedDecimal256::percent(10) / half,
            SignedDecimal256::percent(20)
        );
        assert_eq!(
            SignedDecimal256::percent(100) / half,
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(1000) / half,
            SignedDecimal256::percent(2000)
        );

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
        // negative
        let a = dec("-123127726548762582");
        assert_eq!(a / dec("1"), dec("-123127726548762582"));
        assert_eq!(a / dec("10"), dec("-12312772654876258.2"));
        assert_eq!(a / dec("100"), dec("-1231277265487625.82"));
        assert_eq!(a / dec("1000"), dec("-123127726548762.582"));
        assert_eq!(a / dec("1000000"), dec("-123127726548.762582"));
        assert_eq!(a / dec("1000000000"), dec("-123127726.548762582"));
        assert_eq!(a / dec("1000000000000"), dec("-123127.726548762582"));
        assert_eq!(a / dec("1000000000000000"), dec("-123.127726548762582"));
        assert_eq!(a / dec("1000000000000000000"), dec("-0.123127726548762582"));
        assert_eq!(dec("1") / a, dec("-0.000000000000000008"));

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
        // negative
        let a = dec("-0.123127726548762582");
        assert_eq!(a / dec("1.0"), dec("-0.123127726548762582"));
        assert_eq!(a / dec("0.1"), dec("-1.23127726548762582"));
        assert_eq!(a / dec("0.01"), dec("-12.3127726548762582"));
        assert_eq!(a / dec("0.001"), dec("-123.127726548762582"));
        assert_eq!(a / dec("0.000001"), dec("-123127.726548762582"));
        assert_eq!(a / dec("0.000000001"), dec("-123127726.548762582"));

        assert_eq!(
            SignedDecimal256::percent(15) / SignedDecimal256::percent(60),
            SignedDecimal256::percent(25)
        );

        // works for refs
        let a = SignedDecimal256::percent(100);
        let b = SignedDecimal256::percent(20);
        let expected = SignedDecimal256::percent(500);
        assert_eq!(a / b, expected);
        assert_eq!(&a / b, expected);
        assert_eq!(a / &b, expected);
        assert_eq!(&a / &b, expected);
    }

    #[test]
    fn signed_decimal_256_div_assign_works() {
        let mut a = SignedDecimal256::percent(15);
        a /= SignedDecimal256::percent(20);
        assert_eq!(a, SignedDecimal256::percent(75));

        // works for refs
        let mut a = SignedDecimal256::percent(50);
        let b = SignedDecimal256::percent(20);
        a /= &b;
        assert_eq!(a, SignedDecimal256::percent(250));
    }

    #[test]
    #[should_panic(expected = "Division failed - multiplication overflow")]
    fn signed_decimal_256_div_overflow_panics() {
        let _value = SignedDecimal256::MAX / SignedDecimal256::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn signed_decimal_256_div_by_zero_panics() {
        let _value = SignedDecimal256::one() / SignedDecimal256::zero();
    }

    #[test]
    fn signed_decimal_256_int128_division() {
        // a/b
        let left = SignedDecimal256::percent(150); // 1.5
        let right = Int256::from(3);
        assert_eq!(left / right, SignedDecimal256::percent(50));

        // negative
        let left = SignedDecimal256::percent(-150); // -1.5
        let right = Int256::from(3);
        assert_eq!(left / right, SignedDecimal256::percent(-50));

        // 0/a
        let left = SignedDecimal256::zero();
        let right = Int256::from(300);
        assert_eq!(left / right, SignedDecimal256::zero());
    }

    #[test]
    #[should_panic]
    fn signed_decimal_256_int128_divide_by_zero() {
        let left = SignedDecimal256::percent(150); // 1.5
        let right = Int256::from(0);
        let _result = left / right;
    }

    #[test]
    fn signed_decimal_256_int128_div_assign() {
        // a/b
        let mut dec = SignedDecimal256::percent(150); // 1.5
        dec /= Int256::from(3);
        assert_eq!(dec, SignedDecimal256::percent(50));

        // 0/a
        let mut dec = SignedDecimal256::zero();
        dec /= Int256::from(300);
        assert_eq!(dec, SignedDecimal256::zero());
    }

    #[test]
    #[should_panic]
    fn signed_decimal_256_int128_div_assign_by_zero() {
        // a/0
        let mut dec = SignedDecimal256::percent(50);
        dec /= Int256::from(0);
    }

    #[test]
    fn signed_decimal_256_checked_pow() {
        for exp in 0..10 {
            assert_eq!(
                SignedDecimal256::one().checked_pow(exp).unwrap(),
                SignedDecimal256::one()
            );
        }

        // This case is mathematically undefined but we ensure consistency with Rust standard types
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
        assert_eq!(
            SignedDecimal256::zero().checked_pow(0).unwrap(),
            SignedDecimal256::one()
        );

        for exp in 1..10 {
            assert_eq!(
                SignedDecimal256::zero().checked_pow(exp).unwrap(),
                SignedDecimal256::zero()
            );
        }

        for exp in 1..10 {
            assert_eq!(
                SignedDecimal256::negative_one().checked_pow(exp).unwrap(),
                // alternates between 1 and -1
                if exp % 2 == 0 {
                    SignedDecimal256::one()
                } else {
                    SignedDecimal256::negative_one()
                }
            )
        }

        for num in &[
            SignedDecimal256::percent(50),
            SignedDecimal256::percent(99),
            SignedDecimal256::percent(200),
        ] {
            assert_eq!(num.checked_pow(0).unwrap(), SignedDecimal256::one())
        }

        assert_eq!(
            SignedDecimal256::percent(20).checked_pow(2).unwrap(),
            SignedDecimal256::percent(4)
        );

        assert_eq!(
            SignedDecimal256::percent(20).checked_pow(3).unwrap(),
            SignedDecimal256::permille(8)
        );

        assert_eq!(
            SignedDecimal256::percent(200).checked_pow(4).unwrap(),
            SignedDecimal256::percent(1600)
        );

        assert_eq!(
            SignedDecimal256::percent(200).checked_pow(4).unwrap(),
            SignedDecimal256::percent(1600)
        );

        assert_eq!(
            SignedDecimal256::percent(700).checked_pow(5).unwrap(),
            SignedDecimal256::percent(1680700)
        );

        assert_eq!(
            SignedDecimal256::percent(700).checked_pow(8).unwrap(),
            SignedDecimal256::percent(576480100)
        );

        assert_eq!(
            SignedDecimal256::percent(700).checked_pow(10).unwrap(),
            SignedDecimal256::percent(28247524900)
        );

        assert_eq!(
            SignedDecimal256::percent(120).checked_pow(123).unwrap(),
            SignedDecimal256(5486473221892422150877397607i128.into())
        );

        assert_eq!(
            SignedDecimal256::percent(10).checked_pow(2).unwrap(),
            SignedDecimal256(10000000000000000i128.into())
        );

        assert_eq!(
            SignedDecimal256::percent(10).checked_pow(18).unwrap(),
            SignedDecimal256(1i128.into())
        );

        let decimals = [
            SignedDecimal256::percent(-50),
            SignedDecimal256::percent(-99),
            SignedDecimal256::percent(-200),
        ];
        let exponents = [1, 2, 3, 4, 5, 8, 10];

        for d in decimals {
            for e in exponents {
                // use multiplication as source of truth
                let mut mul = Ok(d);
                for _ in 1..e {
                    mul = mul.and_then(|mul| mul.checked_mul(d));
                }
                assert_eq!(mul, d.checked_pow(e));
            }
        }
    }

    #[test]
    fn signed_decimal_256_checked_pow_overflow() {
        assert_eq!(
            SignedDecimal256::MAX.checked_pow(2),
            Err(OverflowError {
                operation: OverflowOperation::Pow,
                operand1: SignedDecimal256::MAX.to_string(),
                operand2: "2".to_string(),
            })
        );
    }

    #[test]
    fn signed_decimal_256_to_string() {
        // Integers
        assert_eq!(SignedDecimal256::zero().to_string(), "0");
        assert_eq!(SignedDecimal256::one().to_string(), "1");
        assert_eq!(SignedDecimal256::percent(500).to_string(), "5");
        assert_eq!(SignedDecimal256::percent(-500).to_string(), "-5");

        // SignedDecimal256s
        assert_eq!(SignedDecimal256::percent(125).to_string(), "1.25");
        assert_eq!(SignedDecimal256::percent(42638).to_string(), "426.38");
        assert_eq!(SignedDecimal256::percent(3).to_string(), "0.03");
        assert_eq!(SignedDecimal256::permille(987).to_string(), "0.987");
        assert_eq!(SignedDecimal256::percent(-125).to_string(), "-1.25");
        assert_eq!(SignedDecimal256::percent(-42638).to_string(), "-426.38");
        assert_eq!(SignedDecimal256::percent(-3).to_string(), "-0.03");
        assert_eq!(SignedDecimal256::permille(-987).to_string(), "-0.987");

        assert_eq!(
            SignedDecimal256(Int256::from(1i128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10i128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100i128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1000i128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10000i128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100000i128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1000000i128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10000000i128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100000000i128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1000000000i128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10000000000i128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100000000000i128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10000000000000i128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100000000000000i128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1000000000000000i128)).to_string(),
            "0.001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(10000000000000000i128)).to_string(),
            "0.01"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(100000000000000000i128)).to_string(),
            "0.1"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-1i128)).to_string(),
            "-0.000000000000000001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-100000000000000i128)).to_string(),
            "-0.0001"
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-100000000000000000i128)).to_string(),
            "-0.1"
        );
    }

    #[test]
    fn signed_decimal_256_iter_sum() {
        let items = vec![
            SignedDecimal256::zero(),
            SignedDecimal256(Int256::from(2i128)),
            SignedDecimal256(Int256::from(2i128)),
            SignedDecimal256(Int256::from(-2i128)),
        ];
        assert_eq!(
            items.iter().sum::<SignedDecimal256>(),
            SignedDecimal256(Int256::from(2i128))
        );
        assert_eq!(
            items.into_iter().sum::<SignedDecimal256>(),
            SignedDecimal256(Int256::from(2i128))
        );

        let empty: Vec<SignedDecimal256> = vec![];
        assert_eq!(
            SignedDecimal256::zero(),
            empty.iter().sum::<SignedDecimal256>()
        );
    }

    #[test]
    fn signed_decimal_256_serialize() {
        assert_eq!(to_json_vec(&SignedDecimal256::zero()).unwrap(), br#""0""#);
        assert_eq!(to_json_vec(&SignedDecimal256::one()).unwrap(), br#""1""#);
        assert_eq!(
            to_json_vec(&SignedDecimal256::percent(8)).unwrap(),
            br#""0.08""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal256::percent(87)).unwrap(),
            br#""0.87""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal256::percent(876)).unwrap(),
            br#""8.76""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal256::percent(8765)).unwrap(),
            br#""87.65""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal256::percent(-87654)).unwrap(),
            br#""-876.54""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal256::negative_one()).unwrap(),
            br#""-1""#
        );
        assert_eq!(
            to_json_vec(&-SignedDecimal256::percent(8)).unwrap(),
            br#""-0.08""#
        );
    }

    #[test]
    fn signed_decimal_256_deserialize() {
        assert_eq!(
            from_json::<SignedDecimal256>(br#""0""#).unwrap(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""1""#).unwrap(),
            SignedDecimal256::one()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""000""#).unwrap(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""001""#).unwrap(),
            SignedDecimal256::one()
        );

        assert_eq!(
            from_json::<SignedDecimal256>(br#""0.08""#).unwrap(),
            SignedDecimal256::percent(8)
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""0.87""#).unwrap(),
            SignedDecimal256::percent(87)
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""8.76""#).unwrap(),
            SignedDecimal256::percent(876)
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""87.65""#).unwrap(),
            SignedDecimal256::percent(8765)
        );

        // negative numbers
        assert_eq!(
            from_json::<SignedDecimal256>(br#""-0""#).unwrap(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""-1""#).unwrap(),
            SignedDecimal256::negative_one()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""-001""#).unwrap(),
            SignedDecimal256::negative_one()
        );
        assert_eq!(
            from_json::<SignedDecimal256>(br#""-0.08""#).unwrap(),
            SignedDecimal256::percent(-8)
        );
    }

    #[test]
    fn signed_decimal_256_abs_diff_works() {
        let a = SignedDecimal256::percent(285);
        let b = SignedDecimal256::percent(200);
        let expected = Decimal256::percent(85);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let a = SignedDecimal256::percent(-200);
        let b = SignedDecimal256::percent(200);
        let expected = Decimal256::percent(400);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let a = SignedDecimal256::percent(-200);
        let b = SignedDecimal256::percent(-240);
        let expected = Decimal256::percent(40);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_256_rem_works() {
        // 4.02 % 1.11 = 0.69
        assert_eq!(
            SignedDecimal256::percent(402) % SignedDecimal256::percent(111),
            SignedDecimal256::percent(69)
        );

        // 15.25 % 4 = 3.25
        assert_eq!(
            SignedDecimal256::percent(1525) % SignedDecimal256::percent(400),
            SignedDecimal256::percent(325)
        );

        // -20.25 % 5 = -25
        assert_eq!(
            SignedDecimal256::percent(-2025) % SignedDecimal256::percent(500),
            SignedDecimal256::percent(-25)
        );

        let a = SignedDecimal256::percent(318);
        let b = SignedDecimal256::percent(317);
        let expected = SignedDecimal256::percent(1);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn signed_decimal_256_rem_assign_works() {
        let mut a = SignedDecimal256::percent(17673);
        a %= SignedDecimal256::percent(2362);
        assert_eq!(a, SignedDecimal256::percent(1139)); // 176.73 % 23.62 = 11.39

        let mut a = SignedDecimal256::percent(4262);
        let b = SignedDecimal256::percent(1270);
        a %= &b;
        assert_eq!(a, SignedDecimal256::percent(452)); // 42.62 % 12.7 = 4.52

        let mut a = SignedDecimal256::percent(-4262);
        let b = SignedDecimal256::percent(1270);
        a %= &b;
        assert_eq!(a, SignedDecimal256::percent(-452)); // -42.62 % 12.7 = -4.52
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn signed_decimal_256_rem_panics_for_zero() {
        let _ = SignedDecimal256::percent(777) % SignedDecimal256::zero();
    }

    #[test]
    fn signed_decimal_256_checked_methods() {
        // checked add
        assert_eq!(
            SignedDecimal256::percent(402)
                .checked_add(SignedDecimal256::percent(111))
                .unwrap(),
            SignedDecimal256::percent(513)
        );
        assert!(matches!(
            SignedDecimal256::MAX.checked_add(SignedDecimal256::percent(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            SignedDecimal256::MIN.checked_add(SignedDecimal256::percent(-1)),
            Err(OverflowError { .. })
        ));

        // checked sub
        assert_eq!(
            SignedDecimal256::percent(1111)
                .checked_sub(SignedDecimal256::percent(111))
                .unwrap(),
            SignedDecimal256::percent(1000)
        );
        assert_eq!(
            SignedDecimal256::zero()
                .checked_sub(SignedDecimal256::percent(1))
                .unwrap(),
            SignedDecimal256::percent(-1)
        );
        assert!(matches!(
            SignedDecimal256::MIN.checked_sub(SignedDecimal256::percent(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            SignedDecimal256::MAX.checked_sub(SignedDecimal256::percent(-1)),
            Err(OverflowError { .. })
        ));

        // checked div
        assert_eq!(
            SignedDecimal256::percent(30)
                .checked_div(SignedDecimal256::percent(200))
                .unwrap(),
            SignedDecimal256::percent(15)
        );
        assert_eq!(
            SignedDecimal256::percent(88)
                .checked_div(SignedDecimal256::percent(20))
                .unwrap(),
            SignedDecimal256::percent(440)
        );
        assert!(matches!(
            SignedDecimal256::MAX.checked_div(SignedDecimal256::zero()),
            Err(CheckedFromRatioError::DivideByZero {})
        ));
        assert!(matches!(
            SignedDecimal256::MAX.checked_div(SignedDecimal256::percent(1)),
            Err(CheckedFromRatioError::Overflow {})
        ));
        assert_eq!(
            SignedDecimal256::percent(-88)
                .checked_div(SignedDecimal256::percent(20))
                .unwrap(),
            SignedDecimal256::percent(-440)
        );
        assert_eq!(
            SignedDecimal256::percent(-88)
                .checked_div(SignedDecimal256::percent(-20))
                .unwrap(),
            SignedDecimal256::percent(440)
        );

        // checked rem
        assert_eq!(
            SignedDecimal256::percent(402)
                .checked_rem(SignedDecimal256::percent(111))
                .unwrap(),
            SignedDecimal256::percent(69)
        );
        assert_eq!(
            SignedDecimal256::percent(1525)
                .checked_rem(SignedDecimal256::percent(400))
                .unwrap(),
            SignedDecimal256::percent(325)
        );
        assert_eq!(
            SignedDecimal256::percent(-1525)
                .checked_rem(SignedDecimal256::percent(400))
                .unwrap(),
            SignedDecimal256::percent(-325)
        );
        assert_eq!(
            SignedDecimal256::percent(-1525)
                .checked_rem(SignedDecimal256::percent(-400))
                .unwrap(),
            SignedDecimal256::percent(-325)
        );
        assert!(matches!(
            SignedDecimal256::MAX.checked_rem(SignedDecimal256::zero()),
            Err(DivideByZeroError { .. })
        ));
    }

    #[test]
    fn signed_decimal_256_pow_works() {
        assert_eq!(
            SignedDecimal256::percent(200).pow(2),
            SignedDecimal256::percent(400)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).pow(2),
            SignedDecimal256::percent(400)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).pow(3),
            SignedDecimal256::percent(-800)
        );
        assert_eq!(
            SignedDecimal256::percent(200).pow(10),
            SignedDecimal256::percent(102400)
        );
    }

    #[test]
    #[should_panic]
    fn signed_decimal_256_pow_overflow_panics() {
        _ = SignedDecimal256::MAX.pow(2u32);
    }

    #[test]
    fn signed_decimal_256_saturating_works() {
        assert_eq!(
            SignedDecimal256::percent(200).saturating_add(SignedDecimal256::percent(200)),
            SignedDecimal256::percent(400)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_add(SignedDecimal256::percent(200)),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_add(SignedDecimal256::percent(-200)),
            SignedDecimal256::percent(-400)
        );
        assert_eq!(
            SignedDecimal256::MAX.saturating_add(SignedDecimal256::percent(200)),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_add(SignedDecimal256::percent(-1)),
            SignedDecimal256::MIN
        );
        assert_eq!(
            SignedDecimal256::percent(200).saturating_sub(SignedDecimal256::percent(100)),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_sub(SignedDecimal256::percent(100)),
            SignedDecimal256::percent(-300)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_sub(SignedDecimal256::percent(-100)),
            SignedDecimal256::percent(-100)
        );
        assert_eq!(
            SignedDecimal256::zero().saturating_sub(SignedDecimal256::percent(200)),
            SignedDecimal256::from_str("-2").unwrap()
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_sub(SignedDecimal256::percent(200)),
            SignedDecimal256::MIN
        );
        assert_eq!(
            SignedDecimal256::MAX.saturating_sub(SignedDecimal256::percent(-200)),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::percent(200).saturating_mul(SignedDecimal256::percent(50)),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_mul(SignedDecimal256::percent(50)),
            SignedDecimal256::percent(-100)
        );
        assert_eq!(
            SignedDecimal256::percent(-200).saturating_mul(SignedDecimal256::percent(-50)),
            SignedDecimal256::percent(100)
        );
        assert_eq!(
            SignedDecimal256::MAX.saturating_mul(SignedDecimal256::percent(200)),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_mul(SignedDecimal256::percent(200)),
            SignedDecimal256::MIN
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_mul(SignedDecimal256::percent(-200)),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::percent(400).saturating_pow(2u32),
            SignedDecimal256::percent(1600)
        );
        assert_eq!(
            SignedDecimal256::MAX.saturating_pow(2u32),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::MAX.saturating_pow(3u32),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_pow(2u32),
            SignedDecimal256::MAX
        );
        assert_eq!(
            SignedDecimal256::MIN.saturating_pow(3u32),
            SignedDecimal256::MIN
        );
    }

    #[test]
    fn signed_decimal_256_rounding() {
        assert_eq!(SignedDecimal256::one().floor(), SignedDecimal256::one());
        assert_eq!(
            SignedDecimal256::percent(150).floor(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::percent(199).floor(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::percent(200).floor(),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(99).floor(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1i128)).floor(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-1i128)).floor(),
            SignedDecimal256::negative_one()
        );
        assert_eq!(
            SignedDecimal256::permille(-1234).floor(),
            SignedDecimal256::percent(-200)
        );

        assert_eq!(SignedDecimal256::one().ceil(), SignedDecimal256::one());
        assert_eq!(
            SignedDecimal256::percent(150).ceil(),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(199).ceil(),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(99).ceil(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1i128)).ceil(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-1i128)).ceil(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::permille(-1234).ceil(),
            SignedDecimal256::negative_one()
        );

        assert_eq!(SignedDecimal256::one().trunc(), SignedDecimal256::one());
        assert_eq!(
            SignedDecimal256::percent(150).trunc(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::percent(199).trunc(),
            SignedDecimal256::one()
        );
        assert_eq!(
            SignedDecimal256::percent(200).trunc(),
            SignedDecimal256::percent(200)
        );
        assert_eq!(
            SignedDecimal256::percent(99).trunc(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(1i128)).trunc(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256(Int256::from(-1i128)).trunc(),
            SignedDecimal256::zero()
        );
        assert_eq!(
            SignedDecimal256::permille(-1234).trunc(),
            SignedDecimal256::negative_one()
        );
    }

    #[test]
    #[should_panic(expected = "attempt to ceil with overflow")]
    fn signed_decimal_256_ceil_panics() {
        let _ = SignedDecimal256::MAX.ceil();
    }

    #[test]
    #[should_panic(expected = "attempt to floor with overflow")]
    fn signed_decimal_256_floor_panics() {
        let _ = SignedDecimal256::MIN.floor();
    }

    #[test]
    fn signed_decimal_256_checked_ceil() {
        assert_eq!(
            SignedDecimal256::percent(199).checked_ceil(),
            Ok(SignedDecimal256::percent(200))
        );
        assert_eq!(
            SignedDecimal256::MAX.checked_ceil(),
            Err(RoundUpOverflowError)
        );
    }

    #[test]
    fn signed_decimal_256_checked_floor() {
        assert_eq!(
            SignedDecimal256::percent(199).checked_floor(),
            Ok(SignedDecimal256::one())
        );
        assert_eq!(
            SignedDecimal256::percent(-199).checked_floor(),
            Ok(SignedDecimal256::percent(-200))
        );
        assert_eq!(
            SignedDecimal256::MIN.checked_floor(),
            Err(RoundDownOverflowError)
        );
        assert_eq!(
            SignedDecimal256::negative_one().checked_floor(),
            Ok(SignedDecimal256::negative_one())
        );
    }

    #[test]
    fn signed_decimal_256_to_int_floor_works() {
        let d = SignedDecimal256::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(12));
        let d = SignedDecimal256::from_str("12.345").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(12));
        let d = SignedDecimal256::from_str("12.999").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(12));
        let d = SignedDecimal256::from_str("0.98451384").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(0));
        let d = SignedDecimal256::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(-13));
        let d = SignedDecimal256::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(-13));
        let d = SignedDecimal256::from_str("0.0001").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(0));
        let d = SignedDecimal256::from_str("75.0").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(75));
        let d = SignedDecimal256::from_str("0.0").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(0));
        let d = SignedDecimal256::from_str("-0.0").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(0));
        let d = SignedDecimal256::from_str("-0.0001").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(-1));
        let d = SignedDecimal256::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_floor(), Int256::from(-75));

        let d = SignedDecimal256::MAX;
        assert_eq!(
            d.to_int_floor(),
            Int256::from_str("57896044618658097711785492504343953926634992332820282019728")
                .unwrap()
        );
        let d = SignedDecimal256::MIN;
        assert_eq!(
            d.to_int_floor(),
            Int256::from_str("-57896044618658097711785492504343953926634992332820282019729")
                .unwrap()
        );
    }

    #[test]
    fn signed_decimal_256_to_int_ceil_works() {
        let d = SignedDecimal256::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(13));
        let d = SignedDecimal256::from_str("12.345").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(13));
        let d = SignedDecimal256::from_str("12.999").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(13));
        let d = SignedDecimal256::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(-12));
        let d = SignedDecimal256::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(-12));

        let d = SignedDecimal256::from_str("75.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(75));
        let d = SignedDecimal256::from_str("0.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(0));
        let d = SignedDecimal256::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int256::from(-75));

        let d = SignedDecimal256::MAX;
        assert_eq!(
            d.to_int_ceil(),
            Int256::from_str("57896044618658097711785492504343953926634992332820282019729")
                .unwrap()
        );
        let d = SignedDecimal256::MIN;
        assert_eq!(
            d.to_int_ceil(),
            Int256::from_str("-57896044618658097711785492504343953926634992332820282019728")
                .unwrap()
        );
    }

    #[test]
    fn signed_decimal_256_to_int_trunc_works() {
        let d = SignedDecimal256::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(12));
        let d = SignedDecimal256::from_str("12.345").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(12));
        let d = SignedDecimal256::from_str("12.999").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(12));
        let d = SignedDecimal256::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(-12));
        let d = SignedDecimal256::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(-12));

        let d = SignedDecimal256::from_str("75.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(75));
        let d = SignedDecimal256::from_str("0.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(0));
        let d = SignedDecimal256::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int256::from(-75));

        let d = SignedDecimal256::MAX;
        assert_eq!(
            d.to_int_trunc(),
            Int256::from_str("57896044618658097711785492504343953926634992332820282019728")
                .unwrap()
        );
        let d = SignedDecimal256::MIN;
        assert_eq!(
            d.to_int_trunc(),
            Int256::from_str("-57896044618658097711785492504343953926634992332820282019728")
                .unwrap()
        );
    }

    #[test]
    fn signed_decimal_256_neg_works() {
        assert_eq!(
            -SignedDecimal256::percent(50),
            SignedDecimal256::percent(-50)
        );
        assert_eq!(-SignedDecimal256::one(), SignedDecimal256::negative_one());
    }

    #[test]
    fn signed_decimal_256_partial_eq() {
        let test_cases = [
            ("1", "1", true),
            ("0.5", "0.5", true),
            ("0.5", "0.51", false),
            ("0", "0.00000", true),
            ("-1", "-1", true),
            ("-0.5", "-0.5", true),
            ("-0.5", "0.5", false),
            ("-0.5", "-0.51", false),
            ("-0", "-0.00000", true),
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
    fn signed_decimal_256_implements_debug() {
        let decimal = SignedDecimal256::from_str("123.45").unwrap();
        assert_eq!(format!("{decimal:?}"), "SignedDecimal256(123.45)");

        let test_cases = ["5", "5.01", "42", "0", "2", "-0.000001"];
        for s in test_cases {
            let decimal = SignedDecimal256::from_str(s).unwrap();
            let expected = format!("SignedDecimal256({s})");
            assert_eq!(format!("{decimal:?}"), expected);
        }
    }

    #[test]
    fn signed_decimal_256_can_be_instantiated_from_decimal() {
        let d: SignedDecimal256 = Decimal::one().into();
        assert_eq!(d, SignedDecimal256::one());
    }

    #[test]
    fn signed_decimal_256_can_be_instantiated_from_decimal_256() {
        let d: SignedDecimal256 = Decimal256::zero().try_into().unwrap();
        assert_eq!(d, SignedDecimal256::zero());
    }

    #[test]
    fn signed_decimal_256_may_fail_when_instantiated_from_decimal_256() {
        let err = <Decimal256 as TryInto<SignedDecimal256>>::try_into(Decimal256::MAX).unwrap_err();
        assert_eq!("SignedDecimal256RangeExceeded", format!("{err:?}"));
        assert_eq!("SignedDecimal256 range exceeded", format!("{err}"));
    }

    #[test]
    fn signed_decimal_256_can_be_serialized_and_deserialized() {
        // properly deserialized
        let value: SignedDecimal256 = serde_json::from_str(r#""123""#).unwrap();
        assert_eq!(SignedDecimal256::from_str("123").unwrap(), value);

        // properly serialized
        let value = SignedDecimal256::from_str("456").unwrap();
        assert_eq!(r#""456""#, serde_json::to_string(&value).unwrap());

        // invalid: not a string encoded decimal
        assert_eq!(
            "invalid type: integer `123`, expected string-encoded decimal at line 1 column 3",
            serde_json::from_str::<SignedDecimal256>("123")
                .err()
                .unwrap()
                .to_string()
        );

        // invalid: not properly defined signed decimal value
        assert_eq!(
            "Error parsing decimal '1.e': Generic error: Error parsing fractional at line 1 column 5",
            serde_json::from_str::<SignedDecimal256>(r#""1.e""#)
                .err()
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn signed_decimal_256_has_defined_json_schema() {
        let schema = schema_for!(SignedDecimal256);
        assert_eq!(
            "SignedDecimal256",
            schema.schema.metadata.unwrap().title.unwrap()
        );
    }
}
