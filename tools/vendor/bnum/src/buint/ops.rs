use crate::digit;
use crate::nightly::impl_const;
use crate::ExpType;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

macro_rules! ops {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl_const! {
            impl<const N: usize> const Add<$Digit> for $BUint<N> {
                type Output = Self;

                #[inline]
                fn add(self, rhs: $Digit) -> Self {
                    let mut out = Self::ZERO;
                    let result = digit::$Digit::carrying_add(self.digits[0], rhs, false);
                    out.digits[0] = result.0;
                    let mut carry = result.1;
                    let mut i = 1;
                    while i < N {
                        let result = digit::$Digit::carrying_add(self.digits[0], 0, carry);
                        out.digits[i] = result.0;
                        carry = result.1;
                        i += 1;
                    }
                    out
                }
            }
        }

        impl_const! {
            impl<const N: usize> const BitAnd for $BUint<N> {
                type Output = Self;

                #[inline]
                fn bitand(self, rhs: Self) -> Self {
                    Self::bitand(self, rhs)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const BitOr for $BUint<N> {
                type Output = Self;

                #[inline]
                fn bitor(self, rhs: Self) -> Self {
                    Self::bitor(self, rhs)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const BitXor for $BUint<N> {
                type Output = Self;

                #[inline]
                fn bitxor(self, rhs: Self) -> Self {
                    Self::bitxor(self, rhs)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const Div for $BUint<N> {
                type Output = Self;

                #[inline]
                fn div(self, rhs: Self) -> Self {
                    Self::div(self, rhs)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const Div<$Digit> for $BUint<N> {
                type Output = Self;

                #[inline]
                fn div(self, rhs: $Digit) -> Self {
                    self.div_rem_digit(rhs).0
                }
            }

            impl_const! {
                impl<const N: usize> const Not for $BUint<N> {
                    type Output = Self;

                    #[inline]
                    fn not(self) -> Self {
                        Self::not(self)
                    }
                }
            }

            impl_const! {
                impl<const N: usize> const Rem for $BUint<N> {
                    type Output = Self;

                    #[inline]
                    fn rem(self, rhs: Self) -> Self {
                        Self::rem(self, rhs)
                    }
                }
            }

            impl_const! {
                impl<const N: usize> const Rem<$Digit> for $BUint<N> {
                    type Output = $Digit;

                    #[inline]
                    fn rem(self, rhs: $Digit) -> $Digit {
                        self.div_rem_digit(rhs).1
                    }
                }
            }
        }

        crate::int::ops::impls!($BUint, $BUint, $BInt);

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use super::*;
                use crate::test::{test_bignum, types::utest};
                use crate::test::types::big_types::$Digit::*;

                crate::int::ops::tests!(utest);
            }
        }
    };
}

crate::macro_impl!(ops);
