macro_rules! from_int {
    ($BInt: ident, $Digit: ident; $($int: tt),*) => {
        $(impl_const! {
            impl<const N: usize> const From<$int> for $BInt<N> {
                #[inline]
                fn from(int: $int) -> Self {
                    let mut out = if int.is_negative() {
                        !Self::ZERO
                    } else {
                        Self::ZERO
                    };
                    let mut i = 0;
                    while i << crate::digit::$Digit::BIT_SHIFT < $int::BITS as usize {
                        let d = (int >> (i << crate::digit::$Digit::BIT_SHIFT)) as $Digit;
                        out.bits.digits[i] = d;
                        i += 1;
                    }
                    out
                }
            }
        })*
    }
}

macro_rules! from_uint {
    ($BInt: ident, $BUint: ident; $($from: tt), *) => {
        $(impl_const! {
            impl<const N: usize> const From<$from> for $BInt<N> {
                #[inline]
                fn from(int: $from) -> Self {
                    let out = Self::from_bits($BUint::from(int));
                    out
                }
            }
        })*
    }
}

macro_rules! int_try_from_bint {
    { $BInt: ident, $Digit: ident; $($int: ty), * }  => {
        $(crate::nightly::impl_const! {
            impl<const N: usize> const TryFrom<$BInt<N>> for $int {
                type Error = TryFromIntError;

                fn try_from(int: $BInt<N>) -> Result<$int, Self::Error> {
                    let neg = int.is_negative();
                    let (mut out, padding) = if neg {
                        (-1, $Digit::MAX)
                    } else {
                        (0, $Digit::MIN)
                    };
                    let mut i = 0;
                    if $Digit::BITS > <$int>::BITS {
                        let small = int.bits.digits[i] as $int;
                        let trunc = small as $Digit;
                        if int.bits.digits[i] != trunc {
                            return Err(TryFromIntError(()));
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
                                out &= !((!int.bits.digits[i]) as $int << shift);
                                i += 1;
                            }
                        } else {
                            loop {
                                let shift = i << digit::$Digit::BIT_SHIFT;
                                if i >= N || shift >= <$int>::BITS as usize {
                                    break;
                                }
                                out |= int.bits.digits[i] as $int << shift;
                                i += 1;
                            }
                        }
                    }

                    while i < N {
                        if int.bits.digits[i] != padding {
                            return Err(TryFromIntError(()));
                        }
                        i += 1;
                    }

                    if out.is_negative() != neg {
                        return Err(TryFromIntError(()));
                    }

                    Ok(out)
                }
            }
        })*
    };
}

macro_rules! uint_try_from_bint {
    ($BInt: ident; $($uint: ty), *) => {
        $(crate::nightly::impl_const! {
            impl<const N: usize> const TryFrom<$BInt<N>> for $uint {
                type Error = TryFromIntError;

                #[inline]
                fn try_from(int: $BInt<N>) -> Result<$uint, Self::Error> {
                    if int.is_negative() {
                        Err(TryFromIntError(()))
                    } else {
                        <$uint>::try_from(int.bits)
                    }
                }
            }
        })*
    };
}

use crate::cast::CastFrom;
use crate::digit;
use crate::errors::{ParseIntError, TryFromIntError};
use crate::nightly::impl_const;
use core::str::FromStr;

macro_rules! convert {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl<const N: usize> FromStr for $BInt<N> {
            type Err = ParseIntError;

            #[inline]
            fn from_str(src: &str) -> Result<Self, Self::Err> {
                Self::from_str_radix(src, 10)
            }
        }

        from_int!($BInt, $Digit; i8, i16, i32, i64, i128, isize);

        from_uint!($BInt, $BUint; u8, u16, u32, u64, u128, usize);

        impl_const! {
            impl<const N: usize> const From<bool> for $BInt<N> {
                #[inline]
                fn from(small: bool) -> Self {
                    Self::cast_from(small)
                }
            }
        }

        int_try_from_bint!($BInt, $Digit; i8, i16, i32, i64, i128, isize);
        uint_try_from_bint!($BInt; u8, u16, u32, u64, u128, usize);

        impl_const! {
            impl<const N: usize> const TryFrom<$BUint<N>> for $BInt<N> {
                type Error = TryFromIntError;

                #[inline]
                fn try_from(u: $BUint<N>) -> Result<Self, Self::Error> {
                    if u.leading_ones() != 0 {
                        Err(TryFromIntError(()))
                    } else {
                        Ok(Self::from_bits(u))
                    }
                }
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::{self, types::itest};

                #[cfg(not(test_int_bits = "64"))]
                test::test_from! {
                    function: <itest as TryFrom>::try_from,
                    from_types: (i8, i16, i32, i64, i128, u8, u16, u32, u64, bool, usize, isize)
                }

                #[cfg(test_int_bits = "64")]
                test::test_from! {
                    function: <itest as TryFrom>::try_from,
                    from_types: (i8, i16, i32, i64, u8, u16, u32, bool, isize)
                }

                test::test_into! {
                    function: <itest as TryInto>::try_into,
                    into_types: (u8, u16, u32, u64, usize, u128, i8, i16, i32, i64, i128, isize)
                }
            }
        }
    };
}

crate::macro_impl!(convert);
