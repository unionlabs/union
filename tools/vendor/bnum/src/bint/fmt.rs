use core::fmt::{Binary, Debug, Display, Formatter, LowerExp, LowerHex, Octal, UpperExp, UpperHex};

macro_rules! fmt_trait {
    ($BInt: ident, $trait: tt) => {
        impl<const N: usize> $trait for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                $trait::fmt(&self.bits, f)
            }
        }
    };
}

macro_rules! fmt {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        fmt_trait!($BInt, Binary);

        impl<const N: usize> Display for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                f.pad_integral(!self.is_negative(), "", &format!("{}", self.unsigned_abs()))
            }
        }

        impl<const N: usize> Debug for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                Display::fmt(&self, f)
            }
        }

        impl<const N: usize> LowerExp for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                let uint = self.unsigned_abs();
                f.pad_integral(!self.is_negative(), "", &format!("{:e}", uint))
            }
        }
        fmt_trait!($BInt, LowerHex);
        fmt_trait!($BInt, Octal);

        impl<const N: usize> UpperExp for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                let uint = self.unsigned_abs();
                f.pad_integral(!self.is_negative(), "", &format!("{:E}", uint))
            }
        }
        
        fmt_trait!($BInt, UpperHex);

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                crate::int::fmt::tests!(itest);
            }
        }
    };
}

crate::macro_impl!(fmt);
