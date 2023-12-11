use crate::digit;
use crate::errors::div_zero;
use crate::{doc, ExpType};

macro_rules! overflowing {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        #[doc = doc::overflowing::impl_desc!()]
        impl<const N: usize> $BInt<N> {
            #[doc = doc::overflowing::overflowing_add!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                let mut out = Self::ZERO;
                let mut carry = false;

                let self_digits = self.bits.digits;
                let rhs_digits = rhs.bits.digits;

                let mut i = 0;
                while i < Self::N_MINUS_1 {
                    let (sum, c) =
                        digit::$Digit::carrying_add(self_digits[i], rhs_digits[i], carry);
                    out.bits.digits[i] = sum;
                    carry = c;
                    i += 1;
                }
                let (sum, carry) = digit::$Digit::carrying_add_signed(
                    self_digits[Self::N_MINUS_1] as digit::$Digit::SignedDigit,
                    rhs_digits[Self::N_MINUS_1] as digit::$Digit::SignedDigit,
                    carry,
                );
                out.bits.digits[Self::N_MINUS_1] = sum as $Digit;

                (out, carry)
            }

            #[doc = doc::overflowing::overflowing_add_unsigned!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_add_unsigned(self, rhs: $BUint<N>) -> (Self, bool) {
                let rhs = Self::from_bits(rhs);
                let (sum, overflow) = self.overflowing_add(rhs);
                (sum, rhs.is_negative() != overflow)
            }

            #[doc = doc::overflowing::overflowing_sub!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_sub(self, rhs: Self) -> (Self, bool) {
                let mut out = Self::ZERO;
                let mut borrow = false;

                let self_digits = self.bits.digits;
                let rhs_digits = rhs.bits.digits;

                let mut i = 0;
                while i < Self::N_MINUS_1 {
                    let (sub, b) =
                        digit::$Digit::borrowing_sub(self_digits[i], rhs_digits[i], borrow);
                    out.bits.digits[i] = sub;
                    borrow = b;
                    i += 1;
                }
                let (sub, borrow) = digit::$Digit::borrowing_sub_signed(
                    self_digits[Self::N_MINUS_1] as digit::$Digit::SignedDigit,
                    rhs_digits[Self::N_MINUS_1] as digit::$Digit::SignedDigit,
                    borrow,
                );
                out.bits.digits[Self::N_MINUS_1] = sub as $Digit;

                (out, borrow)
            }

            #[doc = doc::overflowing::overflowing_sub_unsigned!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_sub_unsigned(self, rhs: $BUint<N>) -> (Self, bool) {
                let rhs = Self::from_bits(rhs);
                let (sum, overflow) = self.overflowing_sub(rhs);
                (sum, rhs.is_negative() != overflow)
            }

            const BITS_MINUS_1: ExpType = (Self::BITS - 1) as ExpType;

            #[doc = doc::overflowing::overflowing_mul!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_mul(self, rhs: Self) -> (Self, bool) {
                let (uint, overflow) = self.unsigned_abs().overflowing_mul(rhs.unsigned_abs());
                let out = Self::from_bits(uint);
                if self.is_negative() == rhs.is_negative() {
                    (out, overflow || out.is_negative())
                } else {
                    match out.checked_neg() {
                        Some(n) => (n, overflow || out.is_negative()),
                        None => (out, overflow),
                    }
                }
            }

            crate::nightly::const_fns! {
                #[inline]
                pub(crate) const fn div_rem_unchecked(self, rhs: Self) -> (Self, Self) {
                    if self.eq(&Self::MIN) && rhs.is_one() {
                        return (self, Self::ZERO);
                    }
                    //println!("unchecked - self: {}, rhs: {}", self, rhs);
                    let (div, rem) = self.unsigned_abs().div_rem_unchecked(rhs.unsigned_abs());
                    let (div, rem) = (Self::from_bits(div), Self::from_bits(rem));

                    match (self.is_negative(), rhs.is_negative()) {
                        (false, false) => (div, rem),
                        (false, true) => (div.neg(), rem),
                        (true, false) => (div.neg(), rem.neg()),
                        (true, true) => (div, rem.neg()),
                    }
                }

                #[doc = doc::overflowing::overflowing_div!(I)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn overflowing_div(self, rhs: Self) -> (Self, bool) {
                    if rhs.is_zero() {
                        div_zero!()
                    }
                    if self.eq(&Self::MIN) {
                        if rhs.eq(&Self::NEG_ONE) {
                            return (self, true);
                        } else if rhs.is_one() {
                            return (self, false);
                        }
                    }
                    (self.div_rem_unchecked(rhs).0, false)
                }

                #[doc = doc::overflowing::overflowing_div_euclid!(I)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn overflowing_div_euclid(self, rhs: Self) -> (Self, bool) {
                    if rhs.is_zero() {
                        div_zero!()
                    }
                    if self.eq(&Self::MIN) {
                        if rhs.eq(&Self::NEG_ONE) {
                            return (self, true);
                        } else if rhs.is_one() {
                            return (self, false);
                        }
                    }
                    let (div, rem) = self.div_rem_unchecked(rhs);
                    if self.is_negative() {
                        let r_neg = rhs.is_negative();
                        if !rem.is_zero() {
                            if r_neg {
                                return (div.add(Self::ONE), false)
                            } else {
                                return (div.sub(Self::ONE), false)
                            };
                        }
                    }
                    (div, false)
                }

                #[doc = doc::overflowing::overflowing_rem!(I)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn overflowing_rem(self, rhs: Self) -> (Self, bool) {
                    if rhs.is_zero() {
                        div_zero!()
                    }
                    if self.eq(&Self::MIN) && rhs.eq(&Self::NEG_ONE) {
                        (Self::ZERO, true)
                    } else {
                        (self.div_rem_unchecked(rhs).1, false)
                    }
                }

                #[doc = doc::overflowing::overflowing_rem_euclid!(I)]
                #[must_use = doc::must_use_op!()]
                #[inline]
                pub const fn overflowing_rem_euclid(self, rhs: Self) -> (Self, bool) {
                    if rhs.is_zero() {
                        div_zero!()
                    }
                    if self.eq(&Self::MIN) && rhs.eq(&Self::NEG_ONE) {
                        (Self::ZERO, true)
                    } else {
                        let mut rem = self.div_rem_unchecked(rhs).1;
                        if rem.is_negative() {
                            if rhs.is_negative() {
                                rem = rem.wrapping_sub(rhs);
                            } else {
                                rem = rem.wrapping_add(rhs);
                            }
                        }
                        (rem, false)
                    }
                }
            }

            #[doc = doc::overflowing::overflowing_neg!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_neg(mut self) -> (Self, bool) {
                let mut i = 0;
                while i < N - 1 {
                    let (s, o) = (!self.bits.digits[i]).overflowing_add(1); // TODO: use overflowing add on signed integer digit instead
                    self.bits.digits[i] = s;
                    if !o {
                        i += 1;
                        while i < N {
                            self.bits.digits[i] = !self.bits.digits[i];
                            i += 1;
                        }
                        return (self, false);
                    }
                    i += 1;
                }
                let (s, o) =
                    (!self.bits.digits[i] as digit::$Digit::SignedDigit).overflowing_add(1);
                self.bits.digits[i] = s as $Digit;
                (self, o)
            }

            #[doc = doc::overflowing::overflowing_shl!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_shl(self, rhs: ExpType) -> (Self, bool) {
                let (uint, overflow) = self.bits.overflowing_shl(rhs);
                (Self::from_bits(uint), overflow)
            }

            #[doc = doc::overflowing::overflowing_shr!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_shr(self, rhs: ExpType) -> (Self, bool) {
                let bits = self.to_bits();
                let (overflow, shift) = if rhs >= Self::BITS {
                    (true, rhs & Self::BITS_MINUS_1)
                } else {
                    (false, rhs)
                };
                let u = unsafe {
                    if self.is_negative() {
                        $BUint::unchecked_shr_pad_internal::<{$Digit::MAX}>(bits, shift)
                    } else {
                        $BUint::unchecked_shr_pad_internal::<{$Digit::MIN}>(bits, shift)
                    }
                };
                (Self::from_bits(u), overflow)
            }

            #[doc = doc::overflowing::overflowing_abs!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_abs(self) -> (Self, bool) {
                if self.is_negative() {
                    self.overflowing_neg()
                } else {
                    (self, false)
                }
            }

            #[doc = doc::overflowing::overflowing_pow!(I)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn overflowing_pow(self, pow: ExpType) -> (Self, bool) {
                let (u, mut overflow) = self.unsigned_abs().overflowing_pow(pow);
                let out_neg = self.is_negative() && pow & 1 == 1;
                let mut out = Self::from_bits(u);
                if out_neg {
                    out = out.wrapping_neg();
                    overflow = overflow || !out.is_negative();
                } else {
                    overflow = overflow || out.is_negative();
                }
                (out, overflow)
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::{test_bignum, types::itest};
                use crate::test::types::big_types::$Digit::*;

                test_bignum! {
                    function: <itest>::overflowing_add(a: itest, b: itest)
                }
                test_bignum! {
                    function: <itest>::overflowing_sub(a: itest, b: itest)
                }
                test_bignum! {
                    function: <itest>::overflowing_mul(a: itest, b: itest)
                }
                test_bignum! {
                    function: <itest>::overflowing_div(a: itest, b: itest),
                    skip: b == 0,
                    cases: [
                        (itest::MIN, -1i8)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_div_euclid(a: itest, b: itest),
                    skip: b == 0,
                    cases: [
                        (itest::MIN, -1i8)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_rem(a: itest, b: itest),
                    skip: b == 0,
                    cases: [
                        (itest::MIN, -1i8)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_rem_euclid(a: itest, b: itest),
                    skip: b == 0,
                    cases: [
                        (itest::MIN, -1i8)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_neg(a: itest),
                    cases: [
                        (0i8),
                        (itest::MIN)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_shl(a: itest, b: u16)
                }
                test_bignum! {
                    function: <itest>::overflowing_shr(a: itest, b: u16)
                }
                test_bignum! {
                    function: <itest>::overflowing_abs(a: itest),
                    cases: [
                        (0i8),
                        (itest::MIN)
                    ]
                }
                test_bignum! {
                    function: <itest>::overflowing_pow(a: itest, b: u16)
                }
            }
        }
    };
}

crate::macro_impl!(overflowing);
