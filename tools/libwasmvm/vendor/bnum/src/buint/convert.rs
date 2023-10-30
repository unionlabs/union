macro_rules! from_uint {
    ($BUint: ident, $Digit: ident; $($uint: tt),*) => {
        $(impl_const! {
            impl<const N: usize> const From<$uint> for $BUint<N> {
                #[inline]
                fn from(int: $uint) -> Self {
                    const UINT_BITS: usize = $uint::BITS as usize;
                    let mut out = Self::ZERO;
                    let mut i = 0;
                    while i << crate::digit::$Digit::BIT_SHIFT < UINT_BITS {
                        let d = (int >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                        if d != 0 {
                            out.digits[i] = d;
                        }
                        i += 1;
                    }
                    out
                }
            }
        })*
    }
}

macro_rules! try_from_iint {
    ($BUint: ident; $($int: tt -> $uint: tt),*) => {
        $(impl_const! {
            impl<const N: usize> const TryFrom<$int> for $BUint<N> {
                type Error = TryFromIntError;

                #[inline]
                fn try_from(int: $int) -> Result<Self, Self::Error> {
                    if int.is_negative() {
                        return Err(TryFromIntError(()));
                    }
                    let bits = int as $uint;
                    Ok(Self::from(bits))
                }
            }
        })*
    }
}

macro_rules! try_from_buint {
    ($BUint: ident, $Digit: ident; $($int: ty), *) => {
        $(crate::nightly::impl_const! {
            impl<const N: usize> const TryFrom<$BUint<N>> for $int {
                type Error = TryFromIntError;

                #[inline]
                fn try_from(u: $BUint<N>) -> Result<$int, Self::Error> {
                    let mut out = 0;
                    let mut i = 0;
                    if $Digit::BITS > <$int>::BITS {
                        let small = u.digits[i] as $int;
                        let trunc = small as $Digit;
                        if u.digits[i] != trunc {
                            return Err(TryFromIntError(()));
                        }
                        out = small;
                        i = 1;
                    } else {
                        loop {
                            let shift = i << crate::digit::$Digit::BIT_SHIFT;
                            if i >= N || shift >= <$int>::BITS as usize {
                                break;
                            }
                            out |= u.digits[i] as $int << shift;
                            i += 1;
                        }
                    }

                    #[allow(unused_comparisons)]
                    if out < 0 {
                        return Err(TryFromIntError(()));
                    }

                    while i < N {
                        if u.digits[i] != 0 {
                            return Err(TryFromIntError(()));
                        }
                        i += 1;
                    }

                    Ok(out)
                }
            }
        })*
    };
}

use crate::cast::CastFrom;
use crate::errors::TryFromIntError;
use crate::nightly::impl_const;

macro_rules! convert {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl_const! {
            impl<const N: usize> const From<bool> for $BUint<N> {
                #[inline]
                fn from(small: bool) -> Self {
                    Self::cast_from(small)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const From<char> for $BUint<N> {
                #[inline]
                fn from(c: char) -> Self {
                    Self::cast_from(c)
                }
            }
        }

        from_uint!($BUint, $Digit; u8, u16, u32, u64, u128, usize);
        // TODO: decide whether it should be TryFrom<usize> or From<usize>, same for $BInt

        try_from_iint!($BUint; i8 -> u8, i16 -> u16, i32 -> u32, isize -> usize, i64 -> u64, i128 -> u128);

        try_from_buint!($BUint, $Digit; u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize);

        impl_const! {
            impl<const N: usize> const From<[$Digit; N]> for $BUint<N> {
                #[inline]
                fn from(digits: [$Digit; N]) -> Self {
                    Self::from_digits(digits)
                }
            }
        }

        impl_const! {
            impl<const N: usize> const From<$BUint<N>> for [$Digit; N] {
                #[inline]
                fn from(uint: $BUint<N>) -> Self {
                    uint.digits
                }
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::{self, types::utest};

                // We can test with `TryFrom` for all types as `TryFrom` is automatically implemented when `From` is implemented
                test::test_from! {
                    function: <utest as TryFrom>::try_from,
                    from_types: (u8, u16, u32, u64, bool, char, i8, i16, i32, i64, isize, usize)
                }

                test::test_into! {
                    function: <utest as TryInto>::try_into,
                    into_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize)
                }
            }
        }
    };
}

crate::macro_impl!(convert);
