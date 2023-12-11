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

macro_rules! bint_cast_from_float {
    ($f: ty, $BUint: ident <$N: ident>) => {
        #[inline]
        fn cast_from(from: $f) -> Self {
            if from.is_sign_negative() {
                let u = $BUint::<$N>::cast_from(-from);
                if u >= Self::MIN.to_bits() {
                    Self::MIN
                } else {
                    -Self::from_bits(u)
                }
            } else {
                let u = $BUint::<$N>::cast_from(from);
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

pub(crate) use bint_cast_from_float;

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

        impl<const N: usize> CastFrom<f32> for $BInt<N> {
            crate::bint::cast::bint_cast_from_float!(f32, $BUint<N>);
        }

        impl<const N: usize> CastFrom<f64> for $BInt<N> {
            crate::bint::cast::bint_cast_from_float!(f64, $BUint<N>);
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

macro_rules! bint_as_different_digit_bigint {
    ($BUint: ident, $BInt: ident, $Digit: ident; $(($OtherBInt: ident, $OtherDigit: ident)), *) => {
        $(
            crate::nightly::const_impl! {
                impl<const N: usize, const M: usize> const crate::cast::CastFrom<$OtherBInt<M>> for $BUint<N> {
                    #[must_use = doc::must_use_op!()]
                    #[inline]
                    fn cast_from(from: $OtherBInt<M>) -> Self {
                        if !from.is_negative() || M * $OtherDigit::BITS as usize >= N * $Digit::BITS as usize { // $OtherBInt::BITS <= $Int::BITS
                            Self::cast_from(from.to_bits())
                        } else {
                            let mut out = Self::MAX;
                            if $Digit::BITS < $OtherDigit::BITS {
                                const DIVIDE_COUNT: usize = ($OtherDigit::BITS / $Digit::BITS) as usize;
                                let stop_index: usize = if <$OtherBInt<M>>::BITS > <$BUint<N>>::BITS {
                                    N
                                } else {
                                    M * DIVIDE_COUNT
                                };
                                let mut i = 0;
                                while i < stop_index {
                                    let wider_digit = from.bits.digits[i / DIVIDE_COUNT];
                                    let mini_shift = i % DIVIDE_COUNT;
                                    let digit = (wider_digit >> (mini_shift << digit::$Digit::BIT_SHIFT)) as $Digit;
                                    out.digits[i] = digit;
                                    i += 1;
                                }
                            } else {
                                const DIVIDE_COUNT: usize = ($Digit::BITS / $OtherDigit::BITS) as usize;
                                let stop_index: usize = if <$OtherBInt<M>>::BITS > <$BUint<N>>::BITS {
                                    N * DIVIDE_COUNT
                                } else {
                                    M
                                };
                                let mut current_digit: $Digit = $Digit::MAX;
                                let mut i = 0;
                                while i < stop_index {
                                    let mini_shift = i % DIVIDE_COUNT;
                                    current_digit &= !((!from.bits.digits[i] as $Digit) << (mini_shift << digit::$OtherDigit::BIT_SHIFT));
                                    if mini_shift == DIVIDE_COUNT - 1 || i == stop_index - 1 {
                                        out.digits[i / DIVIDE_COUNT] = current_digit;
                                        current_digit = $Digit::MAX;
                                    }
                                    i += 1;
                                }
                            }
                            out
                        }
                    }
                }
            }

            crate::nightly::const_impl! {
                impl<const N: usize, const M: usize> const crate::cast::CastFrom<$OtherBInt<M>> for $BInt<N> {
                    #[must_use = doc::must_use_op!()]
                    #[inline]
                    fn cast_from(from: $OtherBInt<M>) -> Self {
                        Self::from_bits($BUint::<N>::cast_from(from))
                    }
                }
            }
        )*
    };
}

pub(crate) use bint_as_different_digit_bigint;