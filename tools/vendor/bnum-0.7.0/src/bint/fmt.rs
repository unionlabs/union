use core::fmt::{Binary, Debug, Display, Formatter, LowerExp, LowerHex, Octal, UpperExp, UpperHex};

macro_rules! fmt_method {
    ($format: expr, $pad: expr, $prefix: expr, $trait: tt) => {
        #[inline]
        fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
            $trait::fmt(&self.bits, f)
        }
    };
}

macro_rules! fmt {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        impl<const N: usize> Binary for $BInt<N> {
            fmt_method!("{:b}{:0pad$b}", Self::BITS, "0b", Binary);
        }

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

        impl<const N: usize> LowerHex for $BInt<N> {
            fmt_method!("{:x}{:0pad$x}", Self::BITS / 4, "0x", LowerHex);
        }

        impl<const N: usize> Octal for $BInt<N> {
            fmt_method!("{:o}{:0pad$o}", Self::BITS / 4, "0o", Octal);
        }

        impl<const N: usize> UpperExp for $BInt<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                let uint = self.unsigned_abs();
                f.pad_integral(!self.is_negative(), "", &format!("{:E}", uint))
            }
        }

        impl<const N: usize> UpperHex for $BInt<N> {
            fmt_method!("{:X}{:0pad$X}", Self::BITS / 4, "0x", UpperHex);
        }

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
