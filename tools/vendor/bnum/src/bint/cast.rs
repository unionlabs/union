macro_rules! bint_as {
    ($BInt: ident, $Digit: ident; $($int: ty), *) => {
        $(
            impl_const! {
                impl<const N: usize> const CastFrom<$BInt<N>> for $int {
                    #[inline]
                    fn cast_from(from: $BInt<N>) -> Self {
                        if from.is_negative() {
                            let digits = from.bits.digits;
                            let mut out = !0;
                            let mut i = 0;
                            while i << digit::$Digit::BIT_SHIFT < <$int>::BITS as usize && i < N {
                                out &= !((!digits[i]) as $int << (i << digit::$Digit::BIT_SHIFT));
                                i += 1;
                            }
                            out
                        } else {
                            <$int>::cast_from(from.bits)
                        }
                    }
                }
            }
        )*
    };
}

macro_rules! as_bint {
    ($BInt: ident, $BUint: ident; $($ty: ty), *) => {
        $(impl_const! {
            impl<const N: usize> const CastFrom<$ty> for $BInt<N> {
                #[inline]
                fn cast_from(from: $ty) -> Self {
                    Self::from_bits($BUint::cast_from(from))
                }
            }
        })*
    }
}

use crate::cast::CastFrom;
use crate::digit;
use crate::nightly::impl_const;

macro_rules! cast {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        bint_as!($BInt, $Digit; u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

        impl<const N: usize> CastFrom<$BInt<N>> for f32 {
            #[inline]
            fn cast_from(from: $BInt<N>) -> Self {
                let f = f32::cast_from(from.unsigned_abs());
                if from.is_negative() {
                    -f
                } else {
                    f
                }
            }
        }

        impl<const N: usize> CastFrom<$BInt<N>> for f64 {
            #[inline]
            fn cast_from(from: $BInt<N>) -> Self {
                let f = f64::cast_from(from.unsigned_abs());
                if from.is_negative() {
                    -f
                } else {
                    f
                }
            }
        }

        as_bint!($BInt, $BUint; u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, bool, char);

        impl_const! {
            impl<const N: usize, const M: usize> const CastFrom<$BUint<M>> for $BInt<N> {
                #[inline]
                fn cast_from(from: $BUint<M>) -> Self {
                    Self::from_bits($BUint::cast_from(from))
                }
            }
        }

        impl_const! {
            impl<const N: usize, const M: usize> const CastFrom<$BInt<M>> for $BInt<N> {
                #[inline]
                fn cast_from(from: $BInt<M>) -> Self {
                    Self::from_bits($BUint::cast_from(from))
                }
            }
        }

        macro_rules! cast_from_float {
            ($f: ty) => {
                #[inline]
                fn cast_from(from: $f) -> Self {
                    if from.is_sign_negative() {
                        let u = $BUint::<N>::cast_from(-from);
                        if u >= Self::MIN.to_bits() {
                            Self::MIN
                        } else {
                            -Self::from_bits(u)
                        }
                    } else {
                        let u = $BUint::<N>::cast_from(from);
                        let i = Self::from_bits(u);
                        if i.is_negative() {
                            Self::MAX
                        } else {
                            i
                        }
                    }
                }
            };
        }

        impl<const N: usize> CastFrom<f32> for $BInt<N> {
            cast_from_float!(f32);
        }

        impl<const N: usize> CastFrom<f64> for $BInt<N> {
            cast_from_float!(f64);
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                crate::int::cast::tests!(itest);
            }
        }
    };
}

crate::macro_impl!(cast);
