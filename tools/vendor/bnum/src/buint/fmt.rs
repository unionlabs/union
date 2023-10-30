use crate::digit;
use alloc::string::String;
use core::fmt::Write;
use core::fmt::{Binary, Debug, Display, Formatter, LowerExp, LowerHex, Octal, UpperExp, UpperHex};

macro_rules! fmt {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        macro_rules! fmt_method {
            ($format: expr, $format_pad: expr, $pad: expr, $prefix: expr) => {
                #[inline]
                fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                    let mut format_string = String::new();
                    for digit in self.digits.iter().rev() {
                        if format_string.is_empty() {
                            if digit != &0 {
                                write!(format_string, $format, digit)?;
                            }
                        } else {
                            write!(format_string, $format_pad, digit, $pad)?;
                        }
                    }
                    f.pad_integral(
                        true,
                        $prefix,
                        if format_string.is_empty() {
                            "0"
                        } else {
                            &format_string
                        },
                    )
                }
            };
        }

        impl<const N: usize> Binary for $BUint<N> {
            fmt_method!("{:b}", "{:01$b}", digit::$Digit::BITS as usize, "0b");
        }

        impl<const N: usize> Debug for $BUint<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                Display::fmt(&self, f)
            }
        }

        impl<const N: usize> Display for $BUint<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                write!(f, "{}", self.to_str_radix(10))
            }
        }

        macro_rules! exp_fmt {
            ($e: expr) => {
                #[inline]
                fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                    let decimal_str = format!("{}", self);
                    if decimal_str == "0" {
                        return write!(f, "{}{}0", 0, $e);
                    }
                    let exp = decimal_str.len() - 1;
                    let decimal_str = decimal_str.trim_end_matches('0');
                    if decimal_str.len() == 1 {
                        write!(f, "{}{}{}", &decimal_str[0..1], $e, exp)
                    } else {
                        write!(
                            f,
                            "{}.{}{}{}",
                            &decimal_str[0..1],
                            &decimal_str[1..],
                            $e,
                            exp
                        )
                    }
                }
            };
        }

        impl<const N: usize> LowerExp for $BUint<N> {
            exp_fmt!("e");
        }

        impl<const N: usize> LowerHex for $BUint<N> {
            fmt_method!("{:x}", "{:01$x}", digit::$Digit::HEX_PADDING, "0x");
        }

        impl<const N: usize> Octal for $BUint<N> {
            #[inline]
            fn fmt(&self, f: &mut Formatter) -> core::fmt::Result {
                let string = self.to_str_radix(8);
                f.pad_integral(true, "0o", &string)
            }
        }

        impl<const N: usize> UpperExp for $BUint<N> {
            exp_fmt!("E");
        }

        impl<const N: usize> UpperHex for $BUint<N> {
            fmt_method!("{:X}", "{:01$X}", digit::$Digit::HEX_PADDING, "0x");
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                crate::int::fmt::tests!(utest);
            }
        }
    };
}

crate::macro_impl!(fmt);
