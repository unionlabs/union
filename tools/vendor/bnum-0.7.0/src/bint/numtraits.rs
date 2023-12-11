macro_rules! from_int {
    ($BUint: ident, $Digit: ident; $int: ty, $name: ident) => {
        #[inline]
        fn $name(n: $int) -> Option<Self> {
            const INT_BITS: usize = <$int>::BITS as usize;
            let initial_digit = if n.is_negative() {
                $Digit::MAX
            } else {
                $Digit::MIN
            };
            let mut out = Self::from_bits($BUint::from_digits([initial_digit; N]));
            let mut i = 0;
            while i << crate::digit::$Digit::BIT_SHIFT < INT_BITS {
                let d = (n >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                if d != initial_digit {
                    if i < N {
                        out.bits.digits[i] = d;
                    } else {
                        return None;
                    }
                }
                i += 1;
            }
            if n.is_negative() != out.is_negative() {
                return None;
            }
            Some(out)
        }
    };
}

macro_rules! from_uint {
    ($Digit: ident; $uint: ty, $name: ident) => {
        #[inline]
        fn $name(n: $uint) -> Option<Self> {
            const UINT_BITS: usize = <$uint>::BITS as usize;
            let mut out = Self::ZERO;
            let mut i = 0;
            while i << crate::digit::$Digit::BIT_SHIFT < UINT_BITS {
                let d = (n >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                if d != 0 {
                    if i < N {
                        out.bits.digits[i] = d;
                    } else {
                        return None;
                    }
                }
                i += 1;
            }
            if Signed::is_negative(&out) {
                None
            } else {
                Some(out)
            }
        }
    };
}

macro_rules! from_float {
    ($BUint: ident; $method: ident, $float: ty) => {
        #[inline]
        fn $method(f: $float) -> Option<Self> {
            if f.is_sign_negative() {
                let i = Self::from_bits($BUint::$method(-f)?);
                if i == Self::MIN {
                    Some(Self::MIN)
                } else if i.is_negative() {
                    None
                } else {
                    Some(-i)
                }
            } else {
                let i = Self::from_bits($BUint::$method(f)?);
                if i.is_negative() {
                    None
                } else {
                    Some(i)
                }
            }
        }
    };
}

macro_rules! to_uint {
    { $($name: ident -> $uint: ty), * } => {
        $(
            #[inline]
            fn $name(&self) -> Option<$uint> {
                if self.is_negative() {
                    None
                } else {
                    self.bits.$name()
                }
            }
        )*
    };
}

macro_rules! to_int {
    { $Digit: ident; $($name: ident -> $int: ty), * }  => {
        $(
            fn $name(&self) -> Option<$int> {
                let neg = self.is_negative();
                let (mut out, padding) = if neg {
                    (-1, $Digit::MAX)
                } else {
                    (0, $Digit::MIN)
                };
                let mut i = 0;
                if $Digit::BITS > <$int>::BITS {
                    let small = self.bits.digits[i] as $int;
                    let trunc = small as $Digit;
                    if self.bits.digits[i] != trunc {
                        return None;
                    }
                    out = small;
                    i = 1;
                } else {
                    if neg {
                        loop {
                            let shift = i << digit::$Digit::BIT_SHIFT;
                            if i >= N || shift >= <$int>::BITS as usize {
                                break;
                            }
                            out &= !((!self.bits.digits[i]) as $int << shift);
                            i += 1;
                        }
                    } else {
                        loop {
                            let shift = i << digit::$Digit::BIT_SHIFT;
                            if i >= N || shift >= <$int>::BITS as usize {
                                break;
                            }
                            out |= self.bits.digits[i] as $int << shift;
                            i += 1;
                        }
                    }
                }

                while i < N {
                    if self.bits.digits[i] != padding {
                        return None;
                    }
                    i += 1;
                }

                if out.is_negative() != neg {
                    return None;
                }

                Some(out)
            }
        )*
    };
}

use crate::digit;
use crate::errors;
use crate::ExpType;
use num_integer::{Integer, Roots};
use num_traits::{
    AsPrimitive, Bounded, CheckedAdd, CheckedDiv, CheckedMul, CheckedNeg, CheckedRem, CheckedShl,
    CheckedShr, CheckedSub, FromPrimitive, MulAdd, MulAddAssign, Num, One, Pow, PrimInt,
    Saturating, SaturatingAdd, SaturatingMul, SaturatingSub, Signed, ToPrimitive, WrappingAdd,
    WrappingMul, WrappingNeg, WrappingShl, WrappingShr, WrappingSub, Zero,
};

use crate::cast::CastFrom;
use crate::int::numtraits::num_trait_impl;

macro_rules! numtraits {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::int::numtraits::impls!($BInt, $BUint, $BInt);

        impl<const N: usize> FromPrimitive for $BInt<N> {
            from_uint!($Digit; u8, from_u8);
            from_uint!($Digit; u16, from_u16);
            from_uint!($Digit; u32, from_u32);
            from_uint!($Digit; u64, from_u64);
            from_uint!($Digit; u128, from_u128);
            from_uint!($Digit; usize, from_usize);
            from_int!($BUint, $Digit; i8, from_i8);
            from_int!($BUint, $Digit; i16, from_i16);
            from_int!($BUint, $Digit; i32, from_i32);
            from_int!($BUint, $Digit; i64, from_i64);
            from_int!($BUint, $Digit; i128, from_i128);
            from_int!($BUint, $Digit; isize, from_isize);

            from_float!($BUint; from_f32, f32);
            from_float!($BUint; from_f64, f64);
        }

        //crate::nightly::impl_const! {
            impl<const N: usize> Integer for $BInt<N> {
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
                    let gcd = self.unsigned_abs().gcd(&other.unsigned_abs());
                    let out = Self::from_bits(gcd);
                    if out == Self::MIN {
                        panic!("{}", errors::err_msg!("gcd of is too large"));
                    }
                    out.abs()
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
                    self.bits.is_even()
                }

                #[inline]
                fn is_odd(&self) -> bool {
                    self.bits.is_odd()
                }

                #[inline]
                fn div_rem(&self, other: &Self) -> (Self, Self) {
                    (self.div_floor(other), self.mod_floor(other))
                }
            }
        //}

        //crate::nightly::impl_const! {
            impl<const N: usize> PrimInt for $BInt<N> {
                crate::int::numtraits::prim_int_methods!();

                #[inline]
                fn signed_shl(self, n: u32) -> Self {
                    self << n
                }

                #[inline]
                fn signed_shr(self, n: u32) -> Self {
                    self >> n
                }

                #[inline]
                fn unsigned_shl(self, n: u32) -> Self {
                    self << n
                }

                #[inline]
                fn unsigned_shr(self, n: u32) -> Self {
                    Self::from_bits(self.to_bits() >> n)
                }
            }
        //}

        impl<const N: usize> Roots for $BInt<N> {
            #[inline]
            fn sqrt(&self) -> Self {
                if self.is_negative() {
                    panic!(crate::errors::err_msg!("imaginary square root"))
                } else {
                    Self::from_bits(self.bits.sqrt())
                }
            }

            #[inline]
            fn cbrt(&self) -> Self {
                if self.is_negative() {
                    let out = Self::from_bits(self.unsigned_abs().cbrt());
                    -out
                } else {
                    Self::from_bits(self.bits.cbrt())
                }
            }

            #[inline]
            fn nth_root(&self, n: u32) -> Self {
                if self.is_negative() {
                    if n == 0 {
                        panic!(crate::errors::err_msg!("attempt to calculate zeroth root"));
                    }
                    if n == 1 {
                        return *self;
                    }
                    if n.is_even() {
                        panic!("{} imaginary root degree of {}", errors::err_prefix!(), n)
                    } else {
                        let out = Self::from_bits(self.unsigned_abs().nth_root(n));
                        out.wrapping_neg()
                    }
                } else {
                    Self::from_bits(self.bits.nth_root(n))
                }
            }
        }

        //crate::nightly::impl_const! {
        impl<const N: usize> ToPrimitive for $BInt<N> {
            to_uint! {
                to_u8 -> u8,
                to_u16 -> u16,
                to_u32 -> u32,
                to_u64 -> u64,
                to_u128 -> u128,
                to_usize -> usize
            }

            to_int! {
                $Digit;
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

        //crate::nightly::impl_const! {
            impl<const N: usize> Signed for $BInt<N> {
                #[inline]
                fn abs(&self) -> Self {
                    Self::abs(*self)
                }

                #[inline]
                fn abs_sub(&self, other: &Self) -> Self {
                    if *self <= *other {
                        Self::ZERO
                    } else {
                        *self - *other
                    }
                }

                #[inline]
                fn signum(&self) -> Self {
                    Self::signum(*self)
                }

                #[inline]
                fn is_positive(&self) -> bool {
                    Self::is_positive(*self)
                }

                #[inline]
                fn is_negative(&self) -> bool {
                    self.signed_digit().is_negative()
                }
            }
        //}

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::types::itest;

                crate::int::numtraits::tests!(itest);
            }
        }
    };
}

crate::macro_impl!(numtraits);
