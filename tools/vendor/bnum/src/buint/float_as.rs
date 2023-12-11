use crate::ExpType;
use crate::cast::CastFrom;

pub trait Mantissa: Sized {
    const ZERO: Self;
    
    fn checked_shr(self, n: ExpType) -> Option<Self>;
    fn bits(&self) -> ExpType;
}

pub trait Exponent {
    fn is_negative(&self) -> bool;
    fn neg(self) -> Self;
}

pub trait CastUintFromFloatConsts {
    const ZERO: Self;
    const MIN: Self;
    const MAX: Self;
    const BITS: ExpType;

    fn shl(self, n: ExpType) -> Self;
}

pub trait CastUintFromFloatHelper: CastFrom<Self::M> {
    type M: Mantissa;
    type E: Exponent;

    fn is_nan(&self) -> bool;
    fn is_sign_negative(&self) -> bool;
    fn is_infinite(&self) -> bool;
    fn decode(self) -> (Self::M, Self::E);
}

macro_rules! cast_uint_from_float_consts {
    ($($uint: ident), *) => {
        $(
            impl CastUintFromFloatConsts for $uint {
                const ZERO: Self = 0;
                const MIN: Self = Self::MIN;
                const MAX: Self = Self::MAX;
                const BITS: ExpType = Self::BITS;
            
                fn shl(self, n: ExpType) -> Self {
                    self << n
                }
            }
        )*
    };
}

pub(crate) use cast_uint_from_float_consts;

macro_rules! as_float {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl<const N: usize> Mantissa for $BUint<N> {
            const ZERO: Self = Self::ZERO;
        
            #[inline]
            fn checked_shr(self, n: ExpType) -> Option<Self> {
                Self::checked_shr(self, n)
            }
        
            #[inline]
            fn bits(&self) -> ExpType {
                Self::bits(&self)
            }
        }
        
        impl<const N: usize> Exponent for $BInt<N> {
            #[inline]
            fn is_negative(&self) -> bool {
                Self::is_negative(*self)
            }
        
            #[inline]
            fn neg(self) -> Self {
                Self::neg(self)
            }
        }
        
        impl<const N: usize> CastUintFromFloatConsts for $BUint<N> {
            const ZERO: Self = Self::ZERO;
            const MIN: Self = Self::MIN;
            const MAX: Self = Self::MAX;
            const BITS: ExpType = Self::BITS;
        
            fn shl(self, n: ExpType) -> Self {
                Self::shl(self, n)
            }
        }
    };
}

pub(crate) use as_float;

pub fn uint_cast_from_float<F, U>(f: F) -> U
where
    F: CastUintFromFloatHelper,
    U: CastUintFromFloatConsts + CastFrom<F::M>,
    ExpType: TryFrom<F::E>
{
    if f.is_nan() {
        return U::ZERO;
    }
    if f.is_sign_negative() {
        return U::MIN;
    }
    if f.is_infinite() {
        return U::MAX;
    }
    let (mut mant, exp) = f.decode();
    if exp.is_negative() {
        let r: Result<ExpType, _> = exp.neg().try_into();
        mant = match r {
            Ok(exp) => mant.checked_shr(exp).unwrap_or(<F::M>::ZERO),
            _ => <F::M>::ZERO,
        };
        if mant.bits() > U::BITS {
            return U::MAX;
        }
        U::cast_from(mant)
    } else {
        let exp: Result<ExpType, _> = exp.try_into();
        match exp {
            Ok(exp) => {
                if mant.bits() + exp > U::BITS {
                    return U::MAX;
                }
                U::cast_from(mant).shl(exp)
            },
            _ => U::MAX
        }
    }
}

crate::macro_impl!(as_float);

crate::buint::float_as::cast_uint_from_float_consts!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_mantissa_for_uint {
    ($($uint: ty), *) => {
        $(
            impl Mantissa for $uint {
                const ZERO: Self = 0;

                #[inline]
                fn checked_shr(self, n: ExpType) -> Option<Self> {
                    self.checked_shr(n as u32)
                }

                #[inline]
                fn bits(&self) -> ExpType {
                    (Self::BITS - self.leading_zeros()) as ExpType
                }
            }
        )*
    }
}

impl_mantissa_for_uint!(u32, u64);

impl Exponent for i16 {
    #[inline]
    fn is_negative(&self) -> bool {
        Self::is_negative(*self)
    }

    #[inline]
    fn neg(self) -> Self {
        -self
    }
}

macro_rules! impl_cast_uint_from_primitive_float_helper {
    ($float: ty, $mant: ty, $exp: ty, $decoder: ident) => {
        impl CastUintFromFloatHelper for $float {
            type M = $mant;
            type E = $exp;

            #[inline]
            fn is_nan(&self) -> bool {
                Self::is_nan(*self)
            }

            #[inline]
            fn is_sign_negative(&self) -> bool {
                Self::is_sign_negative(*self)
            }

            #[inline]
            fn is_infinite(&self) -> bool {
                Self::is_infinite(*self)
            }

            #[inline]
            fn decode(self) -> (Self::M, Self::E) {
                crate::buint::cast::$decoder(self)
            }
        }
    };
}

impl_cast_uint_from_primitive_float_helper!(f32, u32, i16, decode_f32);
impl_cast_uint_from_primitive_float_helper!(f64, u64, i16, decode_f64);