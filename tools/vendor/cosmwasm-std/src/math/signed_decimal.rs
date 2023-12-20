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
use crate::{forward_ref_partial_eq, Decimal, Decimal256, Int256, SignedDecimal256};

use super::Fraction;
use super::Int128;

/// A signed fixed-point decimal value with 18 fractional digits, i.e. SignedDecimal(1_000_000_000_000_000_000) == 1.0
///
/// The greatest possible value that can be represented is 170141183460469231731.687303715884105727 (which is (2^127 - 1) / 10^18)
/// and the smallest is -170141183460469231731.687303715884105728 (which is -2^127 / 10^18).
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct SignedDecimal(#[schemars(with = "String")] Int128);

forward_ref_partial_eq!(SignedDecimal, SignedDecimal);

#[derive(Error, Debug, PartialEq, Eq)]
#[error("SignedDecimal range exceeded")]
pub struct SignedDecimalRangeExceeded;

impl SignedDecimal {
    const DECIMAL_FRACTIONAL: Int128 = Int128::new(1_000_000_000_000_000_000i128); // 1*10**18
    const DECIMAL_FRACTIONAL_SQUARED: Int128 =
        Int128::new(1_000_000_000_000_000_000_000_000_000_000_000_000i128); // (1*10**18)**2 = 1*10**36

    /// The number of decimal places. Since decimal types are fixed-point rather than
    /// floating-point, this is a constant.
    pub const DECIMAL_PLACES: u32 = 18; // This needs to be an even number.

    /// The largest value that can be represented by this signed decimal type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal;
    /// assert_eq!(SignedDecimal::MAX.to_string(), "170141183460469231731.687303715884105727");
    /// ```
    pub const MAX: Self = Self(Int128::MAX);

    /// The smallest value that can be represented by this signed decimal type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal;
    /// assert_eq!(SignedDecimal::MIN.to_string(), "-170141183460469231731.687303715884105728");
    /// ```
    pub const MIN: Self = Self(Int128::MIN);

    /// Creates a SignedDecimal(value)
    /// This is equivalent to `SignedDecimal::from_atomics(value, 18)` but usable in a const context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal, Int128};
    /// assert_eq!(SignedDecimal::new(Int128::one()).to_string(), "0.000000000000000001");
    /// ```
    pub const fn new(value: Int128) -> Self {
        Self(value)
    }

    /// Creates a SignedDecimal(Int128(value))
    /// This is equivalent to `SignedDecimal::from_atomics(value, 18)` but usable in a const context.
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal;
    /// assert_eq!(SignedDecimal::raw(1234i128).to_string(), "0.000000000000001234");
    /// ```
    pub const fn raw(value: i128) -> Self {
        Self(Int128::new(value))
    }

    /// Create a 1.0 SignedDecimal
    #[inline]
    pub const fn one() -> Self {
        Self(Self::DECIMAL_FRACTIONAL)
    }

    /// Create a -1.0 SignedDecimal
    #[inline]
    pub const fn negative_one() -> Self {
        Self(Int128::new(-Self::DECIMAL_FRACTIONAL.i128()))
    }

    /// Create a 0.0 SignedDecimal
    #[inline]
    pub const fn zero() -> Self {
        Self(Int128::zero())
    }

    /// Convert x% into SignedDecimal
    pub fn percent(x: i64) -> Self {
        Self(((x as i128) * 10_000_000_000_000_000).into())
    }

    /// Convert permille (x/1000) into SignedDecimal
    pub fn permille(x: i64) -> Self {
        Self(((x as i128) * 1_000_000_000_000_000).into())
    }

    /// Convert basis points (x/10000) into SignedDecimal
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
    /// # use cosmwasm_std::{SignedDecimal, Int128};
    /// let a = SignedDecimal::from_atomics(Int128::new(1234), 3).unwrap();
    /// assert_eq!(a.to_string(), "1.234");
    ///
    /// let a = SignedDecimal::from_atomics(1234i128, 0).unwrap();
    /// assert_eq!(a.to_string(), "1234");
    ///
    /// let a = SignedDecimal::from_atomics(1i64, 18).unwrap();
    /// assert_eq!(a.to_string(), "0.000000000000000001");
    ///
    /// let a = SignedDecimal::from_atomics(-1i64, 18).unwrap();
    /// assert_eq!(a.to_string(), "-0.000000000000000001");
    /// ```
    pub fn from_atomics(
        atomics: impl Into<Int128>,
        decimal_places: u32,
    ) -> Result<Self, SignedDecimalRangeExceeded> {
        let atomics = atomics.into();
        const TEN: Int128 = Int128::new(10);
        Ok(match decimal_places.cmp(&(Self::DECIMAL_PLACES)) {
            Ordering::Less => {
                let digits = (Self::DECIMAL_PLACES) - decimal_places; // No overflow because decimal_places < DECIMAL_PLACES
                let factor = TEN.checked_pow(digits).unwrap(); // Safe because digits <= 17
                Self(
                    atomics
                        .checked_mul(factor)
                        .map_err(|_| SignedDecimalRangeExceeded)?,
                )
            }
            Ordering::Equal => Self(atomics),
            Ordering::Greater => {
                let digits = decimal_places - (Self::DECIMAL_PLACES); // No overflow because decimal_places > DECIMAL_PLACES
                if let Ok(factor) = TEN.checked_pow(digits) {
                    Self(atomics.checked_div(factor).unwrap()) // Safe because factor cannot be zero
                } else {
                    // In this case `factor` exceeds the Int128 range.
                    // Any Int128 `x` divided by `factor` with `factor > Int128::MAX` is 0.
                    // Try e.g. Python3: `(2**128-1) // 2**128`
                    Self(Int128::zero())
                }
            }
        })
    }

    /// Returns the ratio (numerator / denominator) as a SignedDecimal
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::SignedDecimal;
    /// assert_eq!(
    ///     SignedDecimal::from_ratio(1, 3).to_string(),
    ///     "0.333333333333333333"
    /// );
    /// ```
    pub fn from_ratio(numerator: impl Into<Int128>, denominator: impl Into<Int128>) -> Self {
        match SignedDecimal::checked_from_ratio(numerator, denominator) {
            Ok(value) => value,
            Err(CheckedFromRatioError::DivideByZero) => {
                panic!("Denominator must not be zero")
            }
            Err(CheckedFromRatioError::Overflow) => panic!("Multiplication overflow"),
        }
    }

    /// Returns the ratio (numerator / denominator) as a SignedDecimal
    ///
    /// # Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal, CheckedFromRatioError};
    /// assert_eq!(
    ///     SignedDecimal::checked_from_ratio(1, 3).unwrap().to_string(),
    ///     "0.333333333333333333"
    /// );
    /// assert_eq!(
    ///     SignedDecimal::checked_from_ratio(1, 0),
    ///     Err(CheckedFromRatioError::DivideByZero)
    /// );
    /// ```
    pub fn checked_from_ratio(
        numerator: impl Into<Int128>,
        denominator: impl Into<Int128>,
    ) -> Result<Self, CheckedFromRatioError> {
        let numerator: Int128 = numerator.into();
        let denominator: Int128 = denominator.into();
        match numerator.checked_multiply_ratio(Self::DECIMAL_FRACTIONAL, denominator) {
            Ok(ratio) => {
                // numerator * DECIMAL_FRACTIONAL / denominator
                Ok(SignedDecimal(ratio))
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
        self.0.i128() < 0
    }

    /// A decimal is an integer of atomic units plus a number that specifies the
    /// position of the decimal dot. So any decimal can be expressed as two numbers.
    ///
    /// ## Examples
    ///
    /// ```
    /// # use cosmwasm_std::{SignedDecimal, Int128};
    /// # use core::str::FromStr;
    /// // Value with whole and fractional part
    /// let a = SignedDecimal::from_str("1.234").unwrap();
    /// assert_eq!(a.decimal_places(), 18);
    /// assert_eq!(a.atomics(), Int128::new(1234000000000000000));
    ///
    /// // Smallest possible value
    /// let b = SignedDecimal::from_str("0.000000000000000001").unwrap();
    /// assert_eq!(b.decimal_places(), 18);
    /// assert_eq!(b.atomics(), Int128::new(1));
    /// ```
    #[must_use]
    #[inline]
    pub const fn atomics(&self) -> Int128 {
        self.0
    }

    /// The number of decimal places. This is a constant value for now
    /// but this could potentially change as the type evolves.
    ///
    /// See also [`SignedDecimal::atomics()`].
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
    /// # use cosmwasm_std::SignedDecimal;
    /// # use core::str::FromStr;
    /// assert!(SignedDecimal::from_str("0.6").unwrap().trunc().is_zero());
    /// assert_eq!(SignedDecimal::from_str("-5.8").unwrap().trunc().to_string(), "-5");
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
    /// # use cosmwasm_std::SignedDecimal;
    /// # use core::str::FromStr;
    /// assert!(SignedDecimal::from_str("0.6").unwrap().floor().is_zero());
    /// assert_eq!(SignedDecimal::from_str("-5.2").unwrap().floor().to_string(), "-6");
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
                    .checked_sub(SignedDecimal::one())
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
    /// # use cosmwasm_std::SignedDecimal;
    /// # use core::str::FromStr;
    /// assert_eq!(SignedDecimal::from_str("0.2").unwrap().ceil(), SignedDecimal::one());
    /// assert_eq!(SignedDecimal::from_str("-5.8").unwrap().ceil().to_string(), "-5");
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
                .checked_add(SignedDecimal::one())
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

    /// Multiplies one `SignedDecimal` by another, returning an `OverflowError` if an overflow occurred.
    pub fn checked_mul(self, other: Self) -> Result<Self, OverflowError> {
        let result_as_int256 =
            self.numerator().full_mul(other.numerator()) / Int256::from(Self::DECIMAL_FRACTIONAL);
        result_as_int256
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

        fn inner(mut x: SignedDecimal, mut n: u32) -> Result<SignedDecimal, OverflowError> {
            if n == 0 {
                return Ok(SignedDecimal::one());
            }

            let mut y = SignedDecimal::one();

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
        SignedDecimal::checked_from_ratio(self.numerator(), other.numerator())
    }

    /// Computes `self % other`, returning an `DivideByZeroError` if `other == 0`.
    pub fn checked_rem(self, other: Self) -> Result<Self, DivideByZeroError> {
        self.0
            .checked_rem(other.0)
            .map(Self)
            .map_err(|_| DivideByZeroError::new(self))
    }

    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub const fn abs_diff(self, other: Self) -> Decimal {
        Decimal::new(self.0.abs_diff(other.0))
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
    /// use cosmwasm_std::{SignedDecimal, Int128};
    ///
    /// let d = SignedDecimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_floor(), Int128::new(12));
    ///
    /// let d = SignedDecimal::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_floor(), Int128::new(-13));
    ///
    /// let d = SignedDecimal::from_str("-0.05").unwrap();
    /// assert_eq!(d.to_int_floor(), Int128::new(-1));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_floor(self) -> Int128 {
        if self.is_negative() {
            // Using `x.to_int_floor() = -(-x).to_int_ceil()` for a negative `x`,
            // but avoiding overflow by implementing the formula from `to_int_ceil` directly.
            let x = self.0;
            let y = Self::DECIMAL_FRACTIONAL;
            // making sure not to negate `x`, as this would overflow
            -Int128::one() - ((-Int128::one() - x) / y)
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
    /// use cosmwasm_std::{SignedDecimal, Int128};
    ///
    /// let d = SignedDecimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int128::new(12));
    ///
    /// let d = SignedDecimal::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int128::new(-12));
    ///
    /// let d = SignedDecimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_int_trunc(), Int128::new(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_trunc(self) -> Int128 {
        self.0 / Self::DECIMAL_FRACTIONAL
    }

    /// Converts this decimal to a signed integer by rounding up
    /// to the next integer, e.g. 22.3 becomes 23 and -1.2 becomes -1.
    ///
    /// ## Examples
    ///
    /// ```
    /// use core::str::FromStr;
    /// use cosmwasm_std::{SignedDecimal, Int128};
    ///
    /// let d = SignedDecimal::from_str("12.345").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int128::new(13));
    ///
    /// let d = SignedDecimal::from_str("-12.999").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int128::new(-12));
    ///
    /// let d = SignedDecimal::from_str("75.0").unwrap();
    /// assert_eq!(d.to_int_ceil(), Int128::new(75));
    /// ```
    #[must_use = "this returns the result of the operation, without modifying the original"]
    pub fn to_int_ceil(self) -> Int128 {
        if self.is_negative() {
            self.to_int_trunc()
        } else {
            // Using `q = 1 + ((x - 1) / y); // if x != 0` with unsigned integers x, y, q
            // from https://stackoverflow.com/a/2745086/2013738. We know `x + y` CAN overflow.
            let x = self.0;
            let y = Self::DECIMAL_FRACTIONAL;
            if x.is_zero() {
                Int128::zero()
            } else {
                Int128::one() + ((x - Int128::one()) / y)
            }
        }
    }
}

impl Fraction<Int128> for SignedDecimal {
    #[inline]
    fn numerator(&self) -> Int128 {
        self.0
    }

    #[inline]
    fn denominator(&self) -> Int128 {
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
            Some(SignedDecimal(Self::DECIMAL_FRACTIONAL_SQUARED / self.0))
        }
    }
}

impl Neg for SignedDecimal {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(-self.0)
    }
}

impl TryFrom<SignedDecimal256> for SignedDecimal {
    type Error = SignedDecimalRangeExceeded;

    fn try_from(value: SignedDecimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(SignedDecimal)
            .map_err(|_| SignedDecimalRangeExceeded)
    }
}

impl TryFrom<Decimal> for SignedDecimal {
    type Error = SignedDecimalRangeExceeded;

    fn try_from(value: Decimal) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(SignedDecimal)
            .map_err(|_| SignedDecimalRangeExceeded)
    }
}

impl TryFrom<Decimal256> for SignedDecimal {
    type Error = SignedDecimalRangeExceeded;

    fn try_from(value: Decimal256) -> Result<Self, Self::Error> {
        value
            .atomics()
            .try_into()
            .map(SignedDecimal)
            .map_err(|_| SignedDecimalRangeExceeded)
    }
}

impl FromStr for SignedDecimal {
    type Err = StdError;

    /// Converts the decimal string to a SignedDecimal
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
            .parse::<Int128>()
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
            let fractional_factor = Int128::from(10i128.pow(exp));

            // This multiplication can't overflow because
            // fractional < 10^DECIMAL_PLACES && fractional_factor <= 10^DECIMAL_PLACES
            let fractional_part = Int128::from(fractional)
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

        Ok(SignedDecimal(atomics))
    }
}

impl fmt::Display for SignedDecimal {
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

impl fmt::Debug for SignedDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SignedDecimal({self})")
    }
}

impl Add for SignedDecimal {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        SignedDecimal(self.0 + other.0)
    }
}
forward_ref_binop!(impl Add, add for SignedDecimal, SignedDecimal);

impl AddAssign for SignedDecimal {
    fn add_assign(&mut self, rhs: SignedDecimal) {
        *self = *self + rhs;
    }
}
forward_ref_op_assign!(impl AddAssign, add_assign for SignedDecimal, SignedDecimal);

impl Sub for SignedDecimal {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        SignedDecimal(self.0 - other.0)
    }
}
forward_ref_binop!(impl Sub, sub for SignedDecimal, SignedDecimal);

impl SubAssign for SignedDecimal {
    fn sub_assign(&mut self, rhs: SignedDecimal) {
        *self = *self - rhs;
    }
}
forward_ref_op_assign!(impl SubAssign, sub_assign for SignedDecimal, SignedDecimal);

impl Mul for SignedDecimal {
    type Output = Self;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn mul(self, other: Self) -> Self {
        // SignedDecimals are fractions. We can multiply two decimals a and b
        // via
        //       (a.numerator() * b.numerator()) / (a.denominator() * b.denominator())
        //     = (a.numerator() * b.numerator()) / a.denominator() / b.denominator()

        let result_as_int256 =
            self.numerator().full_mul(other.numerator()) / Int256::from(Self::DECIMAL_FRACTIONAL);
        match result_as_int256.try_into() {
            Ok(result) => Self(result),
            Err(_) => panic!("attempt to multiply with overflow"),
        }
    }
}
forward_ref_binop!(impl Mul, mul for SignedDecimal, SignedDecimal);

impl MulAssign for SignedDecimal {
    fn mul_assign(&mut self, rhs: SignedDecimal) {
        *self = *self * rhs;
    }
}
forward_ref_op_assign!(impl MulAssign, mul_assign for SignedDecimal, SignedDecimal);

impl Div for SignedDecimal {
    type Output = Self;

    fn div(self, other: Self) -> Self {
        match SignedDecimal::checked_from_ratio(self.numerator(), other.numerator()) {
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
forward_ref_binop!(impl Div, div for SignedDecimal, SignedDecimal);

impl DivAssign for SignedDecimal {
    fn div_assign(&mut self, rhs: SignedDecimal) {
        *self = *self / rhs;
    }
}
forward_ref_op_assign!(impl DivAssign, div_assign for SignedDecimal, SignedDecimal);

impl Div<Int128> for SignedDecimal {
    type Output = Self;

    fn div(self, rhs: Int128) -> Self::Output {
        SignedDecimal(self.0 / rhs)
    }
}

impl DivAssign<Int128> for SignedDecimal {
    fn div_assign(&mut self, rhs: Int128) {
        self.0 /= rhs;
    }
}

impl Rem for SignedDecimal {
    type Output = Self;

    /// # Panics
    ///
    /// This operation will panic if `rhs` is zero
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        Self(self.0.rem(rhs.0))
    }
}
forward_ref_binop!(impl Rem, rem for SignedDecimal, SignedDecimal);

impl RemAssign<SignedDecimal> for SignedDecimal {
    fn rem_assign(&mut self, rhs: SignedDecimal) {
        *self = *self % rhs;
    }
}
forward_ref_op_assign!(impl RemAssign, rem_assign for SignedDecimal, SignedDecimal);

impl<A> core::iter::Sum<A> for SignedDecimal
where
    Self: Add<A, Output = Self>,
{
    fn sum<I: Iterator<Item = A>>(iter: I) -> Self {
        iter.fold(Self::zero(), Add::add)
    }
}

/// Serializes as a decimal string
impl Serialize for SignedDecimal {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Deserializes as a base64 string
impl<'de> Deserialize<'de> for SignedDecimal {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(SignedDecimalVisitor)
    }
}

struct SignedDecimalVisitor;

impl<'de> de::Visitor<'de> for SignedDecimalVisitor {
    type Value = SignedDecimal;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("string-encoded decimal")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match SignedDecimal::from_str(v) {
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

    fn dec(input: &str) -> SignedDecimal {
        SignedDecimal::from_str(input).unwrap()
    }

    #[test]
    fn signed_decimal_new() {
        let expected = Int128::from(300i128);
        assert_eq!(SignedDecimal::new(expected).0, expected);

        let expected = Int128::from(-300i128);
        assert_eq!(SignedDecimal::new(expected).0, expected);
    }

    #[test]
    fn signed_decimal_raw() {
        let value = 300i128;
        assert_eq!(SignedDecimal::raw(value).0.i128(), value);

        let value = -300i128;
        assert_eq!(SignedDecimal::raw(value).0.i128(), value);
    }

    #[test]
    fn signed_decimal_one() {
        let value = SignedDecimal::one();
        assert_eq!(value.0, SignedDecimal::DECIMAL_FRACTIONAL);
    }

    #[test]
    fn signed_decimal_zero() {
        let value = SignedDecimal::zero();
        assert!(value.0.is_zero());
    }

    #[test]
    fn signed_decimal_percent() {
        let value = SignedDecimal::percent(50);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(2u8)
        );

        let value = SignedDecimal::percent(-50);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(-2i8)
        );
    }

    #[test]
    fn signed_decimal_permille() {
        let value = SignedDecimal::permille(125);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(8u8)
        );

        let value = SignedDecimal::permille(-125);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(-8i8)
        );
    }

    #[test]
    fn signed_decimal_bps() {
        let value = SignedDecimal::bps(125);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(80u8)
        );

        let value = SignedDecimal::bps(-125);
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(-80i8)
        );
    }

    #[test]
    fn signed_decimal_from_atomics_works() {
        let one = SignedDecimal::one();
        let two = one + one;
        let neg_one = SignedDecimal::negative_one();

        assert_eq!(SignedDecimal::from_atomics(1i128, 0).unwrap(), one);
        assert_eq!(SignedDecimal::from_atomics(10i128, 1).unwrap(), one);
        assert_eq!(SignedDecimal::from_atomics(100i128, 2).unwrap(), one);
        assert_eq!(SignedDecimal::from_atomics(1000i128, 3).unwrap(), one);
        assert_eq!(
            SignedDecimal::from_atomics(1000000000000000000i128, 18).unwrap(),
            one
        );
        assert_eq!(
            SignedDecimal::from_atomics(10000000000000000000i128, 19).unwrap(),
            one
        );
        assert_eq!(
            SignedDecimal::from_atomics(100000000000000000000i128, 20).unwrap(),
            one
        );

        assert_eq!(SignedDecimal::from_atomics(2i128, 0).unwrap(), two);
        assert_eq!(SignedDecimal::from_atomics(20i128, 1).unwrap(), two);
        assert_eq!(SignedDecimal::from_atomics(200i128, 2).unwrap(), two);
        assert_eq!(SignedDecimal::from_atomics(2000i128, 3).unwrap(), two);
        assert_eq!(
            SignedDecimal::from_atomics(2000000000000000000i128, 18).unwrap(),
            two
        );
        assert_eq!(
            SignedDecimal::from_atomics(20000000000000000000i128, 19).unwrap(),
            two
        );
        assert_eq!(
            SignedDecimal::from_atomics(200000000000000000000i128, 20).unwrap(),
            two
        );

        assert_eq!(SignedDecimal::from_atomics(-1i128, 0).unwrap(), neg_one);
        assert_eq!(SignedDecimal::from_atomics(-10i128, 1).unwrap(), neg_one);
        assert_eq!(
            SignedDecimal::from_atomics(-100000000000000000000i128, 20).unwrap(),
            neg_one
        );

        // Cuts decimal digits (20 provided but only 18 can be stored)
        assert_eq!(
            SignedDecimal::from_atomics(4321i128, 20).unwrap(),
            SignedDecimal::from_str("0.000000000000000043").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(-4321i128, 20).unwrap(),
            SignedDecimal::from_str("-0.000000000000000043").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(6789i128, 20).unwrap(),
            SignedDecimal::from_str("0.000000000000000067").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 38).unwrap(),
            SignedDecimal::from_str("1.701411834604692317").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 39).unwrap(),
            SignedDecimal::from_str("0.170141183460469231").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 45).unwrap(),
            SignedDecimal::from_str("0.000000170141183460").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 51).unwrap(),
            SignedDecimal::from_str("0.000000000000170141").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 56).unwrap(),
            SignedDecimal::from_str("0.000000000000000001").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, 57).unwrap(),
            SignedDecimal::from_str("0.000000000000000000").unwrap()
        );
        assert_eq!(
            SignedDecimal::from_atomics(i128::MAX, u32::MAX).unwrap(),
            SignedDecimal::from_str("0.000000000000000000").unwrap()
        );

        // Can be used with max value
        let max = SignedDecimal::MAX;
        assert_eq!(
            SignedDecimal::from_atomics(max.atomics(), max.decimal_places()).unwrap(),
            max
        );

        // Can be used with min value
        let min = SignedDecimal::MIN;
        assert_eq!(
            SignedDecimal::from_atomics(min.atomics(), min.decimal_places()).unwrap(),
            min
        );

        // Overflow is only possible with digits < 18
        let result = SignedDecimal::from_atomics(i128::MAX, 17);
        assert_eq!(result.unwrap_err(), SignedDecimalRangeExceeded);
    }

    #[test]
    fn signed_decimal_from_ratio_works() {
        // 1.0
        assert_eq!(
            SignedDecimal::from_ratio(1i128, 1i128),
            SignedDecimal::one()
        );
        assert_eq!(
            SignedDecimal::from_ratio(53i128, 53i128),
            SignedDecimal::one()
        );
        assert_eq!(
            SignedDecimal::from_ratio(125i128, 125i128),
            SignedDecimal::one()
        );

        // -1.0
        assert_eq!(
            SignedDecimal::from_ratio(-1i128, 1i128),
            SignedDecimal::negative_one()
        );
        assert_eq!(
            SignedDecimal::from_ratio(-53i128, 53i128),
            SignedDecimal::negative_one()
        );
        assert_eq!(
            SignedDecimal::from_ratio(125i128, -125i128),
            SignedDecimal::negative_one()
        );

        // 1.5
        assert_eq!(
            SignedDecimal::from_ratio(3i128, 2i128),
            SignedDecimal::percent(150)
        );
        assert_eq!(
            SignedDecimal::from_ratio(150i128, 100i128),
            SignedDecimal::percent(150)
        );
        assert_eq!(
            SignedDecimal::from_ratio(333i128, 222i128),
            SignedDecimal::percent(150)
        );

        // 0.125
        assert_eq!(
            SignedDecimal::from_ratio(1i64, 8i64),
            SignedDecimal::permille(125)
        );
        assert_eq!(
            SignedDecimal::from_ratio(125i64, 1000i64),
            SignedDecimal::permille(125)
        );

        // -0.125
        assert_eq!(
            SignedDecimal::from_ratio(-1i64, 8i64),
            SignedDecimal::permille(-125)
        );
        assert_eq!(
            SignedDecimal::from_ratio(125i64, -1000i64),
            SignedDecimal::permille(-125)
        );

        // 1/3 (result floored)
        assert_eq!(
            SignedDecimal::from_ratio(1i64, 3i64),
            SignedDecimal(Int128::from(333_333_333_333_333_333i128))
        );

        // 2/3 (result floored)
        assert_eq!(
            SignedDecimal::from_ratio(2i64, 3i64),
            SignedDecimal(Int128::from(666_666_666_666_666_666i128))
        );

        // large inputs
        assert_eq!(
            SignedDecimal::from_ratio(0i128, i128::MAX),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::from_ratio(i128::MAX, i128::MAX),
            SignedDecimal::one()
        );
        // 170141183460469231731 is the largest integer <= SignedDecimal::MAX
        assert_eq!(
            SignedDecimal::from_ratio(170141183460469231731i128, 1i128),
            SignedDecimal::from_str("170141183460469231731").unwrap()
        );
    }

    #[test]
    #[should_panic(expected = "Denominator must not be zero")]
    fn signed_decimal_from_ratio_panics_for_zero_denominator() {
        SignedDecimal::from_ratio(1i128, 0i128);
    }

    #[test]
    #[should_panic(expected = "Multiplication overflow")]
    fn signed_decimal_from_ratio_panics_for_mul_overflow() {
        SignedDecimal::from_ratio(i128::MAX, 1i128);
    }

    #[test]
    fn signed_decimal_checked_from_ratio_does_not_panic() {
        assert_eq!(
            SignedDecimal::checked_from_ratio(1i128, 0i128),
            Err(CheckedFromRatioError::DivideByZero)
        );

        assert_eq!(
            SignedDecimal::checked_from_ratio(i128::MAX, 1i128),
            Err(CheckedFromRatioError::Overflow)
        );
    }

    #[test]
    fn signed_decimal_implements_fraction() {
        let fraction = SignedDecimal::from_str("1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Int128::from(1_234_567_000_000_000_000_000i128)
        );
        assert_eq!(
            fraction.denominator(),
            Int128::from(1_000_000_000_000_000_000i128)
        );

        let fraction = SignedDecimal::from_str("-1234.567").unwrap();
        assert_eq!(
            fraction.numerator(),
            Int128::from(-1_234_567_000_000_000_000_000i128)
        );
        assert_eq!(
            fraction.denominator(),
            Int128::from(1_000_000_000_000_000_000i128)
        );
    }

    #[test]
    fn signed_decimal_from_str_works() {
        // Integers
        assert_eq!(
            SignedDecimal::from_str("0").unwrap(),
            SignedDecimal::percent(0)
        );
        assert_eq!(
            SignedDecimal::from_str("1").unwrap(),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::from_str("5").unwrap(),
            SignedDecimal::percent(500)
        );
        assert_eq!(
            SignedDecimal::from_str("42").unwrap(),
            SignedDecimal::percent(4200)
        );
        assert_eq!(
            SignedDecimal::from_str("000").unwrap(),
            SignedDecimal::percent(0)
        );
        assert_eq!(
            SignedDecimal::from_str("001").unwrap(),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::from_str("005").unwrap(),
            SignedDecimal::percent(500)
        );
        assert_eq!(
            SignedDecimal::from_str("0042").unwrap(),
            SignedDecimal::percent(4200)
        );

        // Positive decimals
        assert_eq!(
            SignedDecimal::from_str("1.0").unwrap(),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::from_str("1.5").unwrap(),
            SignedDecimal::percent(150)
        );
        assert_eq!(
            SignedDecimal::from_str("0.5").unwrap(),
            SignedDecimal::percent(50)
        );
        assert_eq!(
            SignedDecimal::from_str("0.123").unwrap(),
            SignedDecimal::permille(123)
        );

        assert_eq!(
            SignedDecimal::from_str("40.00").unwrap(),
            SignedDecimal::percent(4000)
        );
        assert_eq!(
            SignedDecimal::from_str("04.00").unwrap(),
            SignedDecimal::percent(400)
        );
        assert_eq!(
            SignedDecimal::from_str("00.40").unwrap(),
            SignedDecimal::percent(40)
        );
        assert_eq!(
            SignedDecimal::from_str("00.04").unwrap(),
            SignedDecimal::percent(4)
        );
        // Negative decimals
        assert_eq!(
            SignedDecimal::from_str("-00.04").unwrap(),
            SignedDecimal::percent(-4)
        );
        assert_eq!(
            SignedDecimal::from_str("-00.40").unwrap(),
            SignedDecimal::percent(-40)
        );
        assert_eq!(
            SignedDecimal::from_str("-04.00").unwrap(),
            SignedDecimal::percent(-400)
        );

        // Can handle DECIMAL_PLACES fractional digits
        assert_eq!(
            SignedDecimal::from_str("7.123456789012345678").unwrap(),
            SignedDecimal(Int128::from(7123456789012345678i128))
        );
        assert_eq!(
            SignedDecimal::from_str("7.999999999999999999").unwrap(),
            SignedDecimal(Int128::from(7999999999999999999i128))
        );

        // Works for documented max value
        assert_eq!(
            SignedDecimal::from_str("170141183460469231731.687303715884105727").unwrap(),
            SignedDecimal::MAX
        );
        // Works for documented min value
        assert_eq!(
            SignedDecimal::from_str("-170141183460469231731.687303715884105728").unwrap(),
            SignedDecimal::MIN
        );
        assert_eq!(
            SignedDecimal::from_str("-1").unwrap(),
            SignedDecimal::negative_one()
        );
    }

    #[test]
    fn signed_decimal_from_str_errors_for_broken_whole_part() {
        let expected_err = StdError::generic_err("Error parsing whole");
        assert_eq!(SignedDecimal::from_str("").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str(" ").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str("-").unwrap_err(), expected_err);
    }

    #[test]
    fn signed_decimal_from_str_errors_for_broken_fractional_part() {
        let expected_err = StdError::generic_err("Error parsing fractional");
        assert_eq!(SignedDecimal::from_str("1.").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str("1. ").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str("1.e").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str("1.2e3").unwrap_err(), expected_err);
        assert_eq!(SignedDecimal::from_str("1.-2").unwrap_err(), expected_err);
    }

    #[test]
    fn signed_decimal_from_str_errors_for_more_than_18_fractional_digits() {
        let expected_err = StdError::generic_err("Cannot parse more than 18 fractional digits");
        assert_eq!(
            SignedDecimal::from_str("7.1234567890123456789").unwrap_err(),
            expected_err
        );
        // No special rules for trailing zeros. This could be changed but adds gas cost for the happy path.
        assert_eq!(
            SignedDecimal::from_str("7.1230000000000000000").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_from_str_errors_for_invalid_number_of_dots() {
        let expected_err = StdError::generic_err("Unexpected number of dots");
        assert_eq!(SignedDecimal::from_str("1.2.3").unwrap_err(), expected_err);
        assert_eq!(
            SignedDecimal::from_str("1.2.3.4").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_from_str_errors_for_more_than_max_value() {
        let expected_err = StdError::generic_err("Value too big");
        // Integer
        assert_eq!(
            SignedDecimal::from_str("170141183460469231732").unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal::from_str("-170141183460469231732").unwrap_err(),
            expected_err
        );

        // SignedDecimal
        assert_eq!(
            SignedDecimal::from_str("170141183460469231732.0").unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal::from_str("170141183460469231731.687303715884105728").unwrap_err(),
            expected_err
        );
        assert_eq!(
            SignedDecimal::from_str("-170141183460469231731.687303715884105729").unwrap_err(),
            expected_err
        );
    }

    #[test]
    fn signed_decimal_conversions_work() {
        // signed decimal to signed decimal
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::MAX).unwrap_err(),
            SignedDecimalRangeExceeded
        );
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::MIN).unwrap_err(),
            SignedDecimalRangeExceeded
        );
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::zero()).unwrap(),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::one()).unwrap(),
            SignedDecimal::one()
        );
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::percent(50)).unwrap(),
            SignedDecimal::percent(50)
        );
        assert_eq!(
            SignedDecimal::try_from(SignedDecimal256::percent(-200)).unwrap(),
            SignedDecimal::percent(-200)
        );

        // unsigned to signed decimal
        assert_eq!(
            SignedDecimal::try_from(Decimal::MAX).unwrap_err(),
            SignedDecimalRangeExceeded
        );
        let max = Decimal::raw(SignedDecimal::MAX.atomics().i128() as u128);
        let too_big = max + Decimal::raw(1);
        assert_eq!(
            SignedDecimal::try_from(too_big).unwrap_err(),
            SignedDecimalRangeExceeded
        );
        assert_eq!(
            SignedDecimal::try_from(Decimal::zero()).unwrap(),
            SignedDecimal::zero()
        );
        assert_eq!(SignedDecimal::try_from(max).unwrap(), SignedDecimal::MAX);
    }

    #[test]
    fn signed_decimal_atomics_works() {
        let zero = SignedDecimal::zero();
        let one = SignedDecimal::one();
        let half = SignedDecimal::percent(50);
        let two = SignedDecimal::percent(200);
        let max = SignedDecimal::MAX;
        let neg_half = SignedDecimal::percent(-50);
        let neg_two = SignedDecimal::percent(-200);
        let min = SignedDecimal::MIN;

        assert_eq!(zero.atomics(), Int128::new(0));
        assert_eq!(one.atomics(), Int128::new(1000000000000000000));
        assert_eq!(half.atomics(), Int128::new(500000000000000000));
        assert_eq!(two.atomics(), Int128::new(2000000000000000000));
        assert_eq!(max.atomics(), Int128::MAX);
        assert_eq!(neg_half.atomics(), Int128::new(-500000000000000000));
        assert_eq!(neg_two.atomics(), Int128::new(-2000000000000000000));
        assert_eq!(min.atomics(), Int128::MIN);
    }

    #[test]
    fn signed_decimal_decimal_places_works() {
        let zero = SignedDecimal::zero();
        let one = SignedDecimal::one();
        let half = SignedDecimal::percent(50);
        let two = SignedDecimal::percent(200);
        let max = SignedDecimal::MAX;
        let neg_one = SignedDecimal::negative_one();

        assert_eq!(zero.decimal_places(), 18);
        assert_eq!(one.decimal_places(), 18);
        assert_eq!(half.decimal_places(), 18);
        assert_eq!(two.decimal_places(), 18);
        assert_eq!(max.decimal_places(), 18);
        assert_eq!(neg_one.decimal_places(), 18);
    }

    #[test]
    fn signed_decimal_is_zero_works() {
        assert!(SignedDecimal::zero().is_zero());
        assert!(SignedDecimal::percent(0).is_zero());
        assert!(SignedDecimal::permille(0).is_zero());

        assert!(!SignedDecimal::one().is_zero());
        assert!(!SignedDecimal::percent(123).is_zero());
        assert!(!SignedDecimal::permille(-1234).is_zero());
    }

    #[test]
    fn signed_decimal_inv_works() {
        // d = 0
        assert_eq!(SignedDecimal::zero().inv(), None);

        // d == 1
        assert_eq!(SignedDecimal::one().inv(), Some(SignedDecimal::one()));

        // d == -1
        assert_eq!(
            SignedDecimal::negative_one().inv(),
            Some(SignedDecimal::negative_one())
        );

        // d > 1 exact
        assert_eq!(
            SignedDecimal::from_str("2").unwrap().inv(),
            Some(SignedDecimal::from_str("0.5").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("20").unwrap().inv(),
            Some(SignedDecimal::from_str("0.05").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("200").unwrap().inv(),
            Some(SignedDecimal::from_str("0.005").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("2000").unwrap().inv(),
            Some(SignedDecimal::from_str("0.0005").unwrap())
        );

        // d > 1 rounded
        assert_eq!(
            SignedDecimal::from_str("3").unwrap().inv(),
            Some(SignedDecimal::from_str("0.333333333333333333").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("6").unwrap().inv(),
            Some(SignedDecimal::from_str("0.166666666666666666").unwrap())
        );

        // d < 1 exact
        assert_eq!(
            SignedDecimal::from_str("0.5").unwrap().inv(),
            Some(SignedDecimal::from_str("2").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("0.05").unwrap().inv(),
            Some(SignedDecimal::from_str("20").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("0.005").unwrap().inv(),
            Some(SignedDecimal::from_str("200").unwrap())
        );
        assert_eq!(
            SignedDecimal::from_str("0.0005").unwrap().inv(),
            Some(SignedDecimal::from_str("2000").unwrap())
        );

        // d < 0
        assert_eq!(
            SignedDecimal::from_str("-0.5").unwrap().inv(),
            Some(SignedDecimal::from_str("-2").unwrap())
        );
        // d < 0 rounded
        assert_eq!(
            SignedDecimal::from_str("-3").unwrap().inv(),
            Some(SignedDecimal::from_str("-0.333333333333333333").unwrap())
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_add_works() {
        let value = SignedDecimal::one() + SignedDecimal::percent(50); // 1.5
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL * Int128::from(3u8) / Int128::from(2u8)
        );

        assert_eq!(
            SignedDecimal::percent(5) + SignedDecimal::percent(4),
            SignedDecimal::percent(9)
        );
        assert_eq!(
            SignedDecimal::percent(5) + SignedDecimal::zero(),
            SignedDecimal::percent(5)
        );
        assert_eq!(
            SignedDecimal::zero() + SignedDecimal::zero(),
            SignedDecimal::zero()
        );
        // negative numbers
        assert_eq!(
            SignedDecimal::percent(-5) + SignedDecimal::percent(-4),
            SignedDecimal::percent(-9)
        );
        assert_eq!(
            SignedDecimal::percent(-5) + SignedDecimal::percent(4),
            SignedDecimal::percent(-1)
        );
        assert_eq!(
            SignedDecimal::percent(5) + SignedDecimal::percent(-4),
            SignedDecimal::percent(1)
        );

        // works for refs
        let a = SignedDecimal::percent(15);
        let b = SignedDecimal::percent(25);
        let expected = SignedDecimal::percent(40);
        assert_eq!(a + b, expected);
        assert_eq!(&a + b, expected);
        assert_eq!(a + &b, expected);
        assert_eq!(&a + &b, expected);
    }

    #[test]
    #[should_panic]
    fn signed_decimal_add_overflow_panics() {
        let _value = SignedDecimal::MAX + SignedDecimal::percent(50);
    }

    #[test]
    fn signed_decimal_add_assign_works() {
        let mut a = SignedDecimal::percent(30);
        a += SignedDecimal::percent(20);
        assert_eq!(a, SignedDecimal::percent(50));

        // works for refs
        let mut a = SignedDecimal::percent(15);
        let b = SignedDecimal::percent(3);
        let expected = SignedDecimal::percent(18);
        a += &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_sub_works() {
        let value = SignedDecimal::one() - SignedDecimal::percent(50); // 0.5
        assert_eq!(
            value.0,
            SignedDecimal::DECIMAL_FRACTIONAL / Int128::from(2u8)
        );

        assert_eq!(
            SignedDecimal::percent(9) - SignedDecimal::percent(4),
            SignedDecimal::percent(5)
        );
        assert_eq!(
            SignedDecimal::percent(16) - SignedDecimal::zero(),
            SignedDecimal::percent(16)
        );
        assert_eq!(
            SignedDecimal::percent(16) - SignedDecimal::percent(16),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::zero() - SignedDecimal::zero(),
            SignedDecimal::zero()
        );

        // negative numbers
        assert_eq!(
            SignedDecimal::percent(-5) - SignedDecimal::percent(-4),
            SignedDecimal::percent(-1)
        );
        assert_eq!(
            SignedDecimal::percent(-5) - SignedDecimal::percent(4),
            SignedDecimal::percent(-9)
        );
        assert_eq!(
            SignedDecimal::percent(500) - SignedDecimal::percent(-4),
            SignedDecimal::percent(504)
        );

        // works for refs
        let a = SignedDecimal::percent(13);
        let b = SignedDecimal::percent(6);
        let expected = SignedDecimal::percent(7);
        assert_eq!(a - b, expected);
        assert_eq!(&a - b, expected);
        assert_eq!(a - &b, expected);
        assert_eq!(&a - &b, expected);
    }

    #[test]
    #[should_panic]
    fn signed_decimal_sub_overflow_panics() {
        let _value = SignedDecimal::MIN - SignedDecimal::percent(50);
    }

    #[test]
    fn signed_decimal_sub_assign_works() {
        let mut a = SignedDecimal::percent(20);
        a -= SignedDecimal::percent(2);
        assert_eq!(a, SignedDecimal::percent(18));

        // works for refs
        let mut a = SignedDecimal::percent(33);
        let b = SignedDecimal::percent(13);
        let expected = SignedDecimal::percent(20);
        a -= &b;
        assert_eq!(a, expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_implements_mul() {
        let one = SignedDecimal::one();
        let two = one + one;
        let half = SignedDecimal::percent(50);

        // 1*x and x*1
        assert_eq!(one * SignedDecimal::percent(0), SignedDecimal::percent(0));
        assert_eq!(one * SignedDecimal::percent(1), SignedDecimal::percent(1));
        assert_eq!(one * SignedDecimal::percent(10), SignedDecimal::percent(10));
        assert_eq!(
            one * SignedDecimal::percent(100),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            one * SignedDecimal::percent(1000),
            SignedDecimal::percent(1000)
        );
        assert_eq!(one * SignedDecimal::MAX, SignedDecimal::MAX);
        assert_eq!(SignedDecimal::percent(0) * one, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) * one, SignedDecimal::percent(1));
        assert_eq!(SignedDecimal::percent(10) * one, SignedDecimal::percent(10));
        assert_eq!(
            SignedDecimal::percent(100) * one,
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::percent(1000) * one,
            SignedDecimal::percent(1000)
        );
        assert_eq!(SignedDecimal::MAX * one, SignedDecimal::MAX);
        assert_eq!(SignedDecimal::percent(-1) * one, SignedDecimal::percent(-1));
        assert_eq!(
            one * SignedDecimal::percent(-10),
            SignedDecimal::percent(-10)
        );

        // double
        assert_eq!(two * SignedDecimal::percent(0), SignedDecimal::percent(0));
        assert_eq!(two * SignedDecimal::percent(1), SignedDecimal::percent(2));
        assert_eq!(two * SignedDecimal::percent(10), SignedDecimal::percent(20));
        assert_eq!(
            two * SignedDecimal::percent(100),
            SignedDecimal::percent(200)
        );
        assert_eq!(
            two * SignedDecimal::percent(1000),
            SignedDecimal::percent(2000)
        );
        assert_eq!(SignedDecimal::percent(0) * two, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) * two, SignedDecimal::percent(2));
        assert_eq!(SignedDecimal::percent(10) * two, SignedDecimal::percent(20));
        assert_eq!(
            SignedDecimal::percent(100) * two,
            SignedDecimal::percent(200)
        );
        assert_eq!(
            SignedDecimal::percent(1000) * two,
            SignedDecimal::percent(2000)
        );
        assert_eq!(SignedDecimal::percent(-1) * two, SignedDecimal::percent(-2));
        assert_eq!(
            two * SignedDecimal::new(Int128::MIN / Int128::new(2)),
            SignedDecimal::MIN
        );

        // half
        assert_eq!(half * SignedDecimal::percent(0), SignedDecimal::percent(0));
        assert_eq!(half * SignedDecimal::percent(1), SignedDecimal::permille(5));
        assert_eq!(half * SignedDecimal::percent(10), SignedDecimal::percent(5));
        assert_eq!(
            half * SignedDecimal::percent(100),
            SignedDecimal::percent(50)
        );
        assert_eq!(
            half * SignedDecimal::percent(1000),
            SignedDecimal::percent(500)
        );
        assert_eq!(SignedDecimal::percent(0) * half, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) * half, SignedDecimal::permille(5));
        assert_eq!(SignedDecimal::percent(10) * half, SignedDecimal::percent(5));
        assert_eq!(
            SignedDecimal::percent(100) * half,
            SignedDecimal::percent(50)
        );
        assert_eq!(
            SignedDecimal::percent(1000) * half,
            SignedDecimal::percent(500)
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
        let max = SignedDecimal::MAX;
        assert_eq!(
            max * dec("1.0"),
            dec("170141183460469231731.687303715884105727")
        );
        assert_eq!(
            max * dec("0.1"),
            dec("17014118346046923173.168730371588410572")
        );
        assert_eq!(
            max * dec("0.01"),
            dec("1701411834604692317.316873037158841057")
        );
        assert_eq!(
            max * dec("0.001"),
            dec("170141183460469231.731687303715884105")
        );
        assert_eq!(
            max * dec("0.000001"),
            dec("170141183460469.231731687303715884")
        );
        assert_eq!(
            max * dec("0.000000001"),
            dec("170141183460.469231731687303715")
        );
        assert_eq!(
            max * dec("0.000000000001"),
            dec("170141183.460469231731687303")
        );
        assert_eq!(
            max * dec("0.000000000000001"),
            dec("170141.183460469231731687")
        );
        assert_eq!(
            max * dec("0.000000000000000001"),
            dec("170.141183460469231731")
        );

        // works for refs
        let a = SignedDecimal::percent(20);
        let b = SignedDecimal::percent(30);
        let expected = SignedDecimal::percent(6);
        assert_eq!(a * b, expected);
        assert_eq!(&a * b, expected);
        assert_eq!(a * &b, expected);
        assert_eq!(&a * &b, expected);
    }

    #[test]
    fn signed_decimal_mul_assign_works() {
        let mut a = SignedDecimal::percent(15);
        a *= SignedDecimal::percent(60);
        assert_eq!(a, SignedDecimal::percent(9));

        // works for refs
        let mut a = SignedDecimal::percent(50);
        let b = SignedDecimal::percent(20);
        a *= &b;
        assert_eq!(a, SignedDecimal::percent(10));
    }

    #[test]
    #[should_panic(expected = "attempt to multiply with overflow")]
    fn signed_decimal_mul_overflow_panics() {
        let _value = SignedDecimal::MAX * SignedDecimal::percent(101);
    }

    #[test]
    fn signed_decimal_checked_mul() {
        let test_data = [
            (SignedDecimal::zero(), SignedDecimal::zero()),
            (SignedDecimal::zero(), SignedDecimal::one()),
            (SignedDecimal::one(), SignedDecimal::zero()),
            (SignedDecimal::percent(10), SignedDecimal::zero()),
            (SignedDecimal::percent(10), SignedDecimal::percent(5)),
            (SignedDecimal::MAX, SignedDecimal::one()),
            (
                SignedDecimal::MAX / Int128::new(2),
                SignedDecimal::percent(200),
            ),
            (SignedDecimal::permille(6), SignedDecimal::permille(13)),
            (SignedDecimal::permille(-6), SignedDecimal::permille(0)),
            (SignedDecimal::MAX, SignedDecimal::negative_one()),
        ];

        // The regular core::ops::Mul is our source of truth for these tests.
        for (x, y) in test_data.into_iter() {
            assert_eq!(x * y, x.checked_mul(y).unwrap());
        }
    }

    #[test]
    fn signed_decimal_checked_mul_overflow() {
        assert_eq!(
            SignedDecimal::MAX.checked_mul(SignedDecimal::percent(200)),
            Err(OverflowError {
                operation: OverflowOperation::Mul,
                operand1: SignedDecimal::MAX.to_string(),
                operand2: SignedDecimal::percent(200).to_string(),
            })
        );
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_implements_div() {
        let one = SignedDecimal::one();
        let two = one + one;
        let half = SignedDecimal::percent(50);

        // 1/x and x/1
        assert_eq!(
            one / SignedDecimal::percent(1),
            SignedDecimal::percent(10_000)
        );
        assert_eq!(
            one / SignedDecimal::percent(10),
            SignedDecimal::percent(1_000)
        );
        assert_eq!(
            one / SignedDecimal::percent(100),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            one / SignedDecimal::percent(1000),
            SignedDecimal::percent(10)
        );
        assert_eq!(SignedDecimal::percent(0) / one, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) / one, SignedDecimal::percent(1));
        assert_eq!(SignedDecimal::percent(10) / one, SignedDecimal::percent(10));
        assert_eq!(
            SignedDecimal::percent(100) / one,
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::percent(1000) / one,
            SignedDecimal::percent(1000)
        );
        assert_eq!(
            one / SignedDecimal::percent(-1),
            SignedDecimal::percent(-10_000)
        );
        assert_eq!(
            one / SignedDecimal::percent(-10),
            SignedDecimal::percent(-1_000)
        );

        // double
        assert_eq!(
            two / SignedDecimal::percent(1),
            SignedDecimal::percent(20_000)
        );
        assert_eq!(
            two / SignedDecimal::percent(10),
            SignedDecimal::percent(2_000)
        );
        assert_eq!(
            two / SignedDecimal::percent(100),
            SignedDecimal::percent(200)
        );
        assert_eq!(
            two / SignedDecimal::percent(1000),
            SignedDecimal::percent(20)
        );
        assert_eq!(SignedDecimal::percent(0) / two, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) / two, dec("0.005"));
        assert_eq!(SignedDecimal::percent(10) / two, SignedDecimal::percent(5));
        assert_eq!(
            SignedDecimal::percent(100) / two,
            SignedDecimal::percent(50)
        );
        assert_eq!(
            SignedDecimal::percent(1000) / two,
            SignedDecimal::percent(500)
        );
        assert_eq!(
            two / SignedDecimal::percent(-1),
            SignedDecimal::percent(-20_000)
        );
        assert_eq!(
            SignedDecimal::percent(-10000) / two,
            SignedDecimal::percent(-5000)
        );

        // half
        assert_eq!(
            half / SignedDecimal::percent(1),
            SignedDecimal::percent(5_000)
        );
        assert_eq!(
            half / SignedDecimal::percent(10),
            SignedDecimal::percent(500)
        );
        assert_eq!(
            half / SignedDecimal::percent(100),
            SignedDecimal::percent(50)
        );
        assert_eq!(
            half / SignedDecimal::percent(1000),
            SignedDecimal::percent(5)
        );
        assert_eq!(SignedDecimal::percent(0) / half, SignedDecimal::percent(0));
        assert_eq!(SignedDecimal::percent(1) / half, SignedDecimal::percent(2));
        assert_eq!(
            SignedDecimal::percent(10) / half,
            SignedDecimal::percent(20)
        );
        assert_eq!(
            SignedDecimal::percent(100) / half,
            SignedDecimal::percent(200)
        );
        assert_eq!(
            SignedDecimal::percent(1000) / half,
            SignedDecimal::percent(2000)
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
            SignedDecimal::percent(15) / SignedDecimal::percent(60),
            SignedDecimal::percent(25)
        );

        // works for refs
        let a = SignedDecimal::percent(100);
        let b = SignedDecimal::percent(20);
        let expected = SignedDecimal::percent(500);
        assert_eq!(a / b, expected);
        assert_eq!(&a / b, expected);
        assert_eq!(a / &b, expected);
        assert_eq!(&a / &b, expected);
    }

    #[test]
    fn signed_decimal_div_assign_works() {
        let mut a = SignedDecimal::percent(15);
        a /= SignedDecimal::percent(20);
        assert_eq!(a, SignedDecimal::percent(75));

        // works for refs
        let mut a = SignedDecimal::percent(50);
        let b = SignedDecimal::percent(20);
        a /= &b;
        assert_eq!(a, SignedDecimal::percent(250));
    }

    #[test]
    #[should_panic(expected = "Division failed - multiplication overflow")]
    fn signed_decimal_div_overflow_panics() {
        let _value = SignedDecimal::MAX / SignedDecimal::percent(10);
    }

    #[test]
    #[should_panic(expected = "Division failed - denominator must not be zero")]
    fn signed_decimal_div_by_zero_panics() {
        let _value = SignedDecimal::one() / SignedDecimal::zero();
    }

    #[test]
    fn signed_decimal_int128_division() {
        // a/b
        let left = SignedDecimal::percent(150); // 1.5
        let right = Int128::new(3);
        assert_eq!(left / right, SignedDecimal::percent(50));

        // negative
        let left = SignedDecimal::percent(-150); // -1.5
        let right = Int128::new(3);
        assert_eq!(left / right, SignedDecimal::percent(-50));

        // 0/a
        let left = SignedDecimal::zero();
        let right = Int128::new(300);
        assert_eq!(left / right, SignedDecimal::zero());
    }

    #[test]
    #[should_panic]
    fn signed_decimal_int128_divide_by_zero() {
        let left = SignedDecimal::percent(150); // 1.5
        let right = Int128::new(0);
        let _result = left / right;
    }

    #[test]
    fn signed_decimal_int128_div_assign() {
        // a/b
        let mut dec = SignedDecimal::percent(150); // 1.5
        dec /= Int128::new(3);
        assert_eq!(dec, SignedDecimal::percent(50));

        // 0/a
        let mut dec = SignedDecimal::zero();
        dec /= Int128::new(300);
        assert_eq!(dec, SignedDecimal::zero());
    }

    #[test]
    #[should_panic]
    fn signed_decimal_int128_div_assign_by_zero() {
        // a/0
        let mut dec = SignedDecimal::percent(50);
        dec /= Int128::new(0);
    }

    #[test]
    fn signed_decimal_checked_pow() {
        for exp in 0..10 {
            assert_eq!(
                SignedDecimal::one().checked_pow(exp).unwrap(),
                SignedDecimal::one()
            );
        }

        // This case is mathematically undefined but we ensure consistency with Rust standard types
        // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=20df6716048e77087acd40194b233494
        assert_eq!(
            SignedDecimal::zero().checked_pow(0).unwrap(),
            SignedDecimal::one()
        );

        for exp in 1..10 {
            assert_eq!(
                SignedDecimal::zero().checked_pow(exp).unwrap(),
                SignedDecimal::zero()
            );
        }

        for exp in 1..10 {
            assert_eq!(
                SignedDecimal::negative_one().checked_pow(exp).unwrap(),
                // alternates between 1 and -1
                if exp % 2 == 0 {
                    SignedDecimal::one()
                } else {
                    SignedDecimal::negative_one()
                }
            )
        }

        for num in &[
            SignedDecimal::percent(50),
            SignedDecimal::percent(99),
            SignedDecimal::percent(200),
        ] {
            assert_eq!(num.checked_pow(0).unwrap(), SignedDecimal::one())
        }

        assert_eq!(
            SignedDecimal::percent(20).checked_pow(2).unwrap(),
            SignedDecimal::percent(4)
        );

        assert_eq!(
            SignedDecimal::percent(20).checked_pow(3).unwrap(),
            SignedDecimal::permille(8)
        );

        assert_eq!(
            SignedDecimal::percent(200).checked_pow(4).unwrap(),
            SignedDecimal::percent(1600)
        );

        assert_eq!(
            SignedDecimal::percent(200).checked_pow(4).unwrap(),
            SignedDecimal::percent(1600)
        );

        assert_eq!(
            SignedDecimal::percent(700).checked_pow(5).unwrap(),
            SignedDecimal::percent(1680700)
        );

        assert_eq!(
            SignedDecimal::percent(700).checked_pow(8).unwrap(),
            SignedDecimal::percent(576480100)
        );

        assert_eq!(
            SignedDecimal::percent(700).checked_pow(10).unwrap(),
            SignedDecimal::percent(28247524900)
        );

        assert_eq!(
            SignedDecimal::percent(120).checked_pow(123).unwrap(),
            SignedDecimal(5486473221892422150877397607i128.into())
        );

        assert_eq!(
            SignedDecimal::percent(10).checked_pow(2).unwrap(),
            SignedDecimal(10000000000000000i128.into())
        );

        assert_eq!(
            SignedDecimal::percent(10).checked_pow(18).unwrap(),
            SignedDecimal(1i128.into())
        );

        let decimals = [
            SignedDecimal::percent(-50),
            SignedDecimal::percent(-99),
            SignedDecimal::percent(-200),
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
    fn signed_decimal_checked_pow_overflow() {
        assert_eq!(
            SignedDecimal::MAX.checked_pow(2),
            Err(OverflowError {
                operation: OverflowOperation::Pow,
                operand1: SignedDecimal::MAX.to_string(),
                operand2: "2".to_string(),
            })
        );
    }

    #[test]
    fn signed_decimal_to_string() {
        // Integers
        assert_eq!(SignedDecimal::zero().to_string(), "0");
        assert_eq!(SignedDecimal::one().to_string(), "1");
        assert_eq!(SignedDecimal::percent(500).to_string(), "5");
        assert_eq!(SignedDecimal::percent(-500).to_string(), "-5");

        // SignedDecimals
        assert_eq!(SignedDecimal::percent(125).to_string(), "1.25");
        assert_eq!(SignedDecimal::percent(42638).to_string(), "426.38");
        assert_eq!(SignedDecimal::percent(3).to_string(), "0.03");
        assert_eq!(SignedDecimal::permille(987).to_string(), "0.987");
        assert_eq!(SignedDecimal::percent(-125).to_string(), "-1.25");
        assert_eq!(SignedDecimal::percent(-42638).to_string(), "-426.38");
        assert_eq!(SignedDecimal::percent(-3).to_string(), "-0.03");
        assert_eq!(SignedDecimal::permille(-987).to_string(), "-0.987");

        assert_eq!(
            SignedDecimal(Int128::from(1i128)).to_string(),
            "0.000000000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10i128)).to_string(),
            "0.00000000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100i128)).to_string(),
            "0.0000000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(1000i128)).to_string(),
            "0.000000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10000i128)).to_string(),
            "0.00000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100000i128)).to_string(),
            "0.0000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(1000000i128)).to_string(),
            "0.000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10000000i128)).to_string(),
            "0.00000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100000000i128)).to_string(),
            "0.0000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(1000000000i128)).to_string(),
            "0.000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10000000000i128)).to_string(),
            "0.00000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100000000000i128)).to_string(),
            "0.0000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10000000000000i128)).to_string(),
            "0.00001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100000000000000i128)).to_string(),
            "0.0001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(1000000000000000i128)).to_string(),
            "0.001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(10000000000000000i128)).to_string(),
            "0.01"
        );
        assert_eq!(
            SignedDecimal(Int128::from(100000000000000000i128)).to_string(),
            "0.1"
        );
        assert_eq!(
            SignedDecimal(Int128::from(-1i128)).to_string(),
            "-0.000000000000000001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(-100000000000000i128)).to_string(),
            "-0.0001"
        );
        assert_eq!(
            SignedDecimal(Int128::from(-100000000000000000i128)).to_string(),
            "-0.1"
        );
    }

    #[test]
    fn signed_decimal_iter_sum() {
        let items = vec![
            SignedDecimal::zero(),
            SignedDecimal(Int128::from(2i128)),
            SignedDecimal(Int128::from(2i128)),
            SignedDecimal(Int128::from(-2i128)),
        ];
        assert_eq!(
            items.iter().sum::<SignedDecimal>(),
            SignedDecimal(Int128::from(2i128))
        );
        assert_eq!(
            items.into_iter().sum::<SignedDecimal>(),
            SignedDecimal(Int128::from(2i128))
        );

        let empty: Vec<SignedDecimal> = vec![];
        assert_eq!(SignedDecimal::zero(), empty.iter().sum::<SignedDecimal>());
    }

    #[test]
    fn signed_decimal_serialize() {
        assert_eq!(to_json_vec(&SignedDecimal::zero()).unwrap(), br#""0""#);
        assert_eq!(to_json_vec(&SignedDecimal::one()).unwrap(), br#""1""#);
        assert_eq!(
            to_json_vec(&SignedDecimal::percent(8)).unwrap(),
            br#""0.08""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal::percent(87)).unwrap(),
            br#""0.87""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal::percent(876)).unwrap(),
            br#""8.76""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal::percent(8765)).unwrap(),
            br#""87.65""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal::percent(-87654)).unwrap(),
            br#""-876.54""#
        );
        assert_eq!(
            to_json_vec(&SignedDecimal::negative_one()).unwrap(),
            br#""-1""#
        );
        assert_eq!(
            to_json_vec(&-SignedDecimal::percent(8)).unwrap(),
            br#""-0.08""#
        );
    }

    #[test]
    fn signed_decimal_deserialize() {
        assert_eq!(
            from_json::<SignedDecimal>(br#""0""#).unwrap(),
            SignedDecimal::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""1""#).unwrap(),
            SignedDecimal::one()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""000""#).unwrap(),
            SignedDecimal::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""001""#).unwrap(),
            SignedDecimal::one()
        );

        assert_eq!(
            from_json::<SignedDecimal>(br#""0.08""#).unwrap(),
            SignedDecimal::percent(8)
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""0.87""#).unwrap(),
            SignedDecimal::percent(87)
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""8.76""#).unwrap(),
            SignedDecimal::percent(876)
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""87.65""#).unwrap(),
            SignedDecimal::percent(8765)
        );

        // negative numbers
        assert_eq!(
            from_json::<SignedDecimal>(br#""-0""#).unwrap(),
            SignedDecimal::zero()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""-1""#).unwrap(),
            SignedDecimal::negative_one()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""-001""#).unwrap(),
            SignedDecimal::negative_one()
        );
        assert_eq!(
            from_json::<SignedDecimal>(br#""-0.08""#).unwrap(),
            SignedDecimal::percent(-8)
        );
    }

    #[test]
    fn signed_decimal_abs_diff_works() {
        let a = SignedDecimal::percent(285);
        let b = SignedDecimal::percent(200);
        let expected = Decimal::percent(85);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let a = SignedDecimal::percent(-200);
        let b = SignedDecimal::percent(200);
        let expected = Decimal::percent(400);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);

        let a = SignedDecimal::percent(-200);
        let b = SignedDecimal::percent(-240);
        let expected = Decimal::percent(40);
        assert_eq!(a.abs_diff(b), expected);
        assert_eq!(b.abs_diff(a), expected);
    }

    #[test]
    #[allow(clippy::op_ref)]
    fn signed_decimal_rem_works() {
        // 4.02 % 1.11 = 0.69
        assert_eq!(
            SignedDecimal::percent(402) % SignedDecimal::percent(111),
            SignedDecimal::percent(69)
        );

        // 15.25 % 4 = 3.25
        assert_eq!(
            SignedDecimal::percent(1525) % SignedDecimal::percent(400),
            SignedDecimal::percent(325)
        );

        // -20.25 % 5 = -25
        assert_eq!(
            SignedDecimal::percent(-2025) % SignedDecimal::percent(500),
            SignedDecimal::percent(-25)
        );

        let a = SignedDecimal::percent(318);
        let b = SignedDecimal::percent(317);
        let expected = SignedDecimal::percent(1);
        assert_eq!(a % b, expected);
        assert_eq!(a % &b, expected);
        assert_eq!(&a % b, expected);
        assert_eq!(&a % &b, expected);
    }

    #[test]
    fn signed_decimal_rem_assign_works() {
        let mut a = SignedDecimal::percent(17673);
        a %= SignedDecimal::percent(2362);
        assert_eq!(a, SignedDecimal::percent(1139)); // 176.73 % 23.62 = 11.39

        let mut a = SignedDecimal::percent(4262);
        let b = SignedDecimal::percent(1270);
        a %= &b;
        assert_eq!(a, SignedDecimal::percent(452)); // 42.62 % 12.7 = 4.52

        let mut a = SignedDecimal::percent(-4262);
        let b = SignedDecimal::percent(1270);
        a %= &b;
        assert_eq!(a, SignedDecimal::percent(-452)); // -42.62 % 12.7 = -4.52
    }

    #[test]
    #[should_panic(expected = "divisor of zero")]
    fn signed_decimal_rem_panics_for_zero() {
        let _ = SignedDecimal::percent(777) % SignedDecimal::zero();
    }

    #[test]
    fn signed_decimal_checked_methods() {
        // checked add
        assert_eq!(
            SignedDecimal::percent(402)
                .checked_add(SignedDecimal::percent(111))
                .unwrap(),
            SignedDecimal::percent(513)
        );
        assert!(matches!(
            SignedDecimal::MAX.checked_add(SignedDecimal::percent(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            SignedDecimal::MIN.checked_add(SignedDecimal::percent(-1)),
            Err(OverflowError { .. })
        ));

        // checked sub
        assert_eq!(
            SignedDecimal::percent(1111)
                .checked_sub(SignedDecimal::percent(111))
                .unwrap(),
            SignedDecimal::percent(1000)
        );
        assert_eq!(
            SignedDecimal::zero()
                .checked_sub(SignedDecimal::percent(1))
                .unwrap(),
            SignedDecimal::percent(-1)
        );
        assert!(matches!(
            SignedDecimal::MIN.checked_sub(SignedDecimal::percent(1)),
            Err(OverflowError { .. })
        ));
        assert!(matches!(
            SignedDecimal::MAX.checked_sub(SignedDecimal::percent(-1)),
            Err(OverflowError { .. })
        ));

        // checked div
        assert_eq!(
            SignedDecimal::percent(30)
                .checked_div(SignedDecimal::percent(200))
                .unwrap(),
            SignedDecimal::percent(15)
        );
        assert_eq!(
            SignedDecimal::percent(88)
                .checked_div(SignedDecimal::percent(20))
                .unwrap(),
            SignedDecimal::percent(440)
        );
        assert!(matches!(
            SignedDecimal::MAX.checked_div(SignedDecimal::zero()),
            Err(CheckedFromRatioError::DivideByZero {})
        ));
        assert!(matches!(
            SignedDecimal::MAX.checked_div(SignedDecimal::percent(1)),
            Err(CheckedFromRatioError::Overflow {})
        ));
        assert_eq!(
            SignedDecimal::percent(-88)
                .checked_div(SignedDecimal::percent(20))
                .unwrap(),
            SignedDecimal::percent(-440)
        );
        assert_eq!(
            SignedDecimal::percent(-88)
                .checked_div(SignedDecimal::percent(-20))
                .unwrap(),
            SignedDecimal::percent(440)
        );

        // checked rem
        assert_eq!(
            SignedDecimal::percent(402)
                .checked_rem(SignedDecimal::percent(111))
                .unwrap(),
            SignedDecimal::percent(69)
        );
        assert_eq!(
            SignedDecimal::percent(1525)
                .checked_rem(SignedDecimal::percent(400))
                .unwrap(),
            SignedDecimal::percent(325)
        );
        assert_eq!(
            SignedDecimal::percent(-1525)
                .checked_rem(SignedDecimal::percent(400))
                .unwrap(),
            SignedDecimal::percent(-325)
        );
        assert_eq!(
            SignedDecimal::percent(-1525)
                .checked_rem(SignedDecimal::percent(-400))
                .unwrap(),
            SignedDecimal::percent(-325)
        );
        assert!(matches!(
            SignedDecimal::MAX.checked_rem(SignedDecimal::zero()),
            Err(DivideByZeroError { .. })
        ));
    }

    #[test]
    fn signed_decimal_pow_works() {
        assert_eq!(
            SignedDecimal::percent(200).pow(2),
            SignedDecimal::percent(400)
        );
        assert_eq!(
            SignedDecimal::percent(-200).pow(2),
            SignedDecimal::percent(400)
        );
        assert_eq!(
            SignedDecimal::percent(-200).pow(3),
            SignedDecimal::percent(-800)
        );
        assert_eq!(
            SignedDecimal::percent(200).pow(10),
            SignedDecimal::percent(102400)
        );
    }

    #[test]
    #[should_panic]
    fn signed_decimal_pow_overflow_panics() {
        _ = SignedDecimal::MAX.pow(2u32);
    }

    #[test]
    fn signed_decimal_saturating_works() {
        assert_eq!(
            SignedDecimal::percent(200).saturating_add(SignedDecimal::percent(200)),
            SignedDecimal::percent(400)
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_add(SignedDecimal::percent(200)),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_add(SignedDecimal::percent(-200)),
            SignedDecimal::percent(-400)
        );
        assert_eq!(
            SignedDecimal::MAX.saturating_add(SignedDecimal::percent(200)),
            SignedDecimal::MAX
        );
        assert_eq!(
            SignedDecimal::MIN.saturating_add(SignedDecimal::percent(-1)),
            SignedDecimal::MIN
        );
        assert_eq!(
            SignedDecimal::percent(200).saturating_sub(SignedDecimal::percent(100)),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_sub(SignedDecimal::percent(100)),
            SignedDecimal::percent(-300)
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_sub(SignedDecimal::percent(-100)),
            SignedDecimal::percent(-100)
        );
        assert_eq!(
            SignedDecimal::zero().saturating_sub(SignedDecimal::percent(200)),
            SignedDecimal::from_str("-2").unwrap()
        );
        assert_eq!(
            SignedDecimal::MIN.saturating_sub(SignedDecimal::percent(200)),
            SignedDecimal::MIN
        );
        assert_eq!(
            SignedDecimal::MAX.saturating_sub(SignedDecimal::percent(-200)),
            SignedDecimal::MAX
        );
        assert_eq!(
            SignedDecimal::percent(200).saturating_mul(SignedDecimal::percent(50)),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_mul(SignedDecimal::percent(50)),
            SignedDecimal::percent(-100)
        );
        assert_eq!(
            SignedDecimal::percent(-200).saturating_mul(SignedDecimal::percent(-50)),
            SignedDecimal::percent(100)
        );
        assert_eq!(
            SignedDecimal::MAX.saturating_mul(SignedDecimal::percent(200)),
            SignedDecimal::MAX
        );
        assert_eq!(
            SignedDecimal::MIN.saturating_mul(SignedDecimal::percent(200)),
            SignedDecimal::MIN
        );
        assert_eq!(
            SignedDecimal::MIN.saturating_mul(SignedDecimal::percent(-200)),
            SignedDecimal::MAX
        );
        assert_eq!(
            SignedDecimal::percent(400).saturating_pow(2u32),
            SignedDecimal::percent(1600)
        );
        assert_eq!(SignedDecimal::MAX.saturating_pow(2u32), SignedDecimal::MAX);
        assert_eq!(SignedDecimal::MAX.saturating_pow(3u32), SignedDecimal::MAX);
        assert_eq!(SignedDecimal::MIN.saturating_pow(2u32), SignedDecimal::MAX);
        assert_eq!(SignedDecimal::MIN.saturating_pow(3u32), SignedDecimal::MIN);
    }

    #[test]
    fn signed_decimal_rounding() {
        assert_eq!(SignedDecimal::one().floor(), SignedDecimal::one());
        assert_eq!(SignedDecimal::percent(150).floor(), SignedDecimal::one());
        assert_eq!(SignedDecimal::percent(199).floor(), SignedDecimal::one());
        assert_eq!(
            SignedDecimal::percent(200).floor(),
            SignedDecimal::percent(200)
        );
        assert_eq!(SignedDecimal::percent(99).floor(), SignedDecimal::zero());
        assert_eq!(
            SignedDecimal(Int128::from(1i128)).floor(),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal(Int128::from(-1i128)).floor(),
            SignedDecimal::negative_one()
        );
        assert_eq!(
            SignedDecimal::permille(-1234).floor(),
            SignedDecimal::percent(-200)
        );

        assert_eq!(SignedDecimal::one().ceil(), SignedDecimal::one());
        assert_eq!(
            SignedDecimal::percent(150).ceil(),
            SignedDecimal::percent(200)
        );
        assert_eq!(
            SignedDecimal::percent(199).ceil(),
            SignedDecimal::percent(200)
        );
        assert_eq!(SignedDecimal::percent(99).ceil(), SignedDecimal::one());
        assert_eq!(
            SignedDecimal(Int128::from(1i128)).ceil(),
            SignedDecimal::one()
        );
        assert_eq!(
            SignedDecimal(Int128::from(-1i128)).ceil(),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::permille(-1234).ceil(),
            SignedDecimal::negative_one()
        );

        assert_eq!(SignedDecimal::one().trunc(), SignedDecimal::one());
        assert_eq!(SignedDecimal::percent(150).trunc(), SignedDecimal::one());
        assert_eq!(SignedDecimal::percent(199).trunc(), SignedDecimal::one());
        assert_eq!(
            SignedDecimal::percent(200).trunc(),
            SignedDecimal::percent(200)
        );
        assert_eq!(SignedDecimal::percent(99).trunc(), SignedDecimal::zero());
        assert_eq!(
            SignedDecimal(Int128::from(1i128)).trunc(),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal(Int128::from(-1i128)).trunc(),
            SignedDecimal::zero()
        );
        assert_eq!(
            SignedDecimal::permille(-1234).trunc(),
            SignedDecimal::negative_one()
        );
    }

    #[test]
    #[should_panic(expected = "attempt to ceil with overflow")]
    fn signed_decimal_ceil_panics() {
        let _ = SignedDecimal::MAX.ceil();
    }

    #[test]
    #[should_panic(expected = "attempt to floor with overflow")]
    fn signed_decimal_floor_panics() {
        let _ = SignedDecimal::MIN.floor();
    }

    #[test]
    fn signed_decimal_checked_ceil() {
        assert_eq!(
            SignedDecimal::percent(199).checked_ceil(),
            Ok(SignedDecimal::percent(200))
        );
        assert_eq!(SignedDecimal::MAX.checked_ceil(), Err(RoundUpOverflowError));
    }

    #[test]
    fn signed_decimal_checked_floor() {
        assert_eq!(
            SignedDecimal::percent(199).checked_floor(),
            Ok(SignedDecimal::one())
        );
        assert_eq!(
            SignedDecimal::percent(-199).checked_floor(),
            Ok(SignedDecimal::percent(-200))
        );
        assert_eq!(
            SignedDecimal::MIN.checked_floor(),
            Err(RoundDownOverflowError)
        );
        assert_eq!(
            SignedDecimal::negative_one().checked_floor(),
            Ok(SignedDecimal::negative_one())
        );
    }

    #[test]
    fn signed_decimal_to_int_floor_works() {
        let d = SignedDecimal::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(12));
        let d = SignedDecimal::from_str("12.345").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(12));
        let d = SignedDecimal::from_str("12.999").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(12));
        let d = SignedDecimal::from_str("0.98451384").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(0));
        let d = SignedDecimal::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(-13));
        let d = SignedDecimal::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(-13));
        let d = SignedDecimal::from_str("75.0").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(75));
        let d = SignedDecimal::from_str("0.0001").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(0));
        let d = SignedDecimal::from_str("0.0").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(0));
        let d = SignedDecimal::from_str("-0.0").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(0));
        let d = SignedDecimal::from_str("-0.0001").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(-1));
        let d = SignedDecimal::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_floor(), Int128::new(-75));
        let d = SignedDecimal::MAX;
        assert_eq!(d.to_int_floor(), Int128::new(170141183460469231731));
        let d = SignedDecimal::MIN;
        assert_eq!(d.to_int_floor(), Int128::new(-170141183460469231732));
    }

    #[test]
    fn signed_decimal_to_int_ceil_works() {
        let d = SignedDecimal::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(13));
        let d = SignedDecimal::from_str("12.345").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(13));
        let d = SignedDecimal::from_str("12.999").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(13));
        let d = SignedDecimal::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(-12));
        let d = SignedDecimal::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(-12));

        let d = SignedDecimal::from_str("75.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(75));
        let d = SignedDecimal::from_str("0.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(0));
        let d = SignedDecimal::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_ceil(), Int128::new(-75));

        let d = SignedDecimal::MAX;
        assert_eq!(d.to_int_ceil(), Int128::new(170141183460469231732));
        let d = SignedDecimal::MIN;
        assert_eq!(d.to_int_ceil(), Int128::new(-170141183460469231731));
    }

    #[test]
    fn signed_decimal_to_int_trunc_works() {
        let d = SignedDecimal::from_str("12.000000000000000001").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(12));
        let d = SignedDecimal::from_str("12.345").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(12));
        let d = SignedDecimal::from_str("12.999").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(12));
        let d = SignedDecimal::from_str("-12.000000000000000001").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(-12));
        let d = SignedDecimal::from_str("-12.345").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(-12));

        let d = SignedDecimal::from_str("75.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(75));
        let d = SignedDecimal::from_str("0.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(0));
        let d = SignedDecimal::from_str("-75.0").unwrap();
        assert_eq!(d.to_int_trunc(), Int128::new(-75));

        let d = SignedDecimal::MAX;
        assert_eq!(d.to_int_trunc(), Int128::new(170141183460469231731));
        let d = SignedDecimal::MIN;
        assert_eq!(d.to_int_trunc(), Int128::new(-170141183460469231731));
    }

    #[test]
    fn signed_decimal_neg_works() {
        assert_eq!(-SignedDecimal::percent(50), SignedDecimal::percent(-50));
        assert_eq!(-SignedDecimal::one(), SignedDecimal::negative_one());
    }

    #[test]
    fn signed_decimal_partial_eq() {
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
    fn signed_decimal_implements_debug() {
        let decimal = SignedDecimal::from_str("123.45").unwrap();
        assert_eq!(format!("{decimal:?}"), "SignedDecimal(123.45)");

        let test_cases = ["5", "5.01", "42", "0", "2", "-0.000001"];
        for s in test_cases {
            let decimal = SignedDecimal::from_str(s).unwrap();
            let expected = format!("SignedDecimal({s})");
            assert_eq!(format!("{decimal:?}"), expected);
        }
    }

    #[test]
    fn signed_decimal_can_be_instantiated_from_decimal256() {
        let d: SignedDecimal = Decimal256::zero().try_into().unwrap();
        assert_eq!(d, SignedDecimal::zero());
    }

    #[test]
    fn signed_decimal_may_fail_when_instantiated_from_decimal256() {
        let err = <Decimal256 as TryInto<SignedDecimal>>::try_into(Decimal256::MAX).unwrap_err();
        assert_eq!("SignedDecimalRangeExceeded", format!("{err:?}"));
        assert_eq!("SignedDecimal range exceeded", format!("{err}"));
    }

    #[test]
    fn signed_decimal_can_be_serialized_and_deserialized() {
        // properly deserialized
        let value: SignedDecimal = serde_json::from_str(r#""123""#).unwrap();
        assert_eq!(SignedDecimal::from_str("123").unwrap(), value);

        // properly serialized
        let value = SignedDecimal::from_str("456").unwrap();
        assert_eq!(r#""456""#, serde_json::to_string(&value).unwrap());

        // invalid: not a string encoded decimal
        assert_eq!(
            "invalid type: integer `123`, expected string-encoded decimal at line 1 column 3",
            serde_json::from_str::<SignedDecimal>("123")
                .err()
                .unwrap()
                .to_string()
        );

        // invalid: not properly defined signed decimal value
        assert_eq!(
            "Error parsing decimal '1.e': Generic error: Error parsing fractional at line 1 column 5",
            serde_json::from_str::<SignedDecimal>(r#""1.e""#)
                .err()
                .unwrap()
                .to_string()
        );
    }

    #[test]
    fn signed_decimal_has_defined_json_schema() {
        let schema = schema_for!(SignedDecimal);
        assert_eq!(
            "SignedDecimal",
            schema.schema.metadata.unwrap().title.unwrap()
        );
    }
}
