use crate::ExpType;
use crate::cast::CastFrom;

#[cfg_attr(feature = "nightly", const_trait)]
pub trait CastToFloatHelper: Copy {
    const ZERO: Self;
    const BITS: ExpType;

    fn is_zero(&self) -> bool;
    fn bits(&self) -> ExpType;
    fn bit(&self, index: ExpType) -> bool;
    fn trailing_zeros(self) -> ExpType;
    fn shr(self, n: ExpType) -> Self;
}

macro_rules! impl_helper_uint {
    ($($uint: ty), *) => {
        $(
            crate::nightly::const_impl! {
                impl const CastToFloatHelper for $uint {
                    const ZERO: Self = 0;
                    const BITS: ExpType = Self::BITS as ExpType;

                    #[inline]
                    fn is_zero(&self) -> bool {
                        *self == 0
                    }

                    #[inline]
                    fn bits(&self) -> ExpType {
                        (<$uint>::BITS - self.leading_zeros()) as ExpType
                    }

                    #[inline]
                    fn bit(&self, index: ExpType) -> bool {
                        *self & (1 << index) != 0
                    }

                    #[inline]
                    fn trailing_zeros(self) -> ExpType {
                        Self::trailing_zeros(self) as ExpType
                    }

                    #[inline]
                    fn shr(self, n: ExpType) -> Self {
                        self >> n
                    }
                }
            }
        )*
    };
}

impl_helper_uint!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_helper_buint {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        crate::nightly::const_impl! {
            impl<const N: usize> const CastToFloatHelper for $BUint<N> {
                const ZERO: Self = Self::ZERO;
                const BITS: ExpType = Self::BITS;

                #[inline]
                fn is_zero(&self) -> bool {
                    Self::is_zero(&self)
                }

                #[inline]
                fn bits(&self) -> ExpType {
                    Self::bits(&self)
                }

                #[inline]
                fn bit(&self, index: ExpType) -> bool {
                    Self::bit(&self, index)
                }

                #[inline]
                fn trailing_zeros(self) -> ExpType {
                    Self::trailing_zeros(self)
                }

                #[inline]
                fn shr(self, n: ExpType) -> Self {
                    Self::shr(self, n)
                }
            }
        }
    };
}

pub(crate) use impl_helper_buint;

crate::macro_impl!(impl_helper_buint);

#[cfg_attr(feature = "nightly", const_trait)]
pub trait CastToFloatConsts {
    type M: Mantissa;

    const ZERO: Self;
    const MANTISSA_DIGITS: ExpType;
    const MAX_EXP: Self::M;
    const INFINITY: Self;
    //const MANTISSA_MASK: Self::M = (Self::M::MAX.shr(Self::BITS - (Self::MANTISSA_DIGITS - 1)));

    fn from_raw_parts(exp: Self::M, mant: Self::M) -> Self;
}

macro_rules! cast_to_float_consts {
    ($($f: ty; $u: ty), *) => {
        $(
           // crate::nightly::const_impl! {
                impl CastToFloatConsts for $f {
                    type M = $u;

                    const ZERO: Self = 0.0;
                    const MANTISSA_DIGITS: ExpType = <$f>::MANTISSA_DIGITS as ExpType;
                    const MAX_EXP: Self::M = <$f>::MAX_EXP as $u;
                    const INFINITY: Self = <$f>::INFINITY;

                    #[inline]
                    fn from_raw_parts(exp: Self::M, mant: Self::M) -> Self {
                        const MANTISSA_MASK: $u = <$u>::MAX >> (<$u>::BITS - (<$f>::MANTISSA_DIGITS - 1));
                        <$f>::from_bits((exp << Self::MANTISSA_DIGITS - 1) | mant & MANTISSA_MASK)
                    }
                }
           // }
        )*
    };
}

cast_to_float_consts!(f32; u32, f64; u64);

#[cfg_attr(feature = "nightly", const_trait)]
pub trait Mantissa {
    const ONE: Self;
    const TWO: Self;
    const MAX: Self;
    const BITS: ExpType;

    fn bit(&self, n: ExpType) -> bool;
    fn shl(self, n: ExpType) -> Self;
    fn shr(self, n: ExpType) -> Self;
    fn add(self, rhs: Self) -> Self;
    fn sub(self, rhs: Self) -> Self;
    fn leading_zeros(self) -> ExpType;
    fn bitand(self, rhs: Self) -> Self;
    fn gt(&self, rhs: &Self) -> bool;
}

macro_rules! impl_mantissa_for_uint {
    ($($uint: ty), *) => {
        $(
            crate::nightly::const_impl! {
                impl const Mantissa for $uint {
                    const ONE: Self = 1;
                    const TWO: Self = 2;
                    const MAX: Self = Self::MAX;
                    const BITS: ExpType = Self::BITS as ExpType;

                    #[inline]
                    fn bit(&self, n: ExpType) -> bool {
                        *self & (1 << n) != 0
                    }

                    #[inline]
                    fn shl(self, n: ExpType) -> Self {
                        self << n
                    }

                    #[inline]
                    fn shr(self, n: ExpType) -> Self {
                        self >> n
                    }

                    #[inline]
                    fn add(self, rhs: Self) -> Self {
                        self + rhs
                    }

                    #[inline]
                    fn sub(self, rhs: Self) -> Self {
                        self - rhs
                    }

                    #[inline]
                    fn leading_zeros(self) -> ExpType {
                        Self::leading_zeros(self) as ExpType
                    }

                    #[inline]
                    fn bitand(self, rhs: Self) -> Self {
                        self & rhs
                    }

                    #[inline]
                    fn gt(&self, rhs: &Self) -> bool {
                        *self > *rhs
                    }
                }
            }
        )*
    };
}

impl_mantissa_for_uint!(u32, u64);

/*macro_rules! impl_mantissa_for_buint {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl<const N: usize> Mantissa for $BUint<N> {
            const ONE: Self = Self::ONE;
            const TWO: Self = Self::TWO;
            const MAX: Self = Self::MAX;
            const BITS: ExpType = Self::BITS;

            #[inline]
            fn bit(&self, n: ExpType) -> bool {
                Self::bit(&self, n)
            }

            #[inline]
            fn shl(self, n: ExpType) -> Self {
                Self::shl(self, n)
            }

            #[inline]
            fn shr(self, n: ExpType) -> Self {
                Self::shr(self, n)
            }

            #[inline]
            fn add(self, rhs: Self) -> Self {
                Self::add(self, rhs)
            }

            #[inline]
            fn sub(self, rhs: Self) -> Self {
                Self::sub(self, rhs)
            }

            #[inline]
            fn leading_zeros(self) -> ExpType {
                Self::leading_zeros(self)
            }

            #[inline]
            fn bitand(self, rhs: Self) -> Self {
                Self::bitand(self, rhs)
            }

            #[inline]
            fn gt(&self, rhs: &Self) -> bool {
                Self::gt(&self, &rhs)
            }
        }
    };
}

pub(crate) use impl_mantissa_for_buint;

crate::macro_impl!(impl_mantissa_for_buint);*/

pub fn cast_float_from_uint<F, U>(from: U) -> F
where
    F: CastToFloatConsts,
    U: CastToFloatHelper,
    F::M: CastFrom<U> + CastFrom<ExpType> + Copy
{
    if from.is_zero() {
        return F::ZERO;
    }
    let bits = from.bits();
    let mut mant = if U::BITS > F::M::BITS {
        if bits < F::MANTISSA_DIGITS {
            F::M::cast_from(from).shl(F::MANTISSA_DIGITS - bits)
        } else {
            F::M::cast_from(from.shr(bits - F::MANTISSA_DIGITS))
        }
    } else if bits < F::MANTISSA_DIGITS {
        F::M::cast_from(from).shl(F::MANTISSA_DIGITS - bits)
    } else {
        F::M::cast_from(from).shr(bits - F::MANTISSA_DIGITS)
    };
    let mut round_up = true;
    if bits <= F::MANTISSA_DIGITS
        || !from.bit(bits - (F::MANTISSA_DIGITS + 1))
        || (!mant.bit(0)
            && from.trailing_zeros() == bits - (F::MANTISSA_DIGITS + 1))
    {
        round_up = false;
    }
    let mut exp = F::M::cast_from(bits).add(F::MAX_EXP).sub(F::M::TWO);
    if round_up {
        mant = mant.add(F::M::ONE);
        if mant.leading_zeros() == F::M::BITS - (F::MANTISSA_DIGITS + 1) {
            exp = exp.add(F::M::ONE);
        }
    }
    if exp.gt(&(F::MAX_EXP.shl(1)).sub(F::M::ONE)) {
        return F::INFINITY;
    }
    F::from_raw_parts(exp, mant)
}