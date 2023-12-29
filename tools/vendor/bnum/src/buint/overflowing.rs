use crate::digit;
use crate::doc;
use crate::ExpType;

macro_rules! overflowing {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        #[doc = doc::overflowing::impl_desc!()]
        impl<const N: usize> $BUint<N> {
            #[doc = doc::overflowing::overflowing_add!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                let mut out = Self::ZERO;
                let mut carry = false;
                let mut i = 0;
                while i < N {
                    let result = digit::$Digit::carrying_add(self.digits[i], rhs.digits[i], carry);
                    out.digits[i] = result.0;
                    carry = result.1;
                    i += 1;
                }
                (out, carry)
            }

            #[doc = doc::overflowing::overflowing_add_signed!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_add_signed(self, rhs: $BInt<N>) -> (Self, bool) {
                let (sum, overflow) = self.overflowing_add(rhs.to_bits());
                (sum, rhs.is_negative() != overflow)
            }

            #[doc = doc::overflowing::overflowing_sub!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                let mut out = Self::ZERO;
                let mut borrow = false;
                let mut i = 0;
                while i < N {
                    let result =
                        digit::$Digit::borrowing_sub(self.digits[i], rhs.digits[i], borrow);
                    out.digits[i] = result.0;
                    borrow = result.1;
                    i += 1;
                }
                (out, borrow)
            }

            #[inline]
            const fn long_mul(self, rhs: Self) -> (Self, bool) {
                let mut overflow = false;
                let mut out = Self::ZERO;
                let mut carry: $Digit;

                let mut i = 0;
                while i < N {
                    carry = 0;
                    let mut j = 0;
                    while j < N {
                        let index = i + j;
                        if index < N {
                            let (prod, c) = digit::$Digit::carrying_mul(
                                self.digits[i],
                                rhs.digits[j],
                                carry,
                                out.digits[index],
                            );
                            out.digits[index] = prod;
                            carry = c;
                        } else if self.digits[i] != 0 && rhs.digits[j] != 0 {
                            overflow = true;
                            break;
                        }
                        j += 1;
                    }
                    if carry != 0 {
                        overflow = true;
                    }
                    i += 1;
                }
                (out, overflow)
            }

            #[doc = doc::overflowing::overflowing_mul!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
                // TODO: implement a faster multiplication algorithm for large values of `N`
                self.long_mul(rhs)
            }

            #[doc = doc::overflowing::overflowing_div!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
                (self.wrapping_div(rhs), false)
            }

            #[doc = doc::overflowing::overflowing_div_euclid!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) {
                self.overflowing_div(rhs)
            }

            #[doc = doc::overflowing::overflowing_rem!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
                (self.wrapping_rem(rhs), false)
            }

            #[doc = doc::overflowing::overflowing_rem_euclid!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_rem_euclid(self, rhs: Self) -> (Self, bool) {
                self.overflowing_rem(rhs)
            }

            #[doc = doc::overflowing::overflowing_neg!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_neg(self) -> (Self, bool) {
                let (a, b) = (self.not()).overflowing_add(Self::ONE);
                (a, !b)
            }

            #[doc = doc::overflowing::overflowing_shl!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_shl(self, rhs: ExpType) -> (Self, bool) {
                unsafe {
                    if rhs >= Self::BITS {
                        (
                            Self::unchecked_shl_internal(self, rhs & (Self::BITS - 1)),
                            true,
                        )
                    } else {
                        (Self::unchecked_shl_internal(self, rhs), false)
                    }
                }
            }

            #[doc = doc::overflowing::overflowing_shr!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_shr(self, rhs: ExpType) -> (Self, bool) {
                unsafe {
                    if rhs >= Self::BITS {
                        (
                            Self::unchecked_shr_internal(self, rhs & (Self::BITS - 1)),
                            true,
                        )
                    } else {
                        (Self::unchecked_shr_internal(self, rhs), false)
                    }
                }
            }

            #[doc = doc::overflowing::overflowing_pow!(U)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_pow(mut self, mut pow: ExpType) -> (Self, bool) {
                if pow == 0 {
                    return (Self::ONE, false);
                }
                let mut overflow = false;
                let mut y = Self::ONE;
                while pow > 1 {
                    if pow & 1 == 1 {
                        let (prod, o) = y.overflowing_mul(self);
                        overflow |= o;
                        y = prod;
                    }
                    let (prod, o) = self.overflowing_mul(self);
                    overflow |= o;
                    self = prod;
                    pow >>= 1;
                }
                let (prod, o) = self.overflowing_mul(y);
                (prod, o || overflow)
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::{test_bignum, types::utest};
                use crate::test::types::big_types::$Digit::*;

                test_bignum! {
                    function: <utest>::overflowing_add(a: utest, b: utest)
                }
                test_bignum! {
                    function: <utest>::overflowing_sub(a: utest, b: utest)
                }
                test_bignum! {
                    function: <utest>::overflowing_mul(a: utest, b: utest)
                }
                test_bignum! {
                    function: <utest>::overflowing_div(a: utest, b: utest),
                    skip: b == 0
                }
                test_bignum! {
                    function: <utest>::overflowing_div_euclid(a: utest, b: utest),
                    skip: b == 0
                }
                test_bignum! {
                    function: <utest>::overflowing_rem(a: utest, b: utest),
                    skip: b == 0
                }
                test_bignum! {
                    function: <utest>::overflowing_rem_euclid(a: utest, b: utest),
                    skip: b == 0
                }
                test_bignum! {
                    function: <utest>::overflowing_neg(a: utest)
                }
                test_bignum! {
                    function: <utest>::overflowing_shl(a: utest, b: u16)
                }
                test_bignum! {
                    function: <utest>::overflowing_shr(a: utest, b: u16)
                }
                test_bignum! {
                    function: <utest>::overflowing_pow(a: utest, b: u16)
                }
            }
        }
    };
}

crate::macro_impl!(overflowing);
