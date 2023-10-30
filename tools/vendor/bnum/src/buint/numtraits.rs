macro_rules! to_int {
    { $Digit: ident; $($name: ident -> $int: ty), * }  => {
        $(
            #[inline]
            fn $name(&self) -> Option<$int> {
                let mut out = 0;
                let mut i = 0;
                if $Digit::BITS > <$int>::BITS {
                    let small = self.digits[i] as $int;
                    let trunc = small as $Digit;
                    if self.digits[i] != trunc {
                        return None;
                    }
                    out = small;
                    i = 1;
                } else {
                    loop {
                        let shift = i << crate::digit::$Digit::BIT_SHIFT;
                        if i >= N || shift >= <$int>::BITS as usize {
                            break;
                        }
                        out |= self.digits[i] as $int << shift;
                        i += 1;
                    }
                }

                #[allow(unused_comparisons)]
                if out < 0 {
                    return None;
                }

                while i < N {
                    if self.digits[i] != 0 {
                        return None;
                    }
                    i += 1;
                }

                Some(out)
            }
        )*
    };
}

use crate::buint::cast::{decode_f32, decode_f64, u32_bits, u64_bits};
//use crate::nightly::impl_const;
use crate::ExpType;
use num_integer::{Integer, Roots};
use num_traits::{
    AsPrimitive, Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl,
    CheckedShr, CheckedSub, FromPrimitive, MulAdd, MulAddAssign, Num, One, Pow, PrimInt,
    Saturating, SaturatingAdd, SaturatingMul, SaturatingSub, ToPrimitive, Unsigned, WrappingAdd,
    WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub, Zero,
};

use crate::cast::CastFrom;
use crate::int::numtraits::num_trait_impl;

macro_rules! numtraits {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::int::numtraits::impls!($BUint, $BUint, $BInt);

        macro_rules! from_float {
            ($method: ident, $float: ty, $decoder: ident, $mant_bits: ident) => {
                #[inline]
                fn $method(f: $float) -> Option<Self> {
                    if !f.is_finite() {
                        return None;
                    }
                    if f == 0.0 {
                        return Some(Self::ZERO);
                    }
                    if f.is_sign_negative() {
                        return None;
                    }
                    let (mut mant, exp) = $decoder(f);
                    if exp.is_negative() {
                        mant = mant.checked_shr((-exp) as ExpType).unwrap_or(0);
                        if $mant_bits(mant) > Self::BITS {
                            return None;
                        }
                        Some(Self::cast_from(mant))
                    } else {
                        if $mant_bits(mant) + exp as ExpType > Self::BITS {
                            return None;
                        }
                        Some(Self::cast_from(mant) << exp)
                    }
                }
            };
        }

        //impl_const! {
        impl<const N: usize> FromPrimitive for $BUint<N> {
            #[inline]
            fn from_u64(int: u64) -> Option<Self> {
                const UINT_BITS: usize = u64::BITS as usize;
                let mut out = $BUint::ZERO;
                let mut i = 0;
                while i << crate::digit::$Digit::BIT_SHIFT < UINT_BITS {
                    let d = (int >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                    if d != 0 {
                        if i < N {
                            out.digits[i] = d;
                        } else {
                            return None;
                        }
                    }
                    i += 1;
                }
                Some(out)
            }

            #[inline]
            fn from_i64(int: i64) -> Option<Self> {
                match u64::try_from(int) {
                    Ok(int) => Self::from_u64(int),
                    _ => None,
                }
            }

            #[inline]
            fn from_u128(int: u128) -> Option<Self> {
                const UINT_BITS: usize = u128::BITS as usize;
                let mut out = $BUint::ZERO;
                let mut i = 0;
                while i << crate::digit::$Digit::BIT_SHIFT < UINT_BITS {
                    let d = (int >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                    if d != 0 {
                        if i < N {
                            out.digits[i] = d;
                        } else {
                            return None;
                        }
                    }
                    i += 1;
                }
                Some(out)
            }

            #[inline]
            fn from_i128(n: i128) -> Option<Self> {
                match u128::try_from(n) {
                    Ok(n) => Self::from_u128(n),
                    _ => None,
                }
            }

            from_float!(from_f32, f32, decode_f32, u32_bits);
            from_float!(from_f64, f64, decode_f64, u64_bits);
        }
        //}

        //impl_const! {
        impl<const N: usize> Integer for $BUint<N> {
            #[inline]
            fn div_floor(&self, other: &Self) -> Self {
                *self / *other
            }

            #[inline]
            fn mod_floor(&self, other: &Self) -> Self {
                *self % *other
            }

            #[inline]
            fn gcd(&self, other: &Self) -> Self {
                // Paul E. Black, "binary GCD", in Dictionary of Algorithms and Data Structures [online], Paul E. Black, ed. 2 November 2020. (accessed 15th June 2022) Available from: https://www.nist.gov/dads/HTML/binaryGCD.html
                // https://en.wikipedia.org/wiki/Binary_GCD_algorithm#Implementation

                let (mut a, mut b) = (*self, *other);
                if a.is_zero() {
                    return b;
                }
                if b.is_zero() {
                    return a;
                }
                let mut a_tz = a.trailing_zeros();
                let mut b_tz = b.trailing_zeros();
                // Normalise `a` and `b` so that both of them has no leading zeros, so both must be odd.
                unsafe {
                    a = Self::unchecked_shr_internal(a, a_tz);
                    b = Self::unchecked_shr_internal(b, b_tz);
                }

                if b_tz > a_tz {
                    // Ensure `a_tz >= b_tz`
                    core::mem::swap(&mut a_tz, &mut b_tz);
                }
                loop {
                    if a < b {
                        // Ensure `a >= b`
                        core::mem::swap(&mut a, &mut b);
                    }
                    a -= b;
                    if a.is_zero() {
                        return unsafe { Self::unchecked_shl_internal(b, b_tz) };
                    }
                    unsafe {
                        a = Self::unchecked_shr_internal(a, a.trailing_zeros());
                    }
                }
            }

            #[inline]
            fn lcm(&self, other: &Self) -> Self {
                self.div_floor(&self.gcd(other)) * *other
            }

            #[inline]
            fn divides(&self, other: &Self) -> bool {
                self.is_multiple_of(other)
            }

            #[inline]
            fn is_multiple_of(&self, other: &Self) -> bool {
                self.mod_floor(other).is_zero()
            }

            #[inline]
            fn is_even(&self) -> bool {
                self.digits[0] & 1 == 0
            }

            #[inline]
            fn is_odd(&self) -> bool {
                self.digits[0] & 1 == 1
            }

            #[inline]
            fn div_rem(&self, rhs: &Self) -> (Self, Self) {
                Self::div_rem(*self, *rhs)
            }
        }
        //}

        //impl_const! {
        impl<const N: usize> PrimInt for $BUint<N> {
            crate::int::numtraits::prim_int_methods!();

            #[inline]
            fn signed_shl(self, n: u32) -> Self {
                self << n
            }

            #[inline]
            fn signed_shr(self, n: u32) -> Self {
                ($BInt::from_bits(self) >> n).to_bits()
            }

            #[inline]
            fn unsigned_shl(self, n: u32) -> Self {
                self << n
            }

            #[inline]
            fn unsigned_shr(self, n: u32) -> Self {
                self >> n
            }
        }
        //}

        macro_rules! check_zero_or_one {
            ($self: ident) => {
                if N == 0 {
                    return *$self;
                }
                if $self.last_digit_index() == 0 {
                    let d = $self.digits[0];
                    if d == 0 || d == 1 {
                        return *$self;
                    }
                }
            };
        }

        /*
        The `fixpoint` function and the implementation of `Roots` below are adapted from the Rust `num_bigint` library, https://docs.rs/num-bigint/latest/num_bigint/, modified under the MIT license. The changes are released under either the MIT license or the Apache License 2.0, as described in the README. See LICENSE-MIT or LICENSE-APACHE at the project root.

        The appropriate copyright notice for the `num_bigint` code is given below:
        Copyright (c) 2014 The Rust Project Developers

        The original license file and copyright notice for `num_bigint` can be found in this project's root at licenses/LICENSE-num-bigint.
        */

        impl<const N: usize> $BUint<N> {
            #[inline]
            fn fixpoint<F>(mut self, max_bits: ExpType, f: F) -> Self
            where
                F: Fn(Self) -> Self,
            {
                let mut xn = f(self);
                while self < xn {
                    self = if xn.bits() > max_bits {
                        Self::power_of_two(max_bits)
                    } else {
                        xn
                    };
                    xn = f(self);
                }
                while self > xn {
                    self = xn;
                    xn = f(self);
                }
                self
            }
        }

        impl<const N: usize> Roots for $BUint<N> {
            #[inline]
            fn sqrt(&self) -> Self {
                check_zero_or_one!(self);

                #[cfg(not(test))]
                // disable this when testing as this condition will always be true when testing against primitives, so the rest of the algorithm wouldn't be tested
                if let Some(n) = self.to_u128() {
                    return n.sqrt().into();
                }
                let bits = self.bits();
                let max_bits = bits / 2 + 1;

                let guess = Self::power_of_two(max_bits);
                guess.fixpoint(max_bits, |s| {
                    let q = self / s;
                    let t = s + q;
                    t >> 1
                })
            }

            #[inline]
            fn cbrt(&self) -> Self {
                check_zero_or_one!(self);

                #[cfg(not(test))]
                // disable this when testing as this condition will always be true when testing against primitives, so the rest of the algorithm wouldn't be tested
                if let Some(n) = self.to_u128() {
                    return n.cbrt().into();
                }
                let bits = self.bits();
                let max_bits = bits / 3 + 1;

                let guess = Self::power_of_two(max_bits);
                guess.fixpoint(max_bits, |s| {
                    let q = self / (s * s);
                    let t: Self = (s << 1) + q;
                    t.div_rem_digit(3).0
                })
            }

            #[inline]
            fn nth_root(&self, n: u32) -> Self {
                match n {
                    0 => panic!(crate::errors::err_msg!("attempt to calculate zeroth root")),
                    1 => *self,
                    2 => self.sqrt(),
                    3 => self.cbrt(),
                    _ => {
                        check_zero_or_one!(self);

                        #[cfg(not(test))]
                        // disable this when testing as this condition will always be true when testing against primitives, so the rest of the algorithm wouldn't be tested
                        if let Some(x) = self.to_u128() {
                            return x.nth_root(n).into();
                        }
                        let bits = self.bits();
                        let n = n as ExpType;
                        if bits <= n {
                            return Self::ONE;
                        }

                        let max_bits = bits / n + 1;

                        let guess = Self::power_of_two(max_bits);
                        let n_minus_1 = n - 1;

                        guess.fixpoint(max_bits, |s| {
                            let q = self / s.pow(n_minus_1);
                            let mul: Self = n_minus_1.into();
                            let t: Self = s * mul + q;
                            t.div_rem_unchecked(n.into()).0
                        })
                    }
                }
            }
        }

        //impl_const! {
        impl<const N: usize> ToPrimitive for $BUint<N> {
            to_int! {
                $Digit;
                to_u8 -> u8,
                to_u16 -> u16,
                to_u32 -> u32,
                to_u64 -> u64,
                to_u128 -> u128,
                to_usize -> usize,

                to_i8 -> i8,
                to_i16 -> i16,
                to_i32 -> i32,
                to_i64 -> i64,
                to_i128 -> i128,
                to_isize -> isize
            }

            #[inline]
            fn to_f32(&self) -> Option<f32> {
                Some(self.as_())
            }

            #[inline]
            fn to_f64(&self) -> Option<f64> {
                Some(self.as_())
            }
        }
        //}

        impl<const N: usize> Unsigned for $BUint<N> {}

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::types::utest;

                crate::int::numtraits::tests!(utest);
            }
        }
    };
}

crate::macro_impl!(numtraits);
