use crate::doc;
use crate::errors::ParseIntError;
use crate::int::radix::assert_range;
use alloc::string::String;
use alloc::vec::Vec;
use core::num::IntErrorKind;

macro_rules! radix {
    ($BUint: ident, $BInt: ident, $Digit: ident) => {
        #[doc = doc::radix::impl_desc!($BInt)]
        impl<const N: usize> $BInt<N> {
            /// Converts a byte slice in a given base to an integer. The input slice must contain ascii/utf8 characters in [0-9a-zA-Z].
            ///
            /// This function is equivalent to the [`from_str_radix`](#method.from_str_radix) function for a string slice equivalent to the byte slice and the same radix.
            ///
            /// Returns `None` if the conversion of the byte slice to string slice fails or if a digit is larger than or equal to the given radix, otherwise the integer is wrapped in `Some`.
            #[inline]
            pub const fn parse_bytes(buf: &[u8], radix: u32) -> Option<Self> {
                let s = crate::nightly::option_try!(crate::nightly::ok!(core::str::from_utf8(buf)));
                crate::nightly::ok!(Self::from_str_radix(s, radix))
            }

            crate::nightly::const_fns! {
                /// Converts a slice of big-endian digits in the given radix to an integer. The digits are first converted to an unsigned integer, then this is transmuted to a signed integer. Each `u8` of the slice is interpreted as one digit of base `radix` of the number, so this function will return `None` if any digit is greater than or equal to `radix`, otherwise the integer is wrapped in `Some`.
                ///
                /// For examples, see the
                #[doc = concat!("[`from_radix_be`](crate::", stringify!($BUint), "::from_radix_be) method documentation for [`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
                #[inline]
                pub const fn from_radix_be(buf: &[u8], radix: u32) -> Option<Self> {
                    match $BUint::from_radix_be(buf, radix) { // TODO: use Option::map when stable
                        Some(uint) => Some(Self::from_bits(uint)),
                        None => None,
                    }
                }

                /// Converts a slice of big-endian digits in the given radix to an integer. The digits are first converted to an unsigned integer, then this is transmuted to a signed integer. Each `u8` of the slice is interpreted as one digit of base `radix` of the number, so this function will return `None` if any digit is greater than or equal to `radix`, otherwise the integer is wrapped in `Some`.
                ///
                /// For examples, see the
                #[doc = concat!("[`from_radix_le`](crate::", stringify!($BUint), "::from_radix_le) method documentation for [`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
                #[inline]
                pub const fn from_radix_le(buf: &[u8], radix: u32) -> Option<Self> {
                    match $BUint::from_radix_le(buf, radix) { // TODO: use Option::map when stable
                        Some(uint) => Some(Self::from_bits(uint)),
                        None => None,
                    }
                }
            }

            /// Converts a string slice in a given base to an integer.
            ///
            /// The string is expected to be an optional `+` or `-` sign followed by digits. Leading and trailing whitespace represent an error. Digits are a subset of these characters, depending on `radix`:
            ///
            /// - `0-9`
            /// - `a-z`
            /// - `A-Z`
            ///
            /// # Panics
            ///
            /// This function panics if `radix` is not in the range from 2 to 36 inclusive.
            ///
            /// For examples, see the
            #[doc = concat!("[`from_str_radix`](crate::", stringify!($BUint), "::from_str_radix) method documentation for [`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
            #[inline]
            pub const fn from_str_radix(src: &str, radix: u32) -> Result<Self, ParseIntError> {
                assert_range!(radix, 36);
                if src.is_empty() {
                    return Err(ParseIntError {
                        kind: IntErrorKind::Empty,
                    });
                }
                let mut negative = false;
                let mut leading_sign = false;
                let buf = src.as_bytes();
                if buf[0] == b'-' {
                    negative = true;
                    leading_sign = true;
                } else if buf[0] == b'+' {
                    leading_sign = true;
                }

                match $BUint::from_buf_radix_internal::<true, true>(buf, radix, leading_sign) {
                    Ok(uint) => {
                        if negative {
                            if uint.bit(Self::BITS - 1) && uint.trailing_zeros() != Self::BITS - 1 {
                                Err(ParseIntError {
                                    kind: IntErrorKind::NegOverflow,
                                })
                            } else {
                                Ok(Self::from_bits(uint).wrapping_neg())
                            }
                        } else {
                            let out = Self::from_bits(uint);
                            if out.is_negative() {
                                Err(ParseIntError {
                                    kind: IntErrorKind::PosOverflow,
                                })
                            } else {
                                Ok(out)
                            }
                        }
                    }
                    Err(err) => {
                        if let IntErrorKind::PosOverflow = err.kind() {
                            if negative {
                                return Err(ParseIntError {
                                    kind: IntErrorKind::NegOverflow,
                                });
                            }
                        }
                        return Err(err)
                    }
                }
            }

            #[doc = doc::radix::parse_str_radix!($BUint)]
            #[inline]
            pub const fn parse_str_radix(src: &str, radix: u32) -> Self {
                match Self::from_str_radix(src, radix) {
                    Ok(n) => n,
                    Err(e) => panic!("{}", e.description()),
                }
            }

            /// Returns the integer as a string in the given radix.
            ///
            /// # Panics
            ///
            /// This function panics if `radix` is not in the range from 2 to 36 inclusive.
            ///
            /// For examples, see the
            #[doc = concat!("[`to_str_radix`](crate::", stringify!($BUint), "::to_str_radix) method documentation for [`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
            #[inline]
            pub fn to_str_radix(&self, radix: u32) -> String {
                if self.is_negative() {
                    format!("-{}", self.unsigned_abs().to_str_radix(radix))
                } else {
                    self.bits.to_str_radix(radix)
                }
            }

            /// Returns the integer's underlying representation as an unsigned integer in the given base in big-endian digit order.
            ///
            /// # Panics
            ///
            /// This function panics if `radix` is not in the range from 2 to 256 inclusive.
            ///
            /// For examples, see the
            #[doc = concat!("[`to_radix_be`](crate::", stringify!($BUint), "::to_radix_be) method documentation for [`", stringify!($BUint), "`]")]
            #[inline]
            pub fn to_radix_be(&self, radix: u32) -> Vec<u8> {
                self.bits.to_radix_be(radix)
            }

            /// Returns the integer's underlying representation as an unsigned integer in the given base in little-endian digit order.
            ///
            /// # Panics
            ///
            /// This function panics if `radix` is not in the range from 2 to 256 inclusive.
            ///
            /// For examples, see the
            #[doc = concat!("[`to_radix_le`](crate::", stringify!($BUint), "::to_radix_le) method documentation for [`", stringify!($BUint), "`](crate::", stringify!($BUint), ").")]
            #[inline]
            pub fn to_radix_le(&self, radix: u32) -> Vec<u8> {
                self.bits.to_radix_le(radix)
            }
        }

        #[cfg(test)]
        paste::paste! {
            mod [<$Digit _digit_tests>] {
                use crate::test::types::big_types::$Digit::*;
                use crate::test::{quickcheck_from_to_radix, test_bignum, self};
                use crate::$BInt;
                use crate::test::types::itest;

                test_bignum! {
                    function: <itest>::from_str_radix,
                    cases: [
                        ("-14359abcasdhfkdgdfgsde", 34u32),
                        ("+23797984569ahgkhhjdskjdfiu", 32u32),
                        ("-253613132341435345", 7u32),
                        ("+23467abcad47790809ef37", 16u32),
                        ("-712930769245766867875986646", 10u32),
                        ("-😱234292", 36u32),
                        ("-+345934758", 13u32),
                        ("12💯12", 15u32),
                        ("gap gap", 36u32),
                        ("-9223372036854775809", 10u32),
                        ("-1000000000000000000001", 8u32),
                        ("+1000000000000000000001", 8u32),
                        ("-8000000000000001", 16u32),
                        ("+-23459374", 15u32),
                        ("8000000000000000", 16u32)
                    ]
                }

                quickcheck_from_to_radix!(itest, radix_be, 256);
                quickcheck_from_to_radix!(itest, radix_le, 256);
                quickcheck_from_to_radix!(itest, str_radix, 36);

                test::quickcheck_from_str_radix!(itest, "+" | "-");
                test::quickcheck_from_str!(itest);

                #[test]
                fn from_to_radix_le() {
                    let buf = &[
                        61, 45, 48, 20, 37, 59, 53, 28, 28, 52, 54, 13, 44, 3, 46, 42, 20, 46, 37, 32,
                        13, 27, 47, 30, 33, 25, 3, 32, 4, 54, 53, 6, 44, 25, 10, 22, 33, 48, 7, 17,
                    ];
                    let u = $BInt::<100>::from_radix_le(buf, 64).unwrap();
                    let v = u.to_radix_le(64);
                    assert_eq!(v, buf);

                    let buf = &[
                        33, 34, 61, 53, 74, 67, 54, 62, 22, 29, 4, 2, 43, 73, 74, 24, 8, 74, 65, 3, 78,
                    ];
                    let option = $BInt::<100>::from_radix_le(buf, 78);
                    assert!(option.is_none());

                    let buf = &[
                        1, 3, 3, 0, 2, 1, 2, 3, 0, 4, 1, 2, 0, 0, 0, 0, 3, 2, 0, 1, 0, 4, 1, 3, 1, 4,
                        3, 3, 3, 4, 1, 2, 2, 1, 3, 0, 2, 1, 2, 3, 1, 1, 0, 2, 2, 1, 1, 2, 1, 0, 0, 0,
                        3, 3, 3, 0, 0, 4, 4, 2,
                    ];
                    let u = $BInt::<100>::from_radix_le(buf, 5).unwrap();
                    let v = u.to_radix_le(5);
                    assert_eq!(v, buf);
                }
                #[test]
                fn from_to_radix_be() {
                    let buf = &[
                        29, 89, 92, 118, 69, 140, 141, 70, 71, 76, 66, 13, 30, 28, 38, 145, 40, 7, 57,
                        18, 25, 65, 150, 119, 155, 18, 64, 76, 106, 87,
                    ];
                    let u = $BInt::<100>::from_radix_be(buf, 157).unwrap();
                    let v = u.to_radix_be(157);
                    assert_eq!(v, buf);

                    let buf = &[
                        1, 1, 0, 0, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 0, 0, 0, 0, 1, 0, 1, 0, 1, 1, 0, 1,
                        1, 1, 1, 1, 0, 1, 1, 1, 1, 1, 0, 1, 1, 1, 1, 0, 0, 0, 0, 0, 1, 0, 1, 1, 0, 0,
                        1, 1, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 1, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 1, 0,
                        0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 1, 0, 0, 1,
                    ];
                    let u = $BInt::<100>::from_radix_be(buf, 2).unwrap();
                    let v = u.to_radix_be(2);
                    assert_eq!(v, buf);

                    let buf = &[
                        91, 167, 5, 99, 61, 38, 158, 149, 115, 79, 13, 118, 53, 16, 144, 123, 70, 81,
                        78, 61, 39, 6, 34, 95, 98, 23, 175, 182,
                    ];
                    let option = $BInt::<100>::from_radix_le(buf, 180);
                    assert!(option.is_none());

                    let buf = &[
                        39, 90, 119, 93, 95, 7, 70, 81, 3, 100, 39, 107, 98, 31, 61, 5, 36, 19, 18,
                        124, 4, 77, 119, 17, 121, 116, 24, 35,
                    ];
                    let u = $BInt::<100>::from_radix_be(buf, 128).unwrap();
                    let v = u.to_radix_be(128);
                    assert_eq!(v, buf);
                }
                #[test]
                fn from_to_str_radix() {
                    let src = "-293487598aashkhkhakb8345cbvjkus";
                    let u = $BInt::<100>::from_str_radix(src, 35).unwrap();
                    let v = u.to_str_radix(35);
                    assert_eq!(v, src);

                    let src = "zzzzzzzzzzzzzzzzzzzzzzzzz";
                    let result = $BInt::<1>::from_str_radix(src, 36);
                    assert!(result.is_err());

                    let invalid = "inval_id string";
                    let result = $BInt::<1>::from_str_radix(invalid, 36);
                    assert!(result.is_err());

                    let src = "72954hslfhbui79845y6audfgiu984h5ihhhdfg";
                    let u = $BInt::<100>::from_str_radix(src, 36).unwrap();
                    assert_eq!(u.to_str_radix(36), src);
                }
                #[test]
                fn parse_bytes() {
                    let src = "1797972456987acbdead7889";
                    let u = $BInt::<100>::parse_bytes(src.as_bytes(), 16).unwrap();
                    let v = $BInt::<100>::from_str_radix(src, 16).unwrap();
                    assert_eq!(u, v);
                    assert_eq!(v.to_str_radix(16), src);

                    let bytes = b"279874657dgfhjh";
                    let option = $BInt::<100>::parse_bytes(bytes, 11);
                    assert!(option.is_none());
                }
            }
        }
    };
}

crate::macro_impl!(radix);
