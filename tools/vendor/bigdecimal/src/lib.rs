// Copyright 2016 Adam Sunderland
//           2016-2017 Andrew Kubera
//           2017 Ruben De Smet
// See the COPYRIGHT file at the top-level directory of this
// distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A Big Decimal
//!
//! `BigDecimal` allows storing any real number to arbitrary precision; which
//! avoids common floating point errors (such as 0.1 + 0.2 ≠ 0.3) at the
//! cost of complexity.
//!
//! Internally, `BigDecimal` uses a `BigInt` object, paired with a 64-bit
//! integer which determines the position of the decimal point. Therefore,
//! the precision *is not* actually arbitrary, but limited to 2<sup>63</sup>
//! decimal places.
//!
//! Common numerical operations are overloaded, so we can treat them
//! the same way we treat other numbers.
//!
//! It is not recommended to convert a floating point number to a decimal
//! directly, as the floating point representation may be unexpected.
//!
//! # Example
//!
//! ```
//! use bigdecimal::BigDecimal;
//! use std::str::FromStr;
//!
//! let input = "0.8";
//! let dec = BigDecimal::from_str(&input).unwrap();
//! let float = f32::from_str(&input).unwrap();
//!
//! println!("Input ({}) with 10 decimals: {} vs {})", input, dec, float);
//! ```
#![allow(clippy::unreadable_literal)]
#![allow(clippy::needless_return)]
#![allow(clippy::suspicious_arithmetic_impl)]
#![allow(clippy::suspicious_op_assign_impl)]
#![allow(clippy::redundant_field_names)]

pub extern crate num_bigint;
pub extern crate num_traits;
extern crate num_integer;

#[cfg(feature = "serde")]
extern crate serde;

use std::cmp::Ordering;
use std::convert::TryFrom;
use std::default::Default;
use std::error::Error;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::num::{ParseFloatError, ParseIntError};
use std::ops::{Add, AddAssign, Div, Mul, MulAssign, Neg, Rem, Sub, SubAssign};
use std::iter::Sum;
use std::str::{self, FromStr};

use num_bigint::{BigInt, ParseBigIntError, Sign, ToBigInt};
use num_integer::Integer as IntegerTrait;
pub use num_traits::{FromPrimitive, Num, One, Signed, ToPrimitive, Zero};

const LOG2_10: f64 = 3.321928094887362_f64;

#[macro_use]
mod macros;

#[inline(always)]
fn ten_to_the(pow: u64) -> BigInt {
    if pow < 20 {
        BigInt::from(10u64.pow(pow as u32))
    } else {
        let (half, rem) = pow.div_rem(&16);

        let mut x = ten_to_the(half);

        for _ in 0..4 {
            x = &x * &x;
        }

        if rem == 0 {
            x
        } else {
            x * ten_to_the(rem)
        }
    }
}

#[inline(always)]
fn count_decimal_digits(int: &BigInt) -> u64 {
    if int.is_zero() {
        return 1;
    }
    // guess number of digits based on number of bits in UInt
    let mut digits = (int.bits() as f64 / LOG2_10) as u64;
    let mut num = ten_to_the(digits);
    while *int >= num {
        num *= 10u8;
        digits += 1;
    }
    digits
}

/// Internal function used for rounding
///
/// returns 1 if most significant digit is >= 5, otherwise 0
///
/// This is used after dividing a number by a power of ten and
/// rounding the last digit.
///
#[inline(always)]
fn get_rounding_term(num: &BigInt) -> u8 {
    if num.is_zero() {
        return 0;
    }

    let digits = (num.bits() as f64 / LOG2_10) as u64;
    let mut n = ten_to_the(digits);

    // loop-method
    loop {
        if *num < n {
            return 1;
        }
        n *= 5;
        if *num < n {
            return 0;
        }
        n *= 2;
    }

    // string-method
    // let s = format!("{}", num);
    // let high_digit = u8::from_str(&s[0..1]).unwrap();
    // if high_digit < 5 { 0 } else { 1 }
}

/// A big decimal type.
///
#[derive(Clone, Eq)]
pub struct BigDecimal {
    int_val: BigInt,
    // A positive scale means a negative power of 10
    scale: i64,
}

impl BigDecimal {
    /// Creates and initializes a `BigDecimal`.
    ///
    #[inline]
    pub fn new(digits: BigInt, scale: i64) -> BigDecimal {
        BigDecimal {
            int_val: digits,
            scale: scale,
        }
    }

    /// Creates and initializes a `BigDecimal`.
    ///
    /// Decodes using `str::from_utf8` and forwards to `BigDecimal::from_str_radix`.
    /// Only base-10 is supported.
    ///
    /// # Examples
    ///
    /// ```
    /// use bigdecimal::{BigDecimal, Zero};
    ///
    /// assert_eq!(BigDecimal::parse_bytes(b"0", 10).unwrap(), BigDecimal::zero());
    /// assert_eq!(BigDecimal::parse_bytes(b"13", 10).unwrap(), BigDecimal::from(13));
    /// ```
    #[inline]
    pub fn parse_bytes(buf: &[u8], radix: u32) -> Option<BigDecimal> {
        str::from_utf8(buf)
            .ok()
            .and_then(|s| BigDecimal::from_str_radix(s, radix).ok())
    }

    /// Return a new BigDecimal object equivalent to self, with internal
    /// scaling set to the number specified.
    /// If the new_scale is lower than the current value (indicating a larger
    /// power of 10), digits will be dropped (as precision is lower)
    ///
    #[inline]
    pub fn with_scale(&self, new_scale: i64) -> BigDecimal {
        if self.int_val.is_zero() {
            return BigDecimal::new(BigInt::zero(), new_scale);
        }

        if new_scale > self.scale {
            let scale_diff = new_scale - self.scale;
            let int_val = &self.int_val * ten_to_the(scale_diff as u64);
            BigDecimal::new(int_val, new_scale)
        } else if new_scale < self.scale {
            let scale_diff = self.scale - new_scale;
            let int_val = &self.int_val / ten_to_the(scale_diff as u64);
            BigDecimal::new(int_val, new_scale)
        } else {
            self.clone()
        }
    }

    #[inline(always)]
    fn take_and_scale(mut self, new_scale: i64) -> BigDecimal {
        // let foo = bar.moved_and_scaled_to()
        if self.int_val.is_zero() {
            return BigDecimal::new(BigInt::zero(), new_scale);
        }

        if new_scale > self.scale {
            self.int_val *= ten_to_the((new_scale - self.scale) as u64);
            BigDecimal::new(self.int_val, new_scale)
        } else if new_scale < self.scale {
            self.int_val /= ten_to_the((self.scale - new_scale) as u64);
            BigDecimal::new(self.int_val, new_scale)
        } else {
            self
        }
    }

    /// Return a new BigDecimal object with precision set to new value
    ///
    #[inline]
    pub fn with_prec(&self, prec: u64) -> BigDecimal {
        let digits = self.digits();

        if digits > prec {
            let diff = digits - prec;
            let p = ten_to_the(diff);
            let (mut q, r) = self.int_val.div_rem(&p);

            // check for "leading zero" in remainder term; otherwise round
            if p < 10 * &r {
                q += get_rounding_term(&r);
            }

            BigDecimal {
                int_val: q,
                scale: self.scale - diff as i64,
            }
        } else if digits < prec {
            let diff = prec - digits;
            BigDecimal {
                int_val: &self.int_val * ten_to_the(diff),
                scale: self.scale + diff as i64,
            }
        } else {
            self.clone()
        }
    }

    /// Return the sign of the `BigDecimal` as `num::bigint::Sign`.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate num_bigint;
    /// extern crate bigdecimal;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(bigdecimal::BigDecimal::from_str("-1").unwrap().sign(), num_bigint::Sign::Minus);
    /// assert_eq!(bigdecimal::BigDecimal::from_str("0").unwrap().sign(), num_bigint::Sign::NoSign);
    /// assert_eq!(bigdecimal::BigDecimal::from_str("1").unwrap().sign(), num_bigint::Sign::Plus);
    /// ```
    #[inline]
    pub fn sign(&self) -> num_bigint::Sign {
        self.int_val.sign()
    }

    /// Return the internal big integer value and an exponent. Note that a positive
    /// exponent indicates a negative power of 10.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate num_bigint;
    /// extern crate bigdecimal;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(bigdecimal::BigDecimal::from_str("1.1").unwrap().as_bigint_and_exponent(),
    ///            (num_bigint::BigInt::from_str("11").unwrap(), 1));
    #[inline]
    pub fn as_bigint_and_exponent(&self) -> (BigInt, i64) {
        (self.int_val.clone(), self.scale)
    }

    /// Convert into the internal big integer value and an exponent. Note that a positive
    /// exponent indicates a negative power of 10.
    ///
    /// # Examples
    ///
    /// ```
    /// extern crate num_bigint;
    /// extern crate bigdecimal;
    /// use std::str::FromStr;
    ///
    /// assert_eq!(bigdecimal::BigDecimal::from_str("1.1").unwrap().into_bigint_and_exponent(),
    ///            (num_bigint::BigInt::from_str("11").unwrap(), 1));
    #[inline]
    pub fn into_bigint_and_exponent(self) -> (BigInt, i64) {
        (self.int_val, self.scale)
    }

    /// Number of digits in the non-scaled integer representation
    ///
    #[inline]
    pub fn digits(&self) -> u64 {
        count_decimal_digits(&self.int_val)
    }

    /// Compute the absolute value of number
    #[inline]
    pub fn abs(&self) -> BigDecimal {
        BigDecimal {
            int_val: self.int_val.abs(),
            scale: self.scale,
        }
    }

    #[inline]
    pub fn double(&self) -> BigDecimal {
        if self.is_zero() {
            self.clone()
        } else {
            BigDecimal {
                int_val: self.int_val.clone() * 2,
                scale: self.scale,
            }
        }
    }

    /// Divide this efficiently by 2
    ///
    /// Note, if this is odd, the precision will increase by 1, regardless
    /// of the context's limit.
    ///
    #[inline]
    pub fn half(&self) -> BigDecimal {
        if self.is_zero() {
            self.clone()
        } else if self.int_val.is_even() {
            BigDecimal {
                int_val: self.int_val.clone().div(2u8),
                scale: self.scale,
            }
        } else {
            BigDecimal {
                int_val: self.int_val.clone().mul(5u8),
                scale: self.scale + 1,
            }
        }
    }

    ///
    #[inline]
    pub fn square(&self) -> BigDecimal {
        if self.is_zero() || self.is_one() {
            self.clone()
        } else {
            BigDecimal {
                int_val: self.int_val.clone() * &self.int_val,
                scale: self.scale * 2,
            }
        }
    }

    #[inline]
    pub fn cube(&self) -> BigDecimal {
        if self.is_zero() || self.is_one() {
            self.clone()
        } else {
            BigDecimal {
                int_val: self.int_val.clone() * &self.int_val * &self.int_val,
                scale: self.scale * 3,
            }
        }
    }

    /// Take the square root of the number
    ///
    /// If the value is < 0, None is returned
    ///
    #[inline]
    pub fn sqrt(&self) -> Option<BigDecimal> {
        if self.is_zero() || self.is_one() {
            return Some(self.clone());
        }
        if self.is_negative() {
            return None;
        }

        // make guess
        let guess = {
            let magic_guess_scale = 1.1951678538495576_f64;
            let initial_guess = (self.int_val.bits() as f64 - self.scale as f64 * LOG2_10) / 2.0;
            let res = magic_guess_scale * initial_guess.exp2();
            if res.is_normal() {
                BigDecimal::try_from(res).unwrap()
            } else {
                // can't guess with float - just guess magnitude
                let scale = (self.int_val.bits() as f64 / -LOG2_10 + self.scale as f64).round() as i64;
                BigDecimal::new(BigInt::from(1), scale / 2)
            }
        };

        // // wikipedia example - use for testing the algorithm
        // if self == &BigDecimal::from_str("125348").unwrap() {
        //     running_result = BigDecimal::from(600)
        // }

        // TODO: Use context variable to set precision
        let max_precision = 100;

        let next_iteration = move |r: BigDecimal| {
            // division needs to be precise to (at least) one extra digit
            let tmp = impl_division(
                self.int_val.clone(),
                &r.int_val,
                self.scale - r.scale,
                max_precision + 1,
            );

            // half will increase precision on each iteration
            (tmp + r).half()
        };

        // calculate first iteration
        let mut running_result = next_iteration(guess);

        let mut prev_result = BigDecimal::one();
        let mut result = BigDecimal::zero();

        // TODO: Prove that we don't need to arbitrarily limit iterations
        // and that convergence can be calculated
        while prev_result != result {
            // store current result to test for convergence
            prev_result = result;

            // calculate next iteration
            running_result = next_iteration(running_result);

            // 'result' has clipped precision, 'running_result' has full precision
            result = if running_result.digits() > max_precision {
                running_result.with_prec(max_precision)
            } else {
                running_result.clone()
            };
        }

        return Some(result);
    }

    /// Take the cube root of the number
    ///
    #[inline]
    pub fn cbrt(&self) -> BigDecimal {
        if self.is_zero() || self.is_one() {
            return self.clone();
        }
        if self.is_negative() {
            return -self.abs().cbrt();
        }

        // make guess
        let guess = {
            let magic_guess_scale = 1.124960491619939_f64;
            let initial_guess = (self.int_val.bits() as f64 - self.scale as f64 * LOG2_10) / 3.0;
            let res = magic_guess_scale * initial_guess.exp2();
            if res.is_normal() {
                BigDecimal::try_from(res).unwrap()
            } else {
                // can't guess with float - just guess magnitude
                let scale = (self.int_val.bits() as f64 / LOG2_10 - self.scale as f64).round() as i64;
                BigDecimal::new(BigInt::from(1), -scale / 3)
            }
        };

        // TODO: Use context variable to set precision
        let max_precision = 100;

        let three = BigDecimal::from(3);

        let next_iteration = move |r: BigDecimal| {
            let sqrd = r.square();
            let tmp = impl_division(
                self.int_val.clone(),
                &sqrd.int_val,
                self.scale - sqrd.scale,
                max_precision + 1,
            );
            let tmp = tmp + r.double();
            impl_division(tmp.int_val, &three.int_val, tmp.scale - three.scale, max_precision + 1)
        };

        // result initial
        let mut running_result = next_iteration(guess);

        let mut prev_result = BigDecimal::one();
        let mut result = BigDecimal::zero();

        // TODO: Prove that we don't need to arbitrarily limit iterations
        // and that convergence can be calculated
        while prev_result != result {
            // store current result to test for convergence
            prev_result = result;

            running_result = next_iteration(running_result);

            // result has clipped precision, running_result has full precision
            result = if running_result.digits() > max_precision {
                running_result.with_prec(max_precision)
            } else {
                running_result.clone()
            };
        }

        return result;
    }

    /// Compute the reciprical of the number: x<sup>-1</sup>
    #[inline]
    pub fn inverse(&self) -> BigDecimal {
        if self.is_zero() || self.is_one() {
            return self.clone();
        }
        if self.is_negative() {
            return self.abs().inverse().neg();
        }
        let guess = {
            let bits = self.int_val.bits() as f64;
            let scale = self.scale as f64;

            let magic_factor = 0.721507597259061_f64;
            let initial_guess = scale * LOG2_10 - bits;
            let res = magic_factor * initial_guess.exp2();

            if res.is_normal() {
                BigDecimal::try_from(res).unwrap()
            } else {
                // can't guess with float - just guess magnitude
                let scale = (bits / LOG2_10 + scale).round() as i64;
                BigDecimal::new(BigInt::from(1), -scale)
            }
        };

        let max_precision = 100;
        let next_iteration = move |r: BigDecimal| {
            let two = BigDecimal::from(2);
            let tmp = two - self * &r;

            r * tmp
        };

        // calculate first iteration
        let mut running_result = next_iteration(guess);

        let mut prev_result = BigDecimal::one();
        let mut result = BigDecimal::zero();

        // TODO: Prove that we don't need to arbitrarily limit iterations
        // and that convergence can be calculated
        while prev_result != result {
            // store current result to test for convergence
            prev_result = result;

            // calculate next iteration
            running_result = next_iteration(running_result).with_prec(max_precision);

            // 'result' has clipped precision, 'running_result' has full precision
            result = if running_result.digits() > max_precision {
                running_result.with_prec(max_precision)
            } else {
                running_result.clone()
            };
        }

        return result;
    }

    /// Return number rounded to round_digits precision after the decimal point
    pub fn round(&self, round_digits: i64) -> BigDecimal {
        let (bigint, decimal_part_digits) = self.as_bigint_and_exponent();
        let need_to_round_digits = decimal_part_digits - round_digits;
        if round_digits >= 0 && need_to_round_digits <= 0 {
            return self.clone();
        }

        let mut number = bigint.to_i128().unwrap();
        if number < 0 {
            number = -number;
        }
        for _ in 0..(need_to_round_digits - 1) {
            number /= 10;
        }
        let digit = number % 10;

        if digit <= 4 {
            self.with_scale(round_digits)
        } else if bigint.is_negative() {
            self.with_scale(round_digits) - BigDecimal::new(BigInt::from(1), round_digits)
        } else {
            self.with_scale(round_digits) + BigDecimal::new(BigInt::from(1), round_digits)
        }
    }

    /// Return true if this number has zero fractional part (is equal
    /// to an integer)
    ///
    #[inline]
    pub fn is_integer(&self) -> bool {
        if self.scale <= 0 {
            true
        } else {
            (self.int_val.clone() % ten_to_the(self.scale as u64)).is_zero()
        }
    }

    /// Evaluate the natural-exponential function e<sup>x</sup>
    ///
    #[inline]
    pub fn exp(&self) -> BigDecimal {
        if self.is_zero() {
            return BigDecimal::one();
        }

        let precision = self.digits();

        let mut term = self.clone();
        let mut result = self.clone() + BigDecimal::one();
        let mut prev_result = result.clone();
        let mut factorial = BigInt::one();

        for n in 2.. {
            term *= self;
            factorial *= n;
            // ∑ term=x^n/n!
            result += impl_division(term.int_val.clone(), &factorial, term.scale, 117 + precision);

            let trimmed_result = result.with_prec(105);
            if prev_result == trimmed_result {
                return trimmed_result.with_prec(100);
            }
            prev_result = trimmed_result;
        }
        return result.with_prec(100);
    }

    #[must_use]
    pub fn normalized(&self) -> BigDecimal {
        if self == &BigDecimal::zero() {
            return BigDecimal::zero();
        }
        let (sign, mut digits) = self.int_val.to_radix_be(10);
        let trailing_count = digits.iter().rev().take_while(|i| **i == 0).count();
        let trunc_to = digits.len() - trailing_count as usize;
        digits.truncate(trunc_to);
        let int_val = BigInt::from_radix_be(sign, &digits, 10).unwrap();
        let scale = self.scale - trailing_count as i64;
        BigDecimal::new(int_val, scale)
    }
}

#[derive(Debug, PartialEq)]
pub enum ParseBigDecimalError {
    ParseDecimal(ParseFloatError),
    ParseInt(ParseIntError),
    ParseBigInt(ParseBigIntError),
    Empty,
    Other(String),
}

impl fmt::Display for ParseBigDecimalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseBigDecimalError::*;

        match *self {
            ParseDecimal(ref e) => e.fmt(f),
            ParseInt(ref e) => e.fmt(f),
            ParseBigInt(ref e) => e.fmt(f),
            Empty => "Failed to parse empty string".fmt(f),
            Other(ref reason) => reason[..].fmt(f),
        }
    }
}

impl Error for ParseBigDecimalError {
    fn description(&self) -> &str {
        "failed to parse bigint/biguint"
    }
}

impl From<ParseFloatError> for ParseBigDecimalError {
    fn from(err: ParseFloatError) -> ParseBigDecimalError {
        ParseBigDecimalError::ParseDecimal(err)
    }
}

impl From<ParseIntError> for ParseBigDecimalError {
    fn from(err: ParseIntError) -> ParseBigDecimalError {
        ParseBigDecimalError::ParseInt(err)
    }
}

impl From<ParseBigIntError> for ParseBigDecimalError {
    fn from(err: ParseBigIntError) -> ParseBigDecimalError {
        ParseBigDecimalError::ParseBigInt(err)
    }
}

impl FromStr for BigDecimal {
    type Err = ParseBigDecimalError;

    #[inline]
    fn from_str(s: &str) -> Result<BigDecimal, ParseBigDecimalError> {
        BigDecimal::from_str_radix(s, 10)
    }
}

#[allow(deprecated)] // trim_right_match -> trim_end_match
impl Hash for BigDecimal {
    fn hash<H: Hasher>(&self, state: &mut H) {
        let mut dec_str = self.int_val.to_str_radix(10).to_string();
        let scale = self.scale;
        let zero = self.int_val.is_zero();
        if scale > 0 && !zero {
            let mut cnt = 0;
            dec_str = dec_str
                .trim_right_matches(|x| {
                    cnt += 1;
                    x == '0' && cnt <= scale
                })
                .to_string();
        } else if scale < 0 && !zero {
            dec_str.push_str(&"0".repeat(self.scale.abs() as usize));
        }
        dec_str.hash(state);
    }
}

impl PartialOrd for BigDecimal {
    #[inline]
    fn partial_cmp(&self, other: &BigDecimal) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BigDecimal {
    /// Complete ordering implementation for BigDecimal
    ///
    /// # Example
    ///
    /// ```
    /// use std::str::FromStr;
    ///
    /// let a = bigdecimal::BigDecimal::from_str("-1").unwrap();
    /// let b = bigdecimal::BigDecimal::from_str("1").unwrap();
    /// assert!(a < b);
    /// assert!(b > a);
    /// let c = bigdecimal::BigDecimal::from_str("1").unwrap();
    /// assert!(b >= c);
    /// assert!(c >= b);
    /// let d = bigdecimal::BigDecimal::from_str("10.0").unwrap();
    /// assert!(d > c);
    /// let e = bigdecimal::BigDecimal::from_str(".5").unwrap();
    /// assert!(e < c);
    /// ```
    #[inline]
    fn cmp(&self, other: &BigDecimal) -> Ordering {
        let scmp = self.sign().cmp(&other.sign());
        if scmp != Ordering::Equal {
            return scmp;
        }

        match self.sign() {
            Sign::NoSign => Ordering::Equal,
            _ => {
                let tmp = self - other;
                match tmp.sign() {
                    Sign::Plus => Ordering::Greater,
                    Sign::Minus => Ordering::Less,
                    Sign::NoSign => Ordering::Equal,
                }
            }
        }
    }
}

impl PartialEq for BigDecimal {
    #[inline]
    fn eq(&self, rhs: &BigDecimal) -> bool {
        // fix scale and test equality
        if self.scale > rhs.scale {
            let scaled_int_val = &rhs.int_val * ten_to_the((self.scale - rhs.scale) as u64);
            self.int_val == scaled_int_val
        } else if self.scale < rhs.scale {
            let scaled_int_val = &self.int_val * ten_to_the((rhs.scale - self.scale) as u64);
            scaled_int_val == rhs.int_val
        } else {
            self.int_val == rhs.int_val
        }
    }
}

impl Default for BigDecimal {
    #[inline]
    fn default() -> BigDecimal {
        Zero::zero()
    }
}

impl Zero for BigDecimal {
    #[inline]
    fn zero() -> BigDecimal {
        BigDecimal::new(BigInt::zero(), 0)
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.int_val.is_zero()
    }
}

impl One for BigDecimal {
    #[inline]
    fn one() -> BigDecimal {
        BigDecimal::new(BigInt::one(), 0)
    }
}

impl Add<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: BigDecimal) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&rhs.scale) {
            Ordering::Equal => {
                lhs.int_val += rhs.int_val;
                lhs
            }
            Ordering::Less => lhs.take_and_scale(rhs.scale) + rhs,
            Ordering::Greater => rhs.take_and_scale(lhs.scale) + lhs,
        }
    }
}

impl<'a> Add<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &'a BigDecimal) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&rhs.scale) {
            Ordering::Equal => {
                lhs.int_val += &rhs.int_val;
                lhs
            }
            Ordering::Less => lhs.take_and_scale(rhs.scale) + rhs,
            Ordering::Greater => rhs.with_scale(lhs.scale) + lhs,
        }
    }
}

impl<'a> Add<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: BigDecimal) -> BigDecimal {
        rhs + self
    }
}

impl<'a, 'b> Add<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &BigDecimal) -> BigDecimal {
        let lhs = self;
        if self.scale < rhs.scale {
            lhs.with_scale(rhs.scale) + rhs
        } else if self.scale > rhs.scale {
            rhs.with_scale(lhs.scale) + lhs
        } else {
            BigDecimal::new(lhs.int_val.clone() + &rhs.int_val, lhs.scale)
        }
    }
}

impl Add<BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: BigInt) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&0) {
            Ordering::Equal => {
                lhs.int_val += rhs;
                lhs
            }
            Ordering::Greater => {
                lhs.int_val += rhs * ten_to_the(lhs.scale as u64);
                lhs
            }
            Ordering::Less => lhs.take_and_scale(0) + rhs,
        }
    }
}

impl<'a> Add<&'a BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &BigInt) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&0) {
            Ordering::Equal => {
                lhs.int_val += rhs;
                lhs
            }
            Ordering::Greater => {
                lhs.int_val += rhs * ten_to_the(lhs.scale as u64);
                lhs
            }
            Ordering::Less => lhs.take_and_scale(0) + rhs,
        }
    }
}

impl<'a> Add<BigInt> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: BigInt) -> BigDecimal {
        BigDecimal::new(rhs, 0) + self
    }
}

impl<'a, 'b> Add<&'a BigInt> for &'b BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn add(self, rhs: &BigInt) -> BigDecimal {
        self.with_scale(0) + rhs
    }
}

forward_val_assignop!(impl AddAssign for BigDecimal, add_assign);

impl<'a> AddAssign<&'a BigDecimal> for BigDecimal {
    #[inline]
    fn add_assign(&mut self, rhs: &BigDecimal) {
        if self.scale < rhs.scale {
            let scaled = self.with_scale(rhs.scale);
            self.int_val = scaled.int_val + &rhs.int_val;
            self.scale = rhs.scale;
        } else if self.scale > rhs.scale {
            let scaled = rhs.with_scale(self.scale);
            self.int_val += scaled.int_val;
        } else {
            self.int_val += &rhs.int_val;
        }
    }
}

impl<'a> AddAssign<BigInt> for BigDecimal {
    #[inline]
    fn add_assign(&mut self, rhs: BigInt) {
        *self += BigDecimal::new(rhs, 0)
    }
}

impl<'a> AddAssign<&'a BigInt> for BigDecimal {
    #[inline]
    fn add_assign(&mut self, rhs: &BigInt) {
        /* // which one looks best?
        if self.scale == 0 {
            self.int_val += rhs;
        } else if self.scale > 0 {
            self.int_val += rhs * ten_to_the(self.scale as u64);
        } else {
            self.int_val *= ten_to_the((-self.scale) as u64);
            self.int_val += rhs;
            self.scale = 0;
        }
         */
        match self.scale.cmp(&0) {
            Ordering::Equal => self.int_val += rhs,
            Ordering::Greater => self.int_val += rhs * ten_to_the(self.scale as u64),
            Ordering::Less => {
                // *self += BigDecimal::new(rhs, 0)
                self.int_val *= ten_to_the((-self.scale) as u64);
                self.int_val += rhs;
                self.scale = 0;
            }
        }
    }
}

impl Sub<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: BigDecimal) -> BigDecimal {
        let mut lhs = self;
        let scale = std::cmp::max(lhs.scale, rhs.scale);

        match lhs.scale.cmp(&rhs.scale) {
            Ordering::Equal => {
                lhs.int_val -= rhs.int_val;
                lhs
            }
            Ordering::Less => lhs.take_and_scale(scale) - rhs,
            Ordering::Greater => lhs - rhs.take_and_scale(scale),
        }
    }
}

impl<'a> Sub<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: &BigDecimal) -> BigDecimal {
        let mut lhs = self;
        let scale = std::cmp::max(lhs.scale, rhs.scale);

        match lhs.scale.cmp(&rhs.scale) {
            Ordering::Equal => {
                lhs.int_val -= &rhs.int_val;
                lhs
            }
            Ordering::Less => lhs.take_and_scale(rhs.scale) - rhs,
            Ordering::Greater => lhs - rhs.with_scale(scale),
        }
    }
}

impl<'a> Sub<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: BigDecimal) -> BigDecimal {
        -(rhs - self)
    }
}

impl<'a, 'b> Sub<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: &BigDecimal) -> BigDecimal {
        if self.scale < rhs.scale {
            self.with_scale(rhs.scale) - rhs
        } else if self.scale > rhs.scale {
            let rhs = rhs.with_scale(self.scale);
            self - rhs
        } else {
            BigDecimal::new(&self.int_val - &rhs.int_val, self.scale)
        }
    }
}

impl Sub<BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: BigInt) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&0) {
            Ordering::Equal => {
                lhs.int_val -= rhs;
                lhs
            }
            Ordering::Greater => {
                lhs.int_val -= rhs * ten_to_the(lhs.scale as u64);
                lhs
            }
            Ordering::Less => lhs.take_and_scale(0) - rhs,
        }
    }
}

impl<'a> Sub<&'a BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: &BigInt) -> BigDecimal {
        let mut lhs = self;

        match lhs.scale.cmp(&0) {
            Ordering::Equal => {
                lhs.int_val -= rhs;
                lhs
            }
            Ordering::Greater => {
                lhs.int_val -= rhs * ten_to_the(lhs.scale as u64);
                lhs
            }
            Ordering::Less => lhs.take_and_scale(0) - rhs,
        }
    }
}

impl<'a> Sub<BigInt> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: BigInt) -> BigDecimal {
        BigDecimal::new(rhs, 0) - self
    }
}

impl<'a, 'b> Sub<&'a BigInt> for &'b BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn sub(self, rhs: &BigInt) -> BigDecimal {
        self.with_scale(0) - rhs
    }
}

forward_val_assignop!(impl SubAssign for BigDecimal, sub_assign);

impl<'a> SubAssign<&'a BigDecimal> for BigDecimal {
    #[inline]
    fn sub_assign(&mut self, rhs: &BigDecimal) {
        if self.scale < rhs.scale {
            let lhs = self.with_scale(rhs.scale);
            self.int_val = lhs.int_val - &rhs.int_val;
            self.scale = rhs.scale;
        } else if self.scale > rhs.scale {
            self.int_val -= rhs.with_scale(self.scale).int_val;
        } else {
            self.int_val = &self.int_val - &rhs.int_val;
        }
    }
}

impl<'a> SubAssign<BigInt> for BigDecimal {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: BigInt) {
        *self -= BigDecimal::new(rhs, 0)
    }
}

impl<'a> SubAssign<&'a BigInt> for BigDecimal {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: &BigInt) {
        match self.scale.cmp(&0) {
            Ordering::Equal => SubAssign::sub_assign(&mut self.int_val, rhs),
            Ordering::Greater => SubAssign::sub_assign(&mut self.int_val, rhs * ten_to_the(self.scale as u64)),
            Ordering::Less => {
                self.int_val *= ten_to_the((-self.scale) as u64);
                SubAssign::sub_assign(&mut self.int_val, rhs);
                self.scale = 0;
            }
        }
    }
}

impl Mul<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(mut self, rhs: BigDecimal) -> BigDecimal {
        self.scale += rhs.scale;
        self.int_val *= rhs.int_val;
        self
    }
}

impl<'a> Mul<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(mut self, rhs: &'a BigDecimal) -> BigDecimal {
        self.scale += rhs.scale;
        MulAssign::mul_assign(&mut self.int_val, &rhs.int_val);
        self
    }
}

impl<'a> Mul<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(self, rhs: BigDecimal) -> BigDecimal {
        rhs * self
    }
}

impl<'a, 'b> Mul<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(self, rhs: &BigDecimal) -> BigDecimal {
        let scale = self.scale + rhs.scale;
        BigDecimal::new(&self.int_val * &rhs.int_val, scale)
    }
}

impl Mul<BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(mut self, rhs: BigInt) -> BigDecimal {
        self.int_val *= rhs;
        self
    }
}

impl<'a> Mul<&'a BigInt> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(mut self, rhs: &BigInt) -> BigDecimal {
        self.int_val *= rhs;
        self
    }
}

impl<'a> Mul<BigInt> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(self, mut rhs: BigInt) -> BigDecimal {
        rhs *= &self.int_val;
        BigDecimal::new(rhs, self.scale)
    }
}

impl<'a, 'b> Mul<&'a BigInt> for &'b BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn mul(self, rhs: &BigInt) -> BigDecimal {
        let value = &self.int_val * rhs;
        BigDecimal::new(value, self.scale)
    }
}

forward_val_assignop!(impl MulAssign for BigDecimal, mul_assign);

impl<'a> MulAssign<&'a BigDecimal> for BigDecimal {
    #[inline]
    fn mul_assign(&mut self, rhs: &BigDecimal) {
        self.scale += rhs.scale;
        self.int_val = &self.int_val * &rhs.int_val;
    }
}

impl_div_for_primitives!();

#[inline(always)]
fn impl_division(mut num: BigInt, den: &BigInt, mut scale: i64, max_precision: u64) -> BigDecimal {
    // quick zero check
    if num.is_zero() {
        return BigDecimal::new(num, 0);
    }

    match (num.is_negative(), den.is_negative()) {
        (true, true) => return impl_division(num.neg(), &den.neg(), scale, max_precision),
        (true, false) => return -impl_division(num.neg(), den, scale, max_precision),
        (false, true) => return -impl_division(num, &den.neg(), scale, max_precision),
        (false, false) => (),
    }

    // shift digits until numerator is larger than denominator (set scale appropriately)
    while num < *den {
        scale += 1;
        num *= 10;
    }

    // first division
    let (mut quotient, mut remainder) = num.div_rem(den);

    // division complete
    if remainder.is_zero() {
        return BigDecimal {
            int_val: quotient,
            scale: scale,
        };
    }

    let mut precision = count_decimal_digits(&quotient);

    // shift remainder by 1 decimal;
    // quotient will be 1 digit upon next division
    remainder *= 10;

    while !remainder.is_zero() && precision < max_precision {
        let (q, r) = remainder.div_rem(den);
        quotient = quotient * 10 + q;
        remainder = r * 10;

        precision += 1;
        scale += 1;
    }

    if !remainder.is_zero() {
        // round final number with remainder
        quotient += get_rounding_term(&remainder.div(den));
    }

    let result = BigDecimal::new(quotient, scale);
    // println!(" {} / {}\n = {}\n", self, other, result);
    return result;
}

impl Div<BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    #[inline]
    fn div(self, other: BigDecimal) -> BigDecimal {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self.is_zero() || other.is_one() {
            return self;
        }

        let scale = self.scale - other.scale;

        if self.int_val == other.int_val {
            return BigDecimal {
                int_val: 1.into(),
                scale: scale,
            };
        }

        let max_precision = 100;

        return impl_division(self.int_val, &other.int_val, scale, max_precision);
    }
}

impl<'a> Div<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;
    #[inline]
    fn div(self, other: &'a BigDecimal) -> BigDecimal {
        if other.is_zero() {
            panic!("Division by zero");
        }
        if self.is_zero() || other.is_one() {
            return self;
        }

        let scale = self.scale - other.scale;

        if self.int_val == other.int_val {
            return BigDecimal {
                int_val: 1.into(),
                scale: scale,
            };
        }

        let max_precision = 100;

        return impl_division(self.int_val, &other.int_val, scale, max_precision);
    }
}

forward_ref_val_binop!(impl Div for BigDecimal, div);

impl<'a, 'b> Div<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn div(self, other: &BigDecimal) -> BigDecimal {
        if other.is_zero() {
            panic!("Division by zero");
        }
        // TODO: Fix setting scale
        if self.is_zero() || other.is_one() {
            return self.clone();
        }

        let scale = self.scale - other.scale;

        let num_int = &self.int_val;
        let den_int = &other.int_val;

        if num_int == den_int {
            return BigDecimal {
                int_val: 1.into(),
                scale: scale,
            };
        }

        let max_precision = 100;

        return impl_division(num_int.clone(), &den_int, scale, max_precision);
    }
}

impl Rem<BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn rem(self, other: BigDecimal) -> BigDecimal {
        let scale = std::cmp::max(self.scale, other.scale);

        let num = self.take_and_scale(scale).int_val;
        let den = other.take_and_scale(scale).int_val;

        BigDecimal::new(num % den, scale)
    }
}

impl<'a> Rem<&'a BigDecimal> for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn rem(self, other: &BigDecimal) -> BigDecimal {
        let scale = std::cmp::max(self.scale, other.scale);
        let num = self.take_and_scale(scale).int_val;
        let den = &other.int_val;

        let result = if scale == other.scale {
            num % den
        } else {
            num % (den * ten_to_the((scale - other.scale) as u64))
        };
        BigDecimal::new(result, scale)
    }
}
impl<'a> Rem<BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn rem(self, other: BigDecimal) -> BigDecimal {
        let scale = std::cmp::max(self.scale, other.scale);
        let num = &self.int_val;
        let den = other.take_and_scale(scale).int_val;

        let result = if scale == self.scale {
            num % den
        } else {
            let scaled_num = num * ten_to_the((scale - self.scale) as u64);
            scaled_num % den
        };

        BigDecimal::new(result, scale)
    }
}

impl<'a, 'b> Rem<&'b BigDecimal> for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn rem(self, other: &BigDecimal) -> BigDecimal {
        let scale = std::cmp::max(self.scale, other.scale);
        let num = &self.int_val;
        let den = &other.int_val;

        let result = match self.scale.cmp(&other.scale) {
            Ordering::Equal => num % den,
            Ordering::Less => {
                let scaled_num = num * ten_to_the((scale - self.scale) as u64);
                scaled_num % den
            }
            Ordering::Greater => {
                let scaled_den = den * ten_to_the((scale - other.scale) as u64);
                num % scaled_den
            }
        };
        BigDecimal::new(result, scale)
    }
}

impl Neg for BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn neg(mut self) -> BigDecimal {
        self.int_val = -self.int_val;
        self
    }
}

impl<'a> Neg for &'a BigDecimal {
    type Output = BigDecimal;

    #[inline]
    fn neg(self) -> BigDecimal {
        -self.clone()
    }
}

impl Signed for BigDecimal {
    #[inline]
    fn abs(&self) -> BigDecimal {
        match self.sign() {
            Sign::Plus | Sign::NoSign => self.clone(),
            Sign::Minus => -self,
        }
    }

    #[inline]
    fn abs_sub(&self, other: &BigDecimal) -> BigDecimal {
        if *self <= *other {
            Zero::zero()
        } else {
            self - other
        }
    }

    #[inline]
    fn signum(&self) -> BigDecimal {
        match self.sign() {
            Sign::Plus => One::one(),
            Sign::NoSign => Zero::zero(),
            Sign::Minus => -Self::one(),
        }
    }

    #[inline]
    fn is_positive(&self) -> bool {
        self.sign() == Sign::Plus
    }

    #[inline]
    fn is_negative(&self) -> bool {
        self.sign() == Sign::Minus
    }
}

impl Sum for BigDecimal {
    #[inline]
    fn sum<I: Iterator<Item = BigDecimal>>(iter: I) -> BigDecimal {
        iter.fold(Zero::zero(), |a, b| a + b)
    }
}

impl<'a> Sum<&'a BigDecimal> for BigDecimal {
    #[inline]
    fn sum<I: Iterator<Item = &'a BigDecimal>>(iter: I) -> BigDecimal {
        iter.fold(Zero::zero(), |a, b| a + b)
    }
}

impl fmt::Display for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Aquire the absolute integer as a decimal string
        let mut abs_int = self.int_val.abs().to_str_radix(10);

        // Split the representation at the decimal point
        let (before, after) = if self.scale >= abs_int.len() as i64 {
            // First case: the integer representation falls
            // completely behind the decimal point
            let scale = self.scale as usize;
            let after = "0".repeat(scale - abs_int.len()) + abs_int.as_str();
            ("0".to_string(), after)
        } else {
            // Second case: the integer representation falls
            // around, or before the decimal point
            let location = abs_int.len() as i64 - self.scale;
            if location > abs_int.len() as i64 {
                // Case 2.1, entirely before the decimal point
                // We should prepend zeros
                let zeros = location as usize - abs_int.len();
                let abs_int = abs_int + "0".repeat(zeros as usize).as_str();
                (abs_int, "".to_string())
            } else {
                // Case 2.2, somewhere around the decimal point
                // Just split it in two
                let after = abs_int.split_off(location as usize);
                (abs_int, after)
            }
        };

        // Alter precision after the decimal point
        let after = if let Some(precision) = f.precision() {
            let len = after.len();
            if len < precision {
                after + "0".repeat(precision - len).as_str()
            } else {
                // TODO: Should we round?
                after[0..precision].to_string()
            }
        } else {
            after
        };

        // Concatenate everything
        let complete_without_sign = if !after.is_empty() {
            before + "." + after.as_str()
        } else {
            before
        };

        let non_negative = match self.int_val.sign() {
            Sign::Plus | Sign::NoSign => true,
            _ => false,
        };
        //pad_integral does the right thing although we have a decimal
        f.pad_integral(non_negative, "", &complete_without_sign)
    }
}

impl fmt::Debug for BigDecimal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "BigDecimal(\"{}\")", self)
    }
}

impl Num for BigDecimal {
    type FromStrRadixErr = ParseBigDecimalError;

    /// Creates and initializes a BigDecimal.
    #[inline]
    fn from_str_radix(s: &str, radix: u32) -> Result<BigDecimal, ParseBigDecimalError> {
        if radix != 10 {
            return Err(ParseBigDecimalError::Other(String::from(
                "The radix for decimal MUST be 10",
            )));
        }

        let exp_separator: &[_] = &['e', 'E'];

        // split slice into base and exponent parts
        let (base_part, exponent_value) = match s.find(exp_separator) {
            // exponent defaults to 0 if (e|E) not found
            None => (s, 0),

            // split and parse exponent field
            Some(loc) => {
                // slice up to `loc` and 1 after to skip the 'e' char
                let (base, exp) = (&s[..loc], &s[loc + 1..]);

                // special consideration for rust 1.0.0 which would not
                // parse a leading '+'
                let exp = match exp.chars().next() {
                    Some('+') => &exp[1..],
                    _ => exp,
                };

                (base, i64::from_str(exp)?)
            }
        };

        // TEMPORARY: Test for emptiness - remove once BigInt supports similar error
        if base_part == "" {
            return Err(ParseBigDecimalError::Empty);
        }

        // split decimal into a digit string and decimal-point offset
        let (digits, decimal_offset): (String, _) = match base_part.find('.') {
            // No dot! pass directly to BigInt
            None => (base_part.to_string(), 0),

            // decimal point found - necessary copy into new string buffer
            Some(loc) => {
                // split into leading and trailing digits
                let (lead, trail) = (&base_part[..loc], &base_part[loc + 1..]);

                // copy all leading characters into 'digits' string
                let mut digits = String::from(lead);

                // copy all trailing characters after '.' into the digits string
                digits.push_str(trail);

                (digits, trail.len() as i64)
            }
        };

        let scale = decimal_offset - exponent_value;
        let big_int = BigInt::from_str_radix(&digits, radix)?;

        Ok(BigDecimal::new(big_int, scale))
    }
}

impl ToPrimitive for BigDecimal {
    fn to_i64(&self) -> Option<i64> {
        match self.sign() {
            Sign::Minus | Sign::Plus => self.with_scale(0).int_val.to_i64(),
            Sign::NoSign => Some(0),
        }
    }
    fn to_u64(&self) -> Option<u64> {
        match self.sign() {
            Sign::Plus => self.with_scale(0).int_val.to_u64(),
            Sign::NoSign => Some(0),
            Sign::Minus => None,
        }
    }

    fn to_f64(&self) -> Option<f64> {
        self.int_val.to_f64().map(|x| x * 10f64.powi(-self.scale as i32))
    }
}

impl From<i64> for BigDecimal {
    #[inline]
    fn from(n: i64) -> Self {
        BigDecimal {
            int_val: BigInt::from(n),
            scale: 0,
        }
    }
}

impl From<u64> for BigDecimal {
    #[inline]
    fn from(n: u64) -> Self {
        BigDecimal {
            int_val: BigInt::from(n),
            scale: 0,
        }
    }
}

impl From<(BigInt, i64)> for BigDecimal {
    #[inline]
    fn from((int_val, scale): (BigInt, i64)) -> Self {
        BigDecimal {
            int_val: int_val,
            scale: scale,
        }
    }
}

impl From<BigInt> for BigDecimal {
    #[inline]
    fn from(int_val: BigInt) -> Self {
        BigDecimal {
            int_val: int_val,
            scale: 0,
        }
    }
}

macro_rules! impl_from_type {
    ($FromType:ty, $AsType:ty) => {
        impl From<$FromType> for BigDecimal {
            #[inline]
            #[allow(clippy::cast_lossless)]
            fn from(n: $FromType) -> Self {
                BigDecimal::from(n as $AsType)
            }
        }
    };
}

impl_from_type!(u8, u64);
impl_from_type!(u16, u64);
impl_from_type!(u32, u64);

impl_from_type!(i8, i64);
impl_from_type!(i16, i64);
impl_from_type!(i32, i64);

impl TryFrom<f32> for BigDecimal {
    type Error = ParseBigDecimalError;

    #[inline]
    fn try_from(n: f32) -> Result<Self, Self::Error> {
        BigDecimal::from_str(&format!("{:.PRECISION$e}", n, PRECISION = ::std::f32::DIGITS as usize))
    }
}

impl TryFrom<f64> for BigDecimal {
    type Error = ParseBigDecimalError;

    #[inline]
    fn try_from(n: f64) -> Result<Self, Self::Error> {
        BigDecimal::from_str(&format!("{:.PRECISION$e}", n, PRECISION = ::std::f64::DIGITS as usize))
    }
}

impl FromPrimitive for BigDecimal {
    #[inline]
    fn from_i64(n: i64) -> Option<Self> {
        Some(BigDecimal::from(n))
    }

    #[inline]
    fn from_u64(n: u64) -> Option<Self> {
        Some(BigDecimal::from(n))
    }

    #[inline]
    fn from_f32(n: f32) -> Option<Self> {
        BigDecimal::try_from(n).ok()
    }

    #[inline]
    fn from_f64(n: f64) -> Option<Self> {
        BigDecimal::try_from(n).ok()
    }
}

impl ToBigInt for BigDecimal {
    fn to_bigint(&self) -> Option<BigInt> {
        Some(self.with_scale(0).int_val)
    }
}

/// Tools to help serializing/deserializing `BigDecimal`s
#[cfg(feature = "serde")]
mod bigdecimal_serde {
    use super::BigDecimal;
    use serde::{de, ser};
    use std::convert::TryFrom;
    use std::fmt;
    use std::str::FromStr;
    #[allow(unused_imports)]
    use num_traits::FromPrimitive;

    impl ser::Serialize for BigDecimal {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: ser::Serializer,
        {
            serializer.collect_str(&self)
        }
    }

    struct BigDecimalVisitor;

    impl<'de> de::Visitor<'de> for BigDecimalVisitor {
        type Value = BigDecimal;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            write!(formatter, "a number or formatted decimal string")
        }

        fn visit_str<E>(self, value: &str) -> Result<BigDecimal, E>
        where
            E: de::Error,
        {
            BigDecimal::from_str(value).map_err(|err| E::custom(format!("{}", err)))
        }

        fn visit_u64<E>(self, value: u64) -> Result<BigDecimal, E>
        where
            E: de::Error,
        {
            Ok(BigDecimal::from(value))
        }

        fn visit_i64<E>(self, value: i64) -> Result<BigDecimal, E>
        where
            E: de::Error,
        {
            Ok(BigDecimal::from(value))
        }

        fn visit_f64<E>(self, value: f64) -> Result<BigDecimal, E>
        where
            E: de::Error,
        {
            BigDecimal::try_from(value).map_err(|err| E::custom(format!("{}", err)))
        }
    }

    #[cfg(not(feature = "string-only"))]
    impl<'de> de::Deserialize<'de> for BigDecimal {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_any(BigDecimalVisitor)
        }
    }

    #[cfg(feature = "string-only")]
    impl<'de> de::Deserialize<'de> for BigDecimal {
        fn deserialize<D>(d: D) -> Result<Self, D::Error>
        where
            D: de::Deserializer<'de>,
        {
            d.deserialize_str(BigDecimalVisitor)
        }
    }

    #[cfg(test)]
    extern crate serde_json;

    #[test]
    fn test_serde_serialize() {
        use std::str::FromStr;

        let vals = vec![
            ("1.0", "1.0"),
            ("0.5", "0.5"),
            ("50", "50"),
            ("50000", "50000"),
            ("1e-3", "0.001"),
            ("1e12", "1000000000000"),
            ("0.25", "0.25"),
            ("12.34", "12.34"),
            ("0.15625", "0.15625"),
            ("0.3333333333333333", "0.3333333333333333"),
            ("3.141592653589793", "3.141592653589793"),
            ("94247.77960769380", "94247.77960769380"),
            ("10.99", "10.99"),
            ("12.0010", "12.0010"),
        ];
        for (s, v) in vals {
            let expected = format!("\"{}\"", v);
            let value = serde_json::to_string(&BigDecimal::from_str(s).unwrap()).unwrap();
            assert_eq!(expected, value);
        }
    }

    #[test]
    fn test_serde_deserialize_str() {
        use std::str::FromStr;

        let vals = vec![
            ("1.0", "1.0"),
            ("0.5", "0.5"),
            ("50", "50"),
            ("50000", "50000"),
            ("1e-3", "0.001"),
            ("1e12", "1000000000000"),
            ("0.25", "0.25"),
            ("12.34", "12.34"),
            ("0.15625", "0.15625"),
            ("0.3333333333333333", "0.3333333333333333"),
            ("3.141592653589793", "3.141592653589793"),
            ("94247.77960769380", "94247.77960769380"),
            ("10.99", "10.99"),
            ("12.0010", "12.0010"),
        ];
        for (s, v) in vals {
            let expected = BigDecimal::from_str(v).unwrap();
            let value: BigDecimal = serde_json::from_str(&format!("\"{}\"", s)).unwrap();
            assert_eq!(expected, value);
        }
    }

    #[test]
    #[cfg(not(feature = "string-only"))]
    fn test_serde_deserialize_int() {
        let vals = vec![0, 1, 81516161, -370, -8, -99999999999];
        for n in vals {
            let expected = BigDecimal::from_i64(n).unwrap();
            let value: BigDecimal = serde_json::from_str(&serde_json::to_string(&n).unwrap()).unwrap();
            assert_eq!(expected, value);
        }
    }

    #[test]
    #[cfg(not(feature = "string-only"))]
    fn test_serde_deserialize_f64() {
        let vals = vec![
            1.0,
            0.5,
            0.25,
            50.0,
            50000.,
            0.001,
            12.34,
            5.0 * 0.03125,
            ::std::f64::consts::PI,
            ::std::f64::consts::PI * 10000.0,
            ::std::f64::consts::PI * 30000.0,
        ];
        for n in vals {
            let expected = BigDecimal::from_f64(n).unwrap();
            let value: BigDecimal = serde_json::from_str(&serde_json::to_string(&n).unwrap()).unwrap();
            assert_eq!(expected, value);
        }
    }
}

#[rustfmt::skip]
#[cfg(test)]
mod bigdecimal_tests {
    use BigDecimal;
    use num_traits::{ToPrimitive, FromPrimitive, Signed, Zero, One};
    use std::convert::TryFrom;
    use std::str::FromStr;
    use num_bigint;

    #[test]
    fn test_sum() {
        let vals = vec![
            BigDecimal::from_f32(2.5).unwrap(),
            BigDecimal::from_f32(0.3).unwrap(),
            BigDecimal::from_f32(0.001).unwrap(),
        ];

        let expected_sum = BigDecimal::from_f32(2.801).unwrap();
        let sum = vals.iter().sum::<BigDecimal>();

        assert_eq!(expected_sum, sum);
    }

    #[test]
    fn test_sum1() {
        let vals = vec![
            BigDecimal::from_f32(0.1).unwrap(),
            BigDecimal::from_f32(0.2).unwrap(),
            // BigDecimal::from_f32(0.001).unwrap(),
        ];

        let expected_sum = BigDecimal::from_f32(0.3).unwrap();
        let sum = vals.iter().sum::<BigDecimal>();

        assert_eq!(expected_sum, sum);
    }

    #[test]
    fn test_to_i64() {
        let vals = vec![
            ("12.34", 12),
            ("3.14", 3),
            ("50", 50),
            ("50000", 50000),
            ("0.001", 0),
            // TODO: Is the desired behaviour to round?
            //("0.56", 1),
        ];
        for (s, ans) in vals {
            let calculated = BigDecimal::from_str(s).unwrap().to_i64().unwrap();

            assert_eq!(ans, calculated);
        }
    }

    #[test]
    fn test_to_f64() {
        let vals = vec![
            ("12.34", 12.34),
            ("3.14", 3.14),
            ("50", 50.),
            ("50000", 50000.),
            ("0.001", 0.001),
        ];
        for (s, ans) in vals {
            let diff = BigDecimal::from_str(s).unwrap().to_f64().unwrap() - ans;
            let diff = diff.abs();

            assert!(diff < 1e-10);
        }
    }

    #[test]
    fn test_from_i8() {
        let vals = vec![
            ("0", 0),
            ("1", 1),
            ("12", 12),
            ("-13", -13),
            ("111", 111),
            ("-128", ::std::i8::MIN),
            ("127", ::std::i8::MAX),
        ];
        for (s, n) in vals {
            let expected = BigDecimal::from_str(s).unwrap();
            let value = BigDecimal::from_i8(n).unwrap();
            assert_eq!(expected, value);
        }
    }

    #[test]
    fn test_from_f32() {
        let vals = vec![
            ("1.0", 1.0),
            ("0.5", 0.5),
            ("0.25", 0.25),
            ("50.", 50.0),
            ("50000", 50000.),
            ("0.001", 0.001),
            ("12.34", 12.34),
            ("0.15625", 5.0 * 0.03125),
            ("3.141593", ::std::f32::consts::PI),
            ("31415.93", ::std::f32::consts::PI * 10000.0),
            ("94247.78", ::std::f32::consts::PI * 30000.0),
            // ("3.14159265358979323846264338327950288f32", ::std::f32::consts::PI),

        ];
        for (s, n) in vals {
            let expected = BigDecimal::from_str(s).unwrap();
            let value = BigDecimal::from_f32(n).unwrap();
            assert_eq!(expected, value);
            // assert_eq!(expected, n);
        }

    }
    #[test]
    fn test_from_f64() {
        let vals = vec![
            ("1.0", 1.0f64),
            ("0.5", 0.5),
            ("50", 50.),
            ("50000", 50000.),
            ("1e-3", 0.001),
            ("0.25", 0.25),
            ("12.34", 12.34),
            // ("12.3399999999999999", 12.34), // <- Precision 16 decimal points
            ("0.15625", 5.0 * 0.03125),
            ("0.3333333333333333", 1.0 / 3.0),
            ("3.141592653589793", ::std::f64::consts::PI),
            ("31415.92653589793", ::std::f64::consts::PI * 10000.0f64),
            ("94247.77960769380", ::std::f64::consts::PI * 30000.0f64),
        ];
        for (s, n) in vals {
            let expected = BigDecimal::from_str(s).unwrap();
            let value = BigDecimal::from_f64(n).unwrap();
            assert_eq!(expected, value);
            // assert_eq!(expected, n);
        }
    }

    #[test]
    fn test_nan_float() {
        assert!(BigDecimal::try_from(std::f32::NAN).is_err());
        assert!(BigDecimal::try_from(std::f64::NAN).is_err());
    }

    #[test]
    fn test_add() {
        let vals = vec![
            ("12.34", "1.234", "13.574"),
            ("12.34", "-1.234", "11.106"),
            ("1234e6", "1234e-6", "1234000000.001234"),
            ("1234e-6", "1234e6", "1234000000.001234"),
            ("18446744073709551616.0", "1", "18446744073709551617"),
            ("184467440737e3380", "0", "184467440737e3380"),
        ];

        for &(x, y, z) in vals.iter() {

            let mut a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            let c = BigDecimal::from_str(z).unwrap();

            assert_eq!(a.clone() + b.clone(), c);

            assert_eq!(a.clone() + &b, c);
            assert_eq!(&a + b.clone(), c);
            assert_eq!(&a + &b, c);

            a += b;
            assert_eq!(a, c);
        }
    }

    #[test]
    fn test_sub() {
        let vals = vec![
            ("12.34", "1.234", "11.106"),
            ("12.34", "-1.234", "13.574"),
            ("1234e6", "1234e-6", "1233999999.998766"),
        ];

        for &(x, y, z) in vals.iter() {

            let mut a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            let c = BigDecimal::from_str(z).unwrap();

            assert_eq!(a.clone() - b.clone(), c);

            assert_eq!(a.clone() - &b, c);
            assert_eq!(&a - b.clone(), c);
            assert_eq!(&a - &b, c);

            a -= b;
            assert_eq!(a, c);
        }
    }

    #[test]
    fn test_mul() {

        let vals = vec![
            ("2", "1", "2"),
            ("12.34", "1.234", "15.22756"),
            ("2e1", "1", "20"),
            ("3", ".333333", "0.999999"),
            ("2389472934723", "209481029831", "500549251119075878721813"),
            ("1e-450", "1e500", ".1e51"),
        ];

        for &(x, y, z) in vals.iter() {

            let mut a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            let c = BigDecimal::from_str(z).unwrap();

            assert_eq!(a.clone() * b.clone(), c);
            assert_eq!(a.clone() * &b, c);
            assert_eq!(&a * b.clone(), c);
            assert_eq!(&a * &b, c);

            a *= b;
            assert_eq!(a, c);
        }
    }

    #[test]
    fn test_div() {
        let vals = vec![
            ("0", "1", "0"),
            ("0", "10", "0"),
            ("2", "1", "2"),
            ("2e1", "1", "2e1"),
            ("10", "10", "1"),
            ("100", "10.0", "1e1"),
            ("20.0", "200", ".1"),
            ("4", "2", "2.0"),
            ("15", "3", "5.0"),
            ("1", "2", "0.5"),
            ("1", "2e-2", "5e1"),
            ("1", "0.2", "5"),
            ("1.0", "0.02", "50"),
            ("1", "0.020", "5e1"),
            ("5.0", "4.00", "1.25"),
            ("5.0", "4.000", "1.25"),
            ("5", "4.000", "1.25"),
            ("5", "4", "125e-2"),
            ("100", "5", "20"),
            ("-50", "5", "-10"),
            ("200", "-5", "-40."),
            ("1", "3", ".3333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333333"),
            ("-2", "-3", ".6666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666666667"),
            ("-12.34", "1.233", "-10.00811030008110300081103000811030008110300081103000811030008110300081103000811030008110300081103001"),
            ("125348", "352.2283", "355.8714617763535752237966114591019517738921035021887792661748076460636467881768727839301952739175132"),
        ];

        for &(x, y, z) in vals.iter() {

            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            let c = BigDecimal::from_str(z).unwrap();

            assert_eq!(a.clone() / b.clone(), c);
            assert_eq!(a.clone() / &b, c);
            assert_eq!(&a / b.clone(), c);
            assert_eq!(&a / &b, c);
            // assert_eq!(q.scale, c.scale);

            // let mut q = a;
            // q /= b;
            // assert_eq!(q, c);
        }
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero_panics() {
        let x = BigDecimal::from_str("3.14").unwrap();
        let _r = x / 0;
    }

    #[test]
    #[should_panic(expected = "Division by zero")]
    fn test_division_by_zero_panics_v2() {
        let x = BigDecimal::from_str("3.14").unwrap();
        let _r = x / BigDecimal::zero();
    }

    #[test]
    fn test_rem() {
        let vals = vec![
            ("100", "5", "0"),
            ("2e1", "1", "0"),
            ("2", "1", "0"),
            ("1", "3", "1"),
            ("1", "0.5", "0"),
            ("1.5", "1", "0.5"),
            ("1", "3e-2", "1e-2"),
            ("10", "0.003", "0.001"),
            ("3", "2", "1"),
            ("-3", "2", "-1"),
            ("3", "-2", "1"),
            ("-3", "-2", "-1"),
            ("12.34", "1.233", "0.01"),
        ];
        for &(x, y, z) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            let c = BigDecimal::from_str(z).unwrap();

            let rem = &a % &b;
            assert_eq!(rem, c, "{} [&{} % &{}] == {}", rem, a, b, c);

            let rem = a.clone() % &b;
            assert_eq!(rem, c, "{} [{} % &{}] == {}", rem, a, b, c);

            let rem = &a % b.clone();
            assert_eq!(rem, c, "{} [&{} % {}] == {}", rem, a, b, c);

            let rem = a.clone() % b.clone();
            assert_eq!(rem, c, "{} [{} % {}] == {}", rem, a, b, c);
        }
        let vals = vec![
            (("100", -2), ("50", -1), "0"),
            (("100", 0), ("50", -1), "0"),
            (("100", -2), ("30", 0), "10"),
            (("100", 0), ("30", -1), "10"),
        ];
        for &((x, xs), (y, ys), z) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().with_scale(xs);
            let b = BigDecimal::from_str(y).unwrap().with_scale(ys);
            let c = BigDecimal::from_str(z).unwrap();
            let rem = &a % &b;
            assert_eq!(rem, c, "{} [{} % {}] == {}", rem, a, b, c);
        }
    }

    #[test]
    fn test_equal() {
        let vals = vec![
            ("2", ".2e1"),
            ("0e1", "0.0"),
            ("0e0", "0.0"),
            ("0e-0", "0.0"),
            ("-0901300e-3", "-901.3"),
            ("-0.901300e+3", "-901.3"),
            ("-0e-1", "-0.0"),
            ("2123121e1231", "212.3121e1235"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_not_equal() {
        let vals = vec![
            ("2", ".2e2"),
            ("1e45", "1e-900"),
            ("1e+900", "1e-900"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert!(a != b, "{} == {}", a, b);
        }
    }

    #[test]
    fn test_hash_equal() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        fn hash<T>(obj: &T) -> u64
            where T: Hash
        {
            let mut hasher = DefaultHasher::new();
            obj.hash(&mut hasher);
            hasher.finish()
        }

        let vals = vec![
            ("1.1234", "1.1234000"),
            ("1.12340000", "1.1234"),
            ("001.1234", "1.1234000"),
            ("001.1234", "0001.1234"),
            ("1.1234000000", "1.1234000"),
            ("1.12340", "1.1234000000"),
            ("-0901300e-3", "-901.3"),
            ("-0.901300e+3", "-901.3"),
            ("100", "100.00"),
            ("100.00", "100"),
            ("0.00", "0"),
            ("0.00", "0.000"),
            ("-0.00", "0.000"),
            ("0.00", "-0.000"),
        ];
        for &(x,y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
            assert_eq!(hash(&a), hash(&b), "hash({}) != hash({})", a, b);
        }
    }

    #[test]
    fn test_hash_not_equal() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        fn hash<T>(obj: &T) -> u64
            where T: Hash
        {
            let mut hasher = DefaultHasher::new();
            obj.hash(&mut hasher);
            hasher.finish()
        }

        let vals = vec![
            ("1.1234", "1.1234001"),
            ("10000", "10"),
            ("10", "10000"),
            ("10.0", "100"),
        ];
        for &(x,y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert!(a != b, "{} == {}", a, b);
            assert!(hash(&a) != hash(&b), "hash({}) == hash({})", a, b);
        }
    }

    #[test]
    fn test_hash_equal_scale() {
        use std::hash::{Hash, Hasher};
        use std::collections::hash_map::DefaultHasher;

        fn hash<T>(obj: &T) -> u64
            where T: Hash
        {
            let mut hasher = DefaultHasher::new();
            obj.hash(&mut hasher);
            hasher.finish()
        }

        let vals = vec![
            ("1234.5678", -2, "1200", 0),
            ("1234.5678", -2, "1200", -2),
            ("1234.5678", 0, "1234.1234", 0),
            ("1234.5678", -3, "1200", -3),
            ("-1234", -2, "-1200", 0),
        ];
        for &(x,xs,y,ys) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().with_scale(xs);
            let b = BigDecimal::from_str(y).unwrap().with_scale(ys);
            assert_eq!(a, b);
            assert_eq!(hash(&a), hash(&b), "hash({}) != hash({})", a, b);
        }
    }

    #[test]
    fn test_with_prec() {
        let vals = vec![
            ("7", 1, "7"),
            ("7", 2, "7.0"),
            ("895", 2, "900"),
            ("8934", 2, "8900"),
            ("8934", 1, "9000"),
            ("1.0001", 5, "1.0001"),
            ("1.0001", 4, "1"),
            ("1.00009", 6, "1.00009"),
            ("1.00009", 5, "1.0001"),
            ("1.00009", 4, "1.000"),
        ];
        for &(x, p, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().with_prec(p);
            assert_eq!(a, BigDecimal::from_str(y).unwrap());
        }
    }


    #[test]
    fn test_digits() {
        let vals = vec![
            ("0", 1),
            ("7", 1),
            ("10", 2),
            ("8934", 4),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            assert_eq!(a.digits(), y);
        }
    }

    #[test]
    fn test_get_rounding_term() {
        use num_bigint::BigInt;
        use super::get_rounding_term;
        let vals = vec![
            ("0", 0),
            ("4", 0),
            ("5", 1),
            ("10", 0),
            ("15", 0),
            ("49", 0),
            ("50", 1),
            ("51", 1),
            ("8934", 1),
            ("9999", 1),
            ("10000", 0),
            ("50000", 1),
            ("99999", 1),
            ("100000", 0),
            ("100001", 0),
            ("10000000000", 0),
            ("9999999999999999999999999999999999999999", 1),
            ("10000000000000000000000000000000000000000", 0),
        ];
        for &(x, y) in vals.iter() {
            let a = BigInt::from_str(x).unwrap();
            assert_eq!(get_rounding_term(&a), y, "{}", x);
        }
    }

    #[test]
    fn test_abs() {
        let vals = vec![
            ("10", "10"),
            ("-10", "10"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().abs();
            let b = BigDecimal::from_str(y).unwrap();
            assert!(a == b, "{} == {}", a, b);
        }
    }

    #[test]
    fn test_count_decimal_digits() {
        use num_bigint::BigInt;
        use super::count_decimal_digits;
        let vals = vec![
            ("10", 2),
            ("1", 1),
            ("9", 1),
            ("999", 3),
            ("1000", 4),
            ("9900", 4),
            ("9999", 4),
            ("10000", 5),
            ("99999", 5),
            ("100000", 6),
            ("999999", 6),
            ("1000000", 7),
            ("9999999", 7),
            ("999999999999", 12),
            ("999999999999999999999999", 24),
            ("999999999999999999999999999999999999999999999999", 48),
            ("999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", 96),
            ("199999911199999999999999999999999999999999999999999999999999999999999999999999999999999999999000", 96),
            ("999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999991", 192),
            ("199999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999999", 192),
        ];
        for &(x, y) in vals.iter() {
            let a = BigInt::from_str(x).unwrap();
            let b = count_decimal_digits(&a);
            assert_eq!(b, y);
        }
    }

    #[test]
    fn test_half() {
        let vals = vec![
            ("100", "50."),
            ("2", "1"),
            (".2", ".1"),
            ("42", "21"),
            ("3", "1.5"),
            ("99", "49.5"),
            ("3.141592653", "1.5707963265"),
            ("3.1415926536", "1.5707963268"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().half();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.scale, b.scale);
        }
    }

    #[test]
    fn test_round() {
        let test_cases = vec![
            ("1.45", 1, "1.5"),
            ("1.444445", 1, "1.4"),
            ("1.44", 1, "1.4"),
            ("0.444", 2, "0.44"),
            ("0.0045", 2, "0.00"),
            ("-1.555", 2, "-1.56"),
            ("-1.555", 99, "-1.555"),
            ("5.5", 0, "6"),
            ("-1", -1, "0"),
            ("5", -1, "10"),
            ("44", -1, "40"),
            ("44", -99, "0"),
            ("1.4499999999", 1, "1.4"),
            ("-1.4499999999", 1, "-1.4"),
            ("1.449999999", 1, "1.4"),
            ("-9999.444455556666", 10, "-9999.4444555567"),
            ("-12345678987654321.123456789", 8, "-12345678987654321.12345679"),
        ];
        for &(x, digits, y) in test_cases.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a.round(digits), b);
        }
    }

    #[test]
    fn test_is_integer() {
        let true_vals = vec![
            "100",
            "100.00",
            "1724e4",
            "31.47e8",
            "-31.47e8",
            "-0.0",
        ];

        let false_vals = vec![
            "100.1",
            "0.001",
            "3147e-3",
            "3147e-8",
            "-0.01",
            "-1e-3",
        ];

        for s in true_vals {
            let d = BigDecimal::from_str(&s).unwrap();
            assert!(d.is_integer());
        }

        for s in false_vals {
            let d = BigDecimal::from_str(&s).unwrap();
            assert!(!d.is_integer());
        }
    }

    #[test]
    fn test_inverse() {
        let vals = vec![
            ("100", "0.01"),
            ("2", "0.5"),
            (".2", "5"),
            ("3.141592653", "0.3183098862435492205742690218851870990799646487459493049686604293188738877535183744268834079171116523"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap();
            let i = a.inverse();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(i, b);
            assert_eq!(BigDecimal::from(1)/&a, b);
            assert_eq!(i.inverse(), a);
            // assert_eq!(a.scale, b.scale, "scale mismatch ({} != {}", a, b);
        }
    }

    #[test]
    fn test_sqrt() {
        let vals = vec![
            ("1e-232", "1e-116"),
            ("1.00", "1"),
            ("1.001", "1.000499875062460964823258287700109753027590031219780479551442971840836093890879944856933288426795152"),
            ("100", "10"),
            ("49", "7"),
            (".25", ".5"),
            ("0.0152399025", ".12345"),
            ("152399025", "12345"),
            (".00400", "0.06324555320336758663997787088865437067439110278650433653715009705585188877278476442688496216758600590"),
            (".1", "0.3162277660168379331998893544432718533719555139325216826857504852792594438639238221344248108379300295"),
            ("2", "1.414213562373095048801688724209698078569671875376948073176679737990732478462107038850387534327641573"),
            ("125348", "354.0451948551201563108487193176101314241016013304294520812832530590100407318465590778759640828114535"),
            ("18446744073709551616.1099511", "4294967296.000000000012799992691725492477397918722952224079252026972356303360555051219312462698703293"),
            ("3.141592653589793115997963468544185161590576171875", "1.772453850905515992751519103139248439290428205003682302442979619028063165921408635567477284443197875"),
            (".000000000089793115997963468544185161590576171875", "0.000009475922962855041517561783740144225422359796851494316346796373337470068631250135521161989831460407155"),
            ("0.7177700109762963922745342343167413624881759290454997218753321040760896053150388903350654937434826216803814031987652326749140535150336357405672040727695124057298138872112244784753994931999476811850580200000000000000000000000000000000", "0.8472130847527653667042980517799020703921106560594525833177762276594388966885185567535692987624493813"),
            ("0.01234567901234567901234567901234567901234567901234567901234567901234567901234567901234567901234567901", "0.1111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111111"),
            ("0.1108890000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000444", "0.3330000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000667"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().sqrt().unwrap();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_big_sqrt() {
        use num_bigint::BigInt;
        let vals = vec![
            (("2", -70), "141421356237309504880168872420969807.8569671875376948073176679737990732478462107038850387534327641573"),
            (("3", -170), "17320508075688772935274463415058723669428052538103806280558069794519330169088000370811.46186757248576"),
            (("9", -199), "9486832980505137995996680633298155601158665417975650480572514558377783315917714664032744325137900886"),
            (("7", -200), "26457513110645905905016157536392604257102591830824501803683344592010688232302836277603928864745436110"),
            (("777", -204), "2.787471972953270789531596912111625325974789615194854615319795902911796043681078997362635440358922503E+103"),
            (("7", -600), "2.645751311064590590501615753639260425710259183082450180368334459201068823230283627760392886474543611E+300"),
            (("2", -900), "1.414213562373095048801688724209698078569671875376948073176679737990732478462107038850387534327641573E+450"),
            (("7", -999), "8.366600265340755479781720257851874893928153692986721998111915430804187725943170098308147119649515362E+499"),
            (("74908163946345982392040522594123773796", -999), "2.736935584670307552030924971360722787091742391079630976117950955395149091570790266754718322365663909E+518"),
            (("20", -1024), "4.472135954999579392818347337462552470881236719223051448541794490821041851275609798828828816757564550E512"),
            (("3", 1025), "5.477225575051661134569697828008021339527446949979832542268944497324932771227227338008584361638706258E-513"),
        ];
        for &((s, scale), e) in vals.iter() {
            let expected = BigDecimal::from_str(e).unwrap();

            let sqrt = BigDecimal::new(BigInt::from_str(s).unwrap(), scale).sqrt().unwrap();
            assert_eq!(sqrt, expected);
        }
    }

    #[test]
    fn test_cbrt() {
        let vals = vec![
            ("0.00", "0"),
            ("1.00", "1"),
            ("1.001", "1.000333222283909495175449559955220102010284758197360454054345461242739715702641939155238095670636841"),
            ("10", "2.154434690031883721759293566519350495259344942192108582489235506346411106648340800185441503543243276"),
            ("-59283293e25", "-84006090355.84281237113712383191213626687332139035750444925827809487776780721673264524620270275301685"),
            ("94213372931e-127", "2.112049945275324414051072540210070583697242797173805198575907094646677475250362108901530353886613160E-39"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().cbrt();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_double() {
        let vals = vec![
            ("1", "2"),
            ("1.00", "2.00"),
            ("1.50", "3.00"),
            ("5", "10"),
            ("5.0", "10.0"),
            ("5.5", "11.0"),
            ("5.05", "10.10"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().double();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.scale, b.scale);
        }
    }

    #[test]
    fn test_square() {
        let vals = vec![
            ("1.00", "1.00"),
            ("1.5", "2.25"),
            ("1.50", "2.2500"),
            ("5", "25"),
            ("5.0", "25.00"),
            ("-5.0", "25.00"),
            ("5.5", "30.25"),
            ("0.80", "0.6400"),
            ("0.01234", "0.0001522756"),
            ("3.1415926", "9.86960406437476"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().square();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.scale, b.scale);
        }
    }

    #[test]
    fn test_cube() {
        let vals = vec![
            ("1.00", "1.00"),
            ("1.50", "3.375000"),
            ("5", "125"),
            ("5.0", "125.000"),
            ("5.00", "125.000000"),
            ("-5", "-125"),
            ("-5.0", "-125.000"),
            ("2.01", "8.120601"),
            ("5.5", "166.375"),
            ("0.01234", "0.000001879080904"),
            ("3.1415926", "31.006275093569669642776"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().cube();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
            assert_eq!(a.scale, b.scale);
        }
    }

    #[test]
    fn test_exp() {
        let vals = vec![
            ("0", "1"),
            ("1", "2.718281828459045235360287471352662497757247093699959574966967627724076630353547594571382178525166427"),
            ("1.01", "2.745601015016916493989776316660387624073750819595962291667398087987297168243899027802501018008905180"),
            ("0.5", "1.648721270700128146848650787814163571653776100710148011575079311640661021194215608632776520056366643"),
            ("-1", "0.3678794411714423215955237701614608674458111310317678345078368016974614957448998033571472743459196437"),
            ("-0.01", "0.9900498337491680535739059771800365577720790812538374668838787452931477271687452950182155307793838110"),
            ("-10.04", "0.00004361977305405268676261569570537884674661515701779752139657120453194647205771372804663141467275928595"),
            //("-1000.04", "4.876927702336787390535723208392195312680380995235400234563172353460484039061383367037381490416091595E-435"),
            ("-20.07", "1.921806899438469499721914055500607234723811054459447828795824348465763824284589956630853464778332349E-9"),
            ("10", "22026.46579480671651695790064528424436635351261855678107423542635522520281857079257519912096816452590"),
            ("20", "485165195.4097902779691068305415405586846389889448472543536108003159779961427097401659798506527473494"),
            //("777.7", "5.634022488451236612534495413455282583175841288248965283178668787259870456538271615076138061788051442E+337"),
        ];
        for &(x, y) in vals.iter() {
            let a = BigDecimal::from_str(x).unwrap().exp();
            let b = BigDecimal::from_str(y).unwrap();
            assert_eq!(a, b);
        }
    }

    #[test]
    fn test_from_str() {
        let vals = vec![
            ("1331.107", 1331107, 3),
            ("1.0", 10, 1),
            ("2e1", 2, -1),
            ("0.00123", 123, 5),
            ("-123", -123, 0),
            ("-1230", -1230, 0),
            ("12.3", 123, 1),
            ("123e-1", 123, 1),
            ("1.23e+1", 123, 1),
            ("1.23E+3", 123, -1),
            ("1.23E-8", 123, 10),
            ("-1.23E-10", -123, 12),
        ];

        for &(source, val, scale) in vals.iter() {
            let x = BigDecimal::from_str(source).unwrap();
            assert_eq!(x.int_val.to_i32().unwrap(), val);
            assert_eq!(x.scale, scale);
        }
    }

    #[test]
    fn test_fmt() {
        let vals = vec![
            // b  s   ( {}        {:.1}     {:.4}      {:4.1}  {:+05.1}  {:<4.1}
            (1, 0,  (  "1",     "1.0",    "1.0000",  " 1.0",  "+01.0",   "1.0 " )),
            (1, 1,  (  "0.1",   "0.1",    "0.1000",  " 0.1",  "+00.1",   "0.1 " )),
            (1, 2,  (  "0.01",  "0.0",    "0.0100",  " 0.0",  "+00.0",   "0.0 " )),
            (1, -2, ("100",   "100.0",  "100.0000", "100.0", "+100.0", "100.0" )),
            (-1, 0, ( "-1",    "-1.0",   "-1.0000",  "-1.0",  "-01.0",  "-1.0" )),
            (-1, 1, ( "-0.1",  "-0.1",   "-0.1000",  "-0.1",  "-00.1",  "-0.1" )),
            (-1, 2, ( "-0.01", "-0.0",   "-0.0100",  "-0.0",  "-00.0",  "-0.0" )),
        ];
        for (i, scale, results) in vals {
            let x = BigDecimal::new(num_bigint::BigInt::from(i), scale);
            assert_eq!(format!("{}", x), results.0);
            assert_eq!(format!("{:.1}", x), results.1);
            assert_eq!(format!("{:.4}", x), results.2);
            assert_eq!(format!("{:4.1}", x), results.3);
            assert_eq!(format!("{:+05.1}", x), results.4);
            assert_eq!(format!("{:<4.1}", x), results.5);
        }
    }

    #[test]
    fn test_debug() {
        let vals = vec![
            ("BigDecimal(\"123.456\")", "123.456"),
            ("BigDecimal(\"123.400\")", "123.400"),
            ("BigDecimal(\"1.20\")", "01.20"),
            // ("BigDecimal(\"1.2E3\")", "01.2E3"), <- ambiguous precision
            ("BigDecimal(\"1200\")", "01.2E3"),
        ];

        for (expected, source) in vals {
            let var = BigDecimal::from_str(source).unwrap();
            assert_eq!(format!("{:?}", var), expected);
        }
    }

    #[test]
    fn test_signed() {
        assert!(!BigDecimal::zero().is_positive());
        assert!(!BigDecimal::one().is_negative());

        assert!(BigDecimal::one().is_positive());
        assert!((-BigDecimal::one()).is_negative());
        assert!((-BigDecimal::one()).abs().is_positive());
    }

    #[test]
    fn test_normalize() {
        use num_bigint::BigInt;

        let vals = vec![
            (BigDecimal::new(BigInt::from(10), 2),
            BigDecimal::new(BigInt::from(1), 1),
            "0.1"),
            (BigDecimal::new(BigInt::from(132400), -4),
            BigDecimal::new(BigInt::from(1324), -6),
            "1324000000"),
            (BigDecimal::new(BigInt::from(1_900_000), 3),
            BigDecimal::new(BigInt::from(19), -2),
            "1900"),
            (BigDecimal::new(BigInt::from(0), -3),
            BigDecimal::zero(),
            "0"),
            (BigDecimal::new(BigInt::from(0), 5),
            BigDecimal::zero(),
            "0"),
        ];

        for (not_normalized, normalized, string) in vals {
            assert_eq!(not_normalized.normalized(), normalized);
            assert_eq!(not_normalized.normalized().to_string(), string);
            assert_eq!(normalized.to_string(), string);
        }
    }

    #[test]
    #[should_panic(expected = "InvalidDigit")]
    fn test_bad_string_nan() {
        BigDecimal::from_str("hello").unwrap();
    }
    #[test]
    #[should_panic(expected = "Empty")]
    fn test_bad_string_empty() {
        BigDecimal::from_str("").unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidDigit")]
    fn test_bad_string_invalid_char() {
        BigDecimal::from_str("12z3.12").unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidDigit")]
    fn test_bad_string_nan_exponent() {
        BigDecimal::from_str("123.123eg").unwrap();
    }
    #[test]
    #[should_panic(expected = "Empty")]
    fn test_bad_string_empty_exponent() {
        BigDecimal::from_str("123.123E").unwrap();
    }
    #[test]
    #[should_panic(expected = "InvalidDigit")]
    fn test_bad_string_multiple_decimal_points() {
        BigDecimal::from_str("123.12.45").unwrap();
    }
    #[test]
    #[should_panic(expected = "Empty")]
    fn test_bad_string_only_decimal() {
        BigDecimal::from_str(".").unwrap();
    }
    #[test]
    #[should_panic(expected = "Empty")]
    fn test_bad_string_only_decimal_and_exponent() {
        BigDecimal::from_str(".e4").unwrap();
    }
}
