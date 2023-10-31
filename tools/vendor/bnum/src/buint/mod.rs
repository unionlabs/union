#[cfg(debug_assertions)]
use crate::errors::{self, option_expect};

use crate::digit;
use crate::doc;
use crate::ExpType;
use core::mem::MaybeUninit;

#[cfg(feature = "serde")]
use ::{
    serde::{Deserialize, Serialize},
    serde_big_array::BigArray,
};

use core::default::Default;

use core::iter::{Iterator, Product, Sum};

#[cfg(feature = "arbitrary")]
use arbitrary::Arbitrary;

macro_rules! mod_impl {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        /// Unsigned integer type composed of
        #[doc = concat!("`", stringify!($Digit), "`")]
        /// digits, of arbitrary fixed size which must be known at compile time.
        ///
        /// Digits are stored in little endian (least significant digit first). This integer type aims to exactly replicate the behaviours of Rust's built-in unsigned integer types: `u8`, `u16`, `u32`, `u64`, `u128` and `usize`. The const generic parameter `N` is the number of
        #[doc = concat!("`", stringify!($Digit), "`")]
        /// digits that are stored.
        ///
        #[doc = doc::arithmetic_doc!($BUint)]

        // Clippy: we can allow derivation of `Hash` and manual implementation of `PartialEq` as the derived `PartialEq` would be the same except we make our implementation const.
        #[allow(clippy::derive_hash_xor_eq)]
        #[derive(Clone, Copy, Hash)]
        #[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
        #[cfg_attr(feature = "arbitrary", derive(Arbitrary))]
        pub struct $BUint<const N: usize> {
            #[cfg_attr(feature = "serde", serde(with = "BigArray"))]
            pub(crate) digits: [$Digit; N],
        }

        impl<const N: usize> $BUint<N> {
            #[doc = doc::count_ones!(U 1024)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn count_ones(self) -> ExpType {
                let mut ones = 0;
                let mut i = 0;
                while i < N {
                    ones += self.digits[i].count_ones() as ExpType;
                    i += 1;
                }
                ones
            }

            #[doc = doc::count_zeros!(U 1024)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn count_zeros(self) -> ExpType {
                let mut zeros = 0;
                let mut i = 0;
                while i < N {
                    zeros += self.digits[i].count_zeros() as ExpType;
                    i += 1;
                }
                zeros
            }

            #[doc = doc::leading_zeros!(U 1024)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn leading_zeros(self) -> ExpType {
                let mut zeros = 0;
                let mut i = N;
                while i > 0 {
                    i -= 1;
                    let digit = self.digits[i];
                    zeros += digit.leading_zeros() as ExpType;
                    if digit != $Digit::MIN {
                        break;
                    }
                }
                zeros
            }

            #[doc = doc::trailing_zeros!(U 1024)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn trailing_zeros(self) -> ExpType {
                let mut zeros = 0;
                let mut i = 0;
                while i < N {
                    let digit = self.digits[i];
                    zeros += digit.trailing_zeros() as ExpType;
                    if digit != $Digit::MIN {
                        break;
                    }
                    i += 1;
                }
                zeros
            }

            #[doc = doc::leading_ones!(U 1024, MAX)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn leading_ones(self) -> ExpType {
                let mut ones = 0;
                let mut i = N;
                while i > 0 {
                    i -= 1;
                    let digit = self.digits[i];
                    ones += digit.leading_ones() as ExpType;
                    if digit != $Digit::MAX {
                        break;
                    }
                }
                ones
            }

            #[doc = doc::trailing_ones!(U 1024)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn trailing_ones(self) -> ExpType {
                let mut ones = 0;
                let mut i = 0;
                while i < N {
                    let digit = self.digits[i];
                    ones += digit.trailing_ones() as ExpType;
                    if digit != $Digit::MAX {
                        break;
                    }
                    i += 1;
                }
                ones
            }

            #[inline]
            const unsafe fn rotate_digits_left(self, n: usize) -> Self {
                let uninit = MaybeUninit::<[$Digit; N]>::uninit();
                let digits_ptr = self.digits.as_ptr();
                let uninit_ptr = uninit.as_ptr().cast_mut() as *mut $Digit; // TODO: can change to as_mut_ptr() when const_mut_refs is stabilised

                digits_ptr.copy_to_nonoverlapping(uninit_ptr.add(n), N - n);
                digits_ptr.add(N - n).copy_to_nonoverlapping(uninit_ptr, n);
                Self::from_digits(uninit.assume_init())
            }

            #[inline]
            const unsafe fn unchecked_rotate_left(self, rhs: ExpType) -> Self {
                let digit_shift = (rhs >> digit::$Digit::BIT_SHIFT) as usize;
                let bit_shift = rhs & digit::$Digit::BITS_MINUS_1;

                let mut out = self.rotate_digits_left(digit_shift);

                if bit_shift != 0 {
                    let carry_shift = digit::$Digit::BITS - bit_shift;
                    let mut carry = 0;

                    let mut i = 0;
                    while i < N {
                        let current_digit = out.digits[i];
                        out.digits[i] = (current_digit << bit_shift) | carry;
                        carry = current_digit >> carry_shift;
                        i += 1;
                    }
                    out.digits[0] |= carry;
                }

                out
            }

            const BITS_MINUS_1: ExpType = (Self::BITS - 1) as ExpType;

            #[doc = doc::rotate_left!(U 256, "u")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn rotate_left(self, n: ExpType) -> Self {
                unsafe {
                    self.unchecked_rotate_left(n & Self::BITS_MINUS_1)
                }
            }

            #[doc = doc::rotate_right!(U 256, "u")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn rotate_right(self, n: ExpType) -> Self {
                let n = n & Self::BITS_MINUS_1;
                unsafe {
                    self.unchecked_rotate_left(Self::BITS as ExpType - n)
                }
            }

            const N_MINUS_1: usize = N - 1;

            #[doc = doc::swap_bytes!(U 256, "u")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn swap_bytes(self) -> Self {
                let mut uint = Self::ZERO;
                let mut i = 0;
                while i < N {
                    uint.digits[i] = self.digits[Self::N_MINUS_1 - i].swap_bytes();
                    i += 1;
                }
                uint
            }

            #[doc = doc::reverse_bits!(U 256, "u")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn reverse_bits(self) -> Self {
                let mut uint = Self::ZERO;
                let mut i = 0;
                while i < N {
                    uint.digits[i] = self.digits[Self::N_MINUS_1 - i].reverse_bits();
                    i += 1;
                }
                uint
            }

            #[doc = doc::pow!(U 256)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn pow(self, exp: ExpType) -> Self {
                #[cfg(debug_assertions)]
                return option_expect!(
                    self.checked_pow(exp),
                    errors::err_msg!("attempt to calculate power with overflow")
                );
                #[cfg(not(debug_assertions))]
                self.wrapping_pow(exp)
            }

            crate::nightly::const_fns! {
                #[doc = doc::div_euclid!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn div_euclid(self, rhs: Self) -> Self {
                    self.wrapping_div_euclid(rhs)
                }


                #[doc = doc::rem_euclid!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn rem_euclid(self, rhs: Self) -> Self {
                    self.wrapping_rem_euclid(rhs)
                }
            }

            #[doc = doc::doc_comment! {
                U 256,
                "Returns `true` if and only if `self == 2^k` for some integer `k`.",

                "let n = " stringify!(U256) "::from(1u16 << 14);\n"
                "assert!(n.is_power_of_two());\n"
                "let m = " stringify!(U256) "::from(100u8);\n"
                "assert!(!m.is_power_of_two());"
            }]
            #[must_use]
            #[inline]
            pub const fn is_power_of_two(self) -> bool {
                let mut i = 0;
                let mut ones = 0;
                while i < N {
                    ones += (&self.digits)[i].count_ones();
                    if ones > 1 {
                        return false;
                    }
                    i += 1;
                }
                ones == 1
            }

            #[doc = doc::next_power_of_two!(U 256, "0", "ZERO")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn next_power_of_two(self) -> Self {
                #[cfg(debug_assertions)]
                return option_expect!(
                    self.checked_next_power_of_two(),
                    errors::err_msg!("attempt to calculate next power of two with overflow")
                );
                #[cfg(not(debug_assertions))]
                self.wrapping_next_power_of_two()
            }

            #[doc = doc::ilog2!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn ilog2(self) -> ExpType {
                #[cfg(debug_assertions)]
                return option_expect!(
                    self.checked_ilog2(),
                    errors::err_msg!("attempt to calculate ilog2 of zero")
                );
                #[cfg(not(debug_assertions))]
                match self.checked_ilog2() {
                    Some(n) => n,
                    None => 0,
                }
            }

            crate::nightly::const_fns! {
                #[doc = doc::ilog10!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn ilog10(self) -> ExpType {
                    #[cfg(debug_assertions)]
                    return option_expect!(self.checked_ilog10(), errors::err_msg!("attempt to calculate ilog10 of zero"));
                    #[cfg(not(debug_assertions))]
                    match self.checked_ilog10() {
                        Some(n) => n,
                        None => 0,
                    }
                }

                #[doc = doc::ilog!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn ilog(self, base: Self) -> ExpType {
                    #[cfg(debug_assertions)]
                    return option_expect!(self.checked_ilog(base), errors::err_msg!("attempt to calculate ilog of zero or ilog with base < 2"));
                    #[cfg(not(debug_assertions))]
                    match self.checked_ilog(base) {
                        Some(n) => n,
                        None => 0,
                    }
                }
            }

            #[doc = doc::abs_diff!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn abs_diff(self, other: Self) -> Self {
                if self.lt(other) {
                    other.wrapping_sub(self)
                } else {
                    self.wrapping_sub(other)
                }
            }

            crate::nightly::const_fns! {
                #[doc = doc::next_multiple_of!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn next_multiple_of(self, rhs: Self) -> Self {
                    let rem = self.wrapping_rem(rhs);
                    if rem.is_zero() {
                        self
                    } else {
                        self.add(rhs.sub(rem))
                    }
                }

                #[doc = doc::div_floor!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn div_floor(self, rhs: Self) -> Self {
                    self.wrapping_div(rhs)
                }

                #[doc = doc::div_ceil!(U)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn div_ceil(self, rhs: Self) -> Self {
                    let (div, rem) = self.div_rem(rhs);
                    if rem.is_zero() {
                        div
                    } else {
                        div.add(Self::ONE)
                    }
                }
            }
        }

        impl<const N: usize> $BUint<N> {
            #[inline]
            pub(crate) const unsafe fn unchecked_shl_internal(u: $BUint<N>, rhs: ExpType) -> $BUint<N> {
                let mut out = $BUint::ZERO;
                let digit_shift = (rhs >> digit::$Digit::BIT_SHIFT) as usize;
                let bit_shift = rhs & digit::$Digit::BITS_MINUS_1;

                let num_copies = N - digit_shift;

                u.digits.as_ptr().copy_to_nonoverlapping(out.digits.as_ptr().cast_mut().add(digit_shift), num_copies); // TODO: can change to out.digits.as_mut_ptr() when const_mut_refs is stabilised

                if bit_shift != 0 {
                    let carry_shift = digit::$Digit::BITS - bit_shift;
                    let mut carry = 0;

                    let mut i = digit_shift;
                    while i < N {
                        let current_digit = out.digits[i];
                        out.digits[i] = (current_digit << bit_shift) | carry;
                        carry = current_digit >> carry_shift;
                        i += 1;
                    }
                }

                out
            }

            #[inline]
            pub(crate) const unsafe fn unchecked_shr_pad_internal<const PAD: $Digit>(u: $BUint<N>, rhs: ExpType) -> $BUint<N> {
                let mut out = $BUint::from_digits([PAD; N]);
                let digit_shift = (rhs >> digit::$Digit::BIT_SHIFT) as usize;
                let bit_shift = rhs & digit::$Digit::BITS_MINUS_1;

                let num_copies = N - digit_shift;

                u.digits.as_ptr().add(digit_shift).copy_to_nonoverlapping(out.digits.as_ptr().cast_mut(), num_copies); // TODO: can change to out.digits.as_mut_ptr() when const_mut_refs is stabilised

                if bit_shift != 0 {
                    let carry_shift = digit::$Digit::BITS - bit_shift;
                    let mut carry = 0;

                    let mut i = num_copies;
                    while i > 0 {
                        i -= 1;
                        let current_digit = out.digits[i];
                        out.digits[i] = (current_digit >> bit_shift) | carry;
                        carry = current_digit << carry_shift;
                    }

                    if PAD == $Digit::MAX {
                        out.digits[num_copies - 1] |= $Digit::MAX << carry_shift;
                    }
                }

                out
            }

            pub(crate) const unsafe fn unchecked_shr_internal(u: $BUint<N>, rhs: ExpType) -> $BUint<N> {
                Self::unchecked_shr_pad_internal::<{$Digit::MIN}>(u, rhs)
            }

            #[doc = doc::bits!(U 256)]
            #[must_use]
            #[inline]
            pub const fn bits(&self) -> ExpType {
                Self::BITS as ExpType - self.leading_zeros()
            }

            #[doc = doc::bit!(U 256)]
            #[must_use]
            #[inline]
            pub const fn bit(&self, index: ExpType) -> bool {
                let digit = self.digits[index as usize >> digit::$Digit::BIT_SHIFT];
                digit & (1 << (index & digit::$Digit::BITS_MINUS_1)) != 0
            }

            /// Returns an integer whose value is `2^power`. This is faster than using a shift left on `Self::ONE`.
            ///
            /// # Panics
            ///
            /// This function will panic if `power` is greater than or equal to `Self::BITS`.
            ///
            /// # Examples
            ///
            /// ```
            /// use bnum::types::U256;
            ///
            /// let power = 11;
            /// assert_eq!(U256::power_of_two(11), (1u128 << 11).into());
            /// ```
            #[must_use]
            #[inline]
            pub const fn power_of_two(power: ExpType) -> Self {
                let mut out = Self::ZERO;
                out.digits[power as usize >> digit::$Digit::BIT_SHIFT] = 1 << (power & (digit::$Digit::BITS - 1));
                out
            }

            /// Returns the digits stored in `self` as an array. Digits are little endian (least significant digit first).
            #[must_use]
            #[inline(always)]
            pub const fn digits(&self) -> &[$Digit; N] {
                &self.digits
            }

            /// Creates a new unsigned integer from the given array of digits. Digits are stored as little endian (least significant digit first).
            #[must_use]
            #[inline(always)]
            pub const fn from_digits(digits: [$Digit; N]) -> Self {
                Self { digits }
            }

            /// Creates a new unsigned integer from the given digit. The given digit is stored as the least significant digit.
            #[must_use]
            #[inline(always)]
            pub const fn from_digit(digit: $Digit) -> Self {
                let mut out = Self::ZERO;
                out.digits[0] = digit;
                out
            }

            #[doc = doc::is_zero!(U 256)]
            #[must_use]
            #[inline]
            pub const fn is_zero(&self) -> bool {
                let mut i = 0;
                while i < N {
                    if (&self.digits)[i] != 0 {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            #[doc = doc::is_one!(U 256)]
            #[must_use]
            #[inline]
            pub const fn is_one(&self) -> bool {
                if N == 0 || self.digits[0] != 1 {
                    return false;
                }
                let mut i = 1;
                while i < N {
                    if (&self.digits)[i] != 0 {
                        return false;
                    }
                    i += 1;
                }
                true
            }

            #[inline]
            pub(crate) const fn last_digit_index(&self) -> usize {
                let mut index = 0;
                let mut i = 1;
                while i < N {
                    if (&self.digits)[i] != 0 {
                        index = i;
                    }
                    i += 1;
                }
                index
            }

            #[inline]
            pub(crate) const fn to_exp_type(self) -> Option<ExpType> {
                let last_index = self.last_digit_index();
                if self.digits[last_index] == 0 {
                    return Some(0);
                }
                if last_index >= ExpType::BITS as usize >> digit::$Digit::BIT_SHIFT {
                    return None;
                }
                let mut out = 0;
                let mut i = 0;
                while i <= last_index {
                    out |= (self.digits[i] as ExpType) << (i << digit::$Digit::BIT_SHIFT);
                    i += 1;
                }
                Some(out)
            }

            #[allow(unused)]
            #[inline]
            fn square(self) -> Self {
                // TODO: optimise this method, this will make exponentiation by squaring faster
                self * self
            }
        }

        impl<const N: usize> Default for $BUint<N> {
            #[doc = doc::default!()]
            #[inline]
            fn default() -> Self {
                Self::ZERO
            }
        }

        impl<const N: usize> Product<Self> for $BUint<N> {
            #[inline]
            fn product<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ONE, |a, b| a * b)
            }
        }

        impl<'a, const N: usize> Product<&'a Self> for $BUint<N> {
            #[inline]
            fn product<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::ONE, |a, b| a * b)
            }
        }

        impl<const N: usize> Sum<Self> for $BUint<N> {
            #[inline]
            fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + b)
            }
        }

        impl<'a, const N: usize> Sum<&'a Self> for $BUint<N> {
            #[inline]
            fn sum<I: Iterator<Item = &'a Self>>(iter: I) -> Self {
                iter.fold(Self::ZERO, |a, b| a + b)
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::{debug_skip, test_bignum, types::utest};
                use crate::test::types::big_types::$Digit::*;

                crate::int::tests!(utest);

                test_bignum! {
                    function: <utest>::next_power_of_two(a: utest),
                    skip: debug_skip!(a.checked_next_power_of_two().is_none())
                }

                test_bignum! {
                    function: <utest>::is_power_of_two(a: utest)
                }

                #[test]
                fn bit() {
                    let u = UTEST::from(0b001010100101010101u64);
                    assert!(u.bit(0));
                    assert!(!u.bit(1));
                    assert!(!u.bit(17));
                    assert!(!u.bit(16));
                    assert!(u.bit(15));
                }

                #[test]
                fn is_zero() {
                    assert!(UTEST::MIN.is_zero());
                    assert!(!UTEST::MAX.is_zero());
                    assert!(!UTEST::ONE.is_zero());
                }

                #[test]
                fn is_one() {
                    assert!(UTEST::ONE.is_one());
                    assert!(!UTEST::MAX.is_one());
                    assert!(!UTEST::ZERO.is_one());
                }

                #[test]
                fn bits() {
                    let u = UTEST::from(0b1001010100101010101u128);
                    assert_eq!(u.bits(), 19);

                    let u = UTEST::power_of_two(34);
                    assert_eq!(u.bits(), 35);
                }

                #[test]
                fn default() {
                    assert_eq!(UTEST::default(), utest::default().into());
                }
            }
        }
    };
}

crate::main_impl!(mod_impl);

mod bigint_helpers;
mod cast;
mod checked;
mod cmp;
mod const_trait_fillers;
mod consts;
mod convert;
mod endian;
mod fmt;
#[cfg(feature = "numtraits")]
mod numtraits;
mod ops;
mod overflowing;
mod radix;
mod saturating;
mod unchecked;
mod wrapping;
