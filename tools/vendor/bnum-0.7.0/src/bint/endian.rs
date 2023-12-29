use crate::digit;
use crate::doc;
use core::mem::MaybeUninit;

macro_rules! endian {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        macro_rules! set_digit {
            ($out_digits: ident, $i: expr, $digit: expr, $is_negative: expr, $sign_bits: expr) => {
                if $i == Self::N_MINUS_1 {
                    if ($digit as digit::$Digit::SignedDigit).is_negative() == $is_negative {
                        $out_digits[$i] = $digit;
                    } else {
                        return None;
                    }
                } else if $i < N {
                    $out_digits[$i] = $digit;
                } else if $digit != $sign_bits {
                    return None;
                };
            };
        }

        #[doc = doc::endian::impl_desc!($BInt)]
        impl<const N: usize> $BInt<N> {
            #[doc = doc::endian::from_be!(I 256)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn from_be(x: Self) -> Self {
                Self::from_bits($BUint::from_be(x.bits))
            }

            #[doc = doc::endian::from_le!(I 256)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn from_le(x: Self) -> Self {
                Self::from_bits($BUint::from_le(x.bits))
            }

            #[doc = doc::endian::to_be!(I 256)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn to_be(self) -> Self {
                Self::from_be(self)
            }

            #[doc = doc::endian::to_le!(I 256)]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn to_le(self) -> Self {
                Self::from_le(self)
            }

            /// Create an integer value from a slice of bytes in big endian. The value is wrapped in an [`Option`](https://doc.rust-lang.org/core/option/enum.Option.html) as the integer represented by the slice of bytes may represent an integer too large to be represented by the type.
            ///
            /// If the length of the slice is shorter than `Self::BYTES`, the slice is padded with zeros or ones at the start so that it's length equals `Self::BYTES`. It is padded with ones if the bytes represent a negative integer, otherwise it is padded with zeros.
            ///
            /// If the length of the slice is longer than `Self::BYTES`, `None` will be returned, unless the bytes represent a non-negative integer and leading zeros from the slice can be removed until the length of the slice equals `Self::BYTES`, or if the bytes represent a negative integer and leading ones from the slice can be removed until the length of the slice equals `Self::BYTES`.
            ///
            /// For examples, see the
            #[doc = concat!("[`from_be_slice`](crate::", stringify!($BUint), "::from_be_slice)")]
            /// method documentation for
            #[doc = concat!("[`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
            #[must_use = doc::must_use_op!()]
            pub const fn from_be_slice(slice: &[u8]) -> Option<Self> {
                let len = slice.len();
                if len == 0 {
                    return Some(Self::ZERO);
                }
                let is_negative = (slice[0] as i8).is_negative();
                let sign_bits = if is_negative { $Digit::MAX } else { $Digit::MIN };
                let mut out_digits = if is_negative { [$Digit::MAX; N] } else { [0; N] };
                let slice_ptr = slice.as_ptr();
                let mut i = 0;
                let exact = len >> digit::$Digit::BYTE_SHIFT;
                while i < exact {
                    let uninit = MaybeUninit::<[u8; digit::$Digit::BYTES as usize]>::uninit();
                    let ptr = uninit.as_ptr().cast_mut() as *mut u8;
                    let digit_bytes = unsafe {
                        slice_ptr
                            .add(len - digit::$Digit::BYTES as usize - (i << digit::$Digit::BYTE_SHIFT))
                            .copy_to_nonoverlapping(ptr, digit::$Digit::BYTES as usize);
                        uninit.assume_init()
                    };
                    let digit = $Digit::from_be_bytes(digit_bytes);
                    set_digit!(out_digits, i, digit, is_negative, sign_bits);
                    i += 1;
                }
                let rem = len & (digit::$Digit::BYTES as usize - 1);
                if rem == 0 {
                    Some(Self::from_bits($BUint::from_digits(out_digits)))
                } else {
                    let pad_byte = if is_negative { u8::MAX } else { 0 };
                    let mut last_digit_bytes = [pad_byte; digit::$Digit::BYTES as usize];
                    let mut j = 0;
                    while j < rem {
                        last_digit_bytes[digit::$Digit::BYTES as usize - rem + j] = slice[j];
                        j += 1;
                    }
                    let digit = $Digit::from_be_bytes(last_digit_bytes);
                    set_digit!(out_digits, i, digit, is_negative, sign_bits);
                    Some(Self::from_bits($BUint::from_digits(out_digits)))
                }
            }

            /// Creates an integer value from a slice of bytes in little endian. The value is wrapped in an [`Option`](https://doc.rust-lang.org/core/option/enum.Option.html) as the bytes may represent an integer too large to be represented by the type.
            ///
            /// If the length of the slice is shorter than `Self::BYTES`, the slice is padded with zeros or ones at the end so that it's length equals `Self::BYTES`. It is padded with ones if the bytes represent a negative integer, otherwise it is padded with zeros.
            ///
            /// If the length of the slice is longer than `Self::BYTES`, `None` will be returned, unless the bytes represent a non-negative integer and trailing zeros from the slice can be removed until the length of the slice equals `Self::BYTES`, or if the bytes represent a negative integer and trailing ones from the slice can be removed until the length of the slice equals `Self::BYTES`.
            ///
            /// For examples, see the
            #[doc = concat!("[`from_le_slice`](crate::", stringify!($BUint), "::from_le_slice)")]
            /// method documentation for
            #[doc = concat!("[`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
            #[must_use = doc::must_use_op!()]
            pub const fn from_le_slice(slice: &[u8]) -> Option<Self> {
                let len = slice.len();
                if len == 0 {
                    return Some(Self::ZERO);
                }
                let is_negative = (slice[len - 1] as i8).is_negative();
                let sign_bits = if is_negative { $Digit::MAX } else { $Digit::MIN };
                let mut out_digits = [sign_bits; N];
                let slice_ptr = slice.as_ptr();
                let mut i = 0;
                let exact = len >> digit::$Digit::BYTE_SHIFT;
                while i < exact {
                    let uninit = MaybeUninit::<[u8; digit::$Digit::BYTES as usize]>::uninit();
                    let ptr = uninit.as_ptr().cast_mut() as *mut u8; // TODO: can change to as_mut_ptr() when const_mut_refs is stabilised
                    let digit_bytes = unsafe {
                        slice_ptr
                            .add(i << digit::$Digit::BYTE_SHIFT)
                            .copy_to_nonoverlapping(ptr, digit::$Digit::BYTES as usize);
                        uninit.assume_init()
                    };
                    let digit = $Digit::from_le_bytes(digit_bytes);
                    set_digit!(out_digits, i, digit, is_negative, sign_bits);
                    i += 1;
                }
                if len & (digit::$Digit::BYTES as usize - 1) == 0 {
                    Some(Self::from_bits($BUint::from_digits(out_digits)))
                } else {
                    let pad_byte = if is_negative { u8::MAX } else { 0 };
                    let mut last_digit_bytes = [pad_byte; digit::$Digit::BYTES as usize];
                    let addition = exact << digit::$Digit::BYTE_SHIFT;
                    let mut j = 0;
                    while j + addition < len {
                        last_digit_bytes[j] = slice[j + addition];
                        j += 1;
                    }
                    let digit = $Digit::from_le_bytes(last_digit_bytes);
                    set_digit!(out_digits, i, digit, is_negative, sign_bits);
                    Some(Self::from_bits($BUint::from_digits(out_digits)))
                }
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::to_be_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn to_be_bytes(self) -> [u8; N * digit::$Digit::BYTES as usize] {
                self.bits.to_be_bytes()
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::to_le_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn to_le_bytes(self) -> [u8; N * digit::$Digit::BYTES as usize] {
                self.bits.to_le_bytes()
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::to_ne_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use = doc::must_use_op!()]
            #[inline]
            pub const fn to_ne_bytes(self) -> [u8; N * digit::$Digit::BYTES as usize] {
                self.bits.to_ne_bytes()
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::from_be_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use]
            #[inline]
            pub const fn from_be_bytes(bytes: [u8; N * digit::$Digit::BYTES as usize]) -> Self {
                Self::from_bits($BUint::from_be_bytes(bytes))
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::from_le_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use]
            #[inline]
            pub const fn from_le_bytes(bytes: [u8; N * digit::$Digit::BYTES as usize]) -> Self {
                Self::from_bits($BUint::from_le_bytes(bytes))
            }

            #[cfg(feature = "nightly")]
            #[doc = doc::endian::from_ne_bytes!(I)]
            #[doc = doc::requires_feature!("nightly")]
            #[must_use]
            #[inline]
            pub const fn from_ne_bytes(bytes: [u8; N * digit::$Digit::BYTES as usize]) -> Self {
                Self::from_bits($BUint::from_ne_bytes(bytes))
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::test_bignum;
                use crate::test::types::itest;

                crate::int::endian::tests!($Digit; itest);
            }
        }
    };
}

crate::macro_impl!(endian);
