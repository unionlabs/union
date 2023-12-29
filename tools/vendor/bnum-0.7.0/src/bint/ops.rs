use crate::ExpType;
use core::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, ShrAssign, Sub, SubAssign,
};

macro_rules! ops {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::nightly::impl_const! {
            impl<const N: usize> const Neg for $BInt<N> {
                type Output = Self;

                #[inline]
                fn neg(self) -> Self {
                    Self::neg(self)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const Neg for &$BInt<N> {
                type Output = $BInt<N>;

                #[inline]
                fn neg(self) -> $BInt<N> {
                    $BInt::neg(*self)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const BitAnd for $BInt<N> {
                type Output = Self;

                #[inline]
                fn bitand(self, rhs: Self) -> Self {
                    Self::bitand(self, rhs)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const BitOr for $BInt<N> {
                type Output = Self;

                #[inline]
                fn bitor(self, rhs: Self) -> Self {
                    Self::bitor(self, rhs)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const BitXor for $BInt<N> {
                type Output = Self;

                #[inline]
                fn bitxor(self, rhs: Self) -> Self {
                    Self::bitxor(self, rhs)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const Div for $BInt<N> {
                type Output = Self;

                #[inline]
                fn div(self, rhs: Self) -> Self {
                    Self::div(self, rhs)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const Not for $BInt<N> {
                type Output = Self;

                fn not(self) -> Self {
                    Self::not(self)
                }
            }
        }

        crate::nightly::impl_const! {
            impl<const N: usize> const Rem for $BInt<N> {
                type Output = Self;

                #[inline]
                fn rem(self, rhs: Self) -> Self {
                    Self::rem(self, rhs)
                }
            }
        }

        crate::int::ops::impls!($BInt, $BUint, $BInt);

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use super::*;
                use crate::test::{debug_skip, test_bignum, types::itest};
                use crate::test::types::big_types::$Digit::*;

                crate::int::ops::tests!(itest);

                test_bignum! {
                    function: <itest>::neg(a: itest),
                    skip: debug_skip!(a == itest::MIN)
                }
            }
        }
    };
}

crate::macro_impl!(ops);
