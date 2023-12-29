pub mod bigint_helpers;
pub mod checked;
pub mod const_trait_fillers;
pub mod consts;
pub mod endian;
pub mod overflowing;
pub mod radix;
pub mod saturating;
pub mod unchecked;

pub mod wrapping;

macro_rules! arithmetic_doc {
    ($Int: ty) => {
concat!("`", stringify!($Int), "` implements all the arithmetic traits from the [`core::ops`](https://doc.rust-lang.org/core/ops/) module. The behaviour of the implementation of these traits is the same as for Rust's primitive integers - i.e. in debug mode it panics on overflow, and in release mode it performs two's complement wrapping (see <https://doc.rust-lang.org/book/ch03-02-data-types.html#integer-overflow>). However, an attempt to divide by zero or calculate a remainder with a divisor of zero will always panic, unless the [`checked_`](#method.checked_div) methods are used, which never panic.")
    }
}

pub(crate) use arithmetic_doc;

macro_rules! must_use_op {
    () => {
        "this returns the result of the operation, without modifying the original"
    };
}

pub(crate) use must_use_op;

macro_rules! arithmetic_impl_desc {
    ($name: literal, $method: literal, $rest: literal) => {
        concat!(
            $name,
            " arithmetic methods which act on `self`: `self.",
            $method,
            "_...`. ",
            $rest
        )
    };
}

pub(crate) use arithmetic_impl_desc;

#[cfg(feature = "nightly")]
macro_rules! requires_feature {
    ($feature: literal) => {
        concat!(
            "\n\nThis is supported on the crate feature `",
            $feature,
            "` only."
        )
    };
}

#[cfg(feature = "nightly")]
pub(crate) use requires_feature;

macro_rules! type_str {
    ($sign: ident $bits: literal) => {
        concat!(stringify!($sign), $bits)
    };
}

pub(crate) use type_str;

macro_rules! example_header {
    ($sign: ident $bits: literal) => {
        concat!(
"

# Examples
        
```
use bnum::types::",
            doc::type_str!($sign $bits),
";

"
        )
    }
}

pub(crate) use example_header;

macro_rules! small_sign {
    (U) => {
        "u"
    };
    (I) => {
        "i"
    };
}

pub(crate) use small_sign;

macro_rules! doc_comment {
    { $(# $method: ident, )? $sign: ident $bits: literal, $($($desc: expr)+)? $(, $($code: expr)+)? } => {
        concat!(
            $($("\n\n", $desc), +,)?
            $("\n\n", "See also: <https://doc.rust-lang.org/std/primitive.", doc::small_sign!($sign), "64.html#method.", stringify!($method), ">.", )?
            $(
                doc::example_header!($sign $bits),
                $($code), +,
                "\n```"
            )?
        )
    }
}

macro_rules! link_doc_comment {
    ($($name: ident), *) => {
        $(
            macro_rules! $name {
                ($sign: ident) => {
                    doc::doc_comment! {
                        #$name,
                        $sign 256,
                    }
                };
            }

            pub(crate) use $name;
        )*
    }
}

pub(crate) use link_doc_comment;

pub(crate) use doc_comment;

macro_rules! count_ones {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #count_ones,
            $sign $bits,
            "Returns the number of ones in the binary representation of `self`.",

            "let n = " doc::type_str!($sign $bits) "::from(0b010111101010000u16);\n"
            "assert_eq!(n.count_ones(), 7);"
        }
    };
}

pub(crate) use count_ones;

macro_rules! count_zeros {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #count_zeros,
            $sign $bits,
            "Returns the number of zeros in the binary representation of `self`.",

            "assert_eq!((!" doc::type_str!($sign $bits) "::ZERO).count_zeros(), 0);"
        }
    };
}

pub(crate) use count_zeros;

macro_rules! leading_zeros {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #leading_zeros,
            $sign $bits,
            "Returns the number of leading zeros in the binary representation of `self`.",

            "let n = " doc::type_str!($sign $bits) "::ZERO;\n"
            "assert_eq!(n.leading_zeros(), " $bits ");"
        }
    };
}

pub(crate) use leading_zeros;

macro_rules! trailing_zeros {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #trailing_zeros,
            $sign $bits,
            "Returns the number of trailing zeros in the binary representation of `self`.",

            "let n = (!" doc::type_str!($sign $bits) "::ZERO) << 6u32;\n"
            "assert_eq!(n.trailing_zeros(), 6);"
        }
    };
}

pub(crate) use trailing_zeros;

macro_rules! leading_ones {
    ($sign: ident $bits: literal, $c: ident) => {
        doc::doc_comment! {
            #leading_ones,
            $sign $bits,
            "Returns the number of leading ones in the binary representation of `self`.",

            "let n = !" doc::type_str!($sign $bits) "::ZERO;\n"
            "assert_eq!(n.leading_ones(), " $bits ");"
        }
    };
}

pub(crate) use leading_ones;

macro_rules! trailing_ones {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #trailing_ones,
            $sign $bits,
            "Returns the number of trailing ones in the binary representation of `self`.",

            "let n = " doc::type_str!($sign $bits) "::from(0b1000101011011u16);\n"
            "assert_eq!(n.trailing_ones(), 2);"
        }
    };
}

pub(crate) use trailing_ones;

macro_rules! rotate_left {
    ($sign: ident $bits: literal, $u: literal) => {
        doc::doc_comment! {
            #rotate_left,
            $sign $bits,
            "Shifts the bits to the left by a specified amount, `n`, wrapping the truncated bits to the end of the resulting integer."
            "Please note this isn't the same operation as the `<<` shifting operator!"
        }
    }
}

pub(crate) use rotate_left;

macro_rules! rotate_right {
    ($sign: ident $bits: literal, $u: literal) => {
        doc::doc_comment! {
            #rotate_right,
            $sign $bits,
            "Shifts the bits to the left by a specified amount, `n`, wrapping the truncated bits to the end of the resulting integer."
            "Please note this isn't the same operation as the `>>` shifting operator!"
            "`self.rotate_right(n)` is equivalent to `self.rotate_left(Self::BITS - n)`."
        }
    }
}

pub(crate) use rotate_right;

macro_rules! swap_bytes {
    ($sign: ident $bits: literal, $u: literal) => {
        doc::doc_comment! {
            #swap_bytes,
            $sign $bits,
            "Reverses the byte order of the integer.",

            "let n = " doc::type_str!($sign $bits) "::from(0x12345678901234567890123456789012" $u "128);\n"
            "assert_eq!(n.swap_bytes().swap_bytes(), n);"
        }
    }
}

pub(crate) use swap_bytes;

macro_rules! reverse_bits {
    ($sign: ident $bits: literal, $u: literal) => {
        doc::doc_comment! {
            #reverse_bits,
            $sign $bits,
            "Reverses the order of bits in the integer. The least significant bit becomes the most significant bit, second least-significant bit becomes second most-significant bit, etc.",

            "let n = " doc::type_str!($sign $bits) "::from(0x12345678901234567890123456789012" $u "128);\n"
            "assert_eq!(n.reverse_bits().reverse_bits(), n);"
        }
    };
}

pub(crate) use reverse_bits;

macro_rules! pow {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #pow,
            $sign $bits,
            "Raises `self` to the power of `exp`, using exponentiation by squaring.",

            "let n = " doc::type_str!($sign $bits) "::from(3u8);\n"
            "assert_eq!(n.pow(5), (243u8).into());"
        }
    };
}

pub(crate) use pow;

macro_rules! next_power_of_two {
    ($sign: ident $bits: literal, $wrap: literal, $small: literal) => {
        doc::doc_comment! {
            #next_power_of_two,
            $sign $bits,
            concat!("When return value overflows, it panics in debug mode and the return value is wrapped to ", $wrap, " in release mode (the only situation in which method can return ", $wrap, ")."),

            "let n = " doc::type_str!($sign $bits) "::from(2u8);\n"
            "assert_eq!(n.next_power_of_two(), n);\n"
            "assert_eq!(" doc::type_str!($sign $bits)
            "::from(3u8).next_power_of_two(), 4u8.into());\n"
            "assert_eq!(" doc::type_str!($sign $bits) "::" $small ".next_power_of_two(), " doc::type_str!($sign $bits) "::ONE);"
        }
    }
}

pub(crate) use next_power_of_two;

macro_rules! default {
    () => {
        "Returns the default value of `Self::ZERO`."
    };
}

pub(crate) use default;

macro_rules! bits {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "Returns the smallest number of bits necessary to represent `self`."
            "This is equal to the size of the type in bits minus the leading zeros of `self`.",

            "assert_eq!(" doc::type_str!($sign $bits) "::from(0b1111001010100u16).bits(), 13);\n"
            "assert_eq!(" doc::type_str!($sign $bits) "::ZERO.bits(), 0);"
        }
    };
}

pub(crate) use bits;

macro_rules! bit {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "Returns a boolean representing the bit in the given position (`true` if the bit is set). The least significant bit is at index `0`, the most significant bit is at index `Self::BITS - 1`",

            "let n = " doc::type_str!($sign $bits) "::from(0b001010100101010101u32);\n"
            "assert!(n.bit(0));\n"
            "assert!(!n.bit(1));\n"
            "assert!(!n.bit(" doc::type_str!($sign $bits) "::BITS - 1));"
        }
    };
}

pub(crate) use bit;

macro_rules! is_zero {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "Returns whether `self` is zero.",

            "assert!(" doc::type_str!($sign $bits) "::ZERO.is_zero());\n"
            "assert!(!" doc::type_str!($sign $bits) "::ONE.is_zero());"
        }
    };
}

pub(crate) use is_zero;

macro_rules! is_one {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "Returns whether `self` is one.",

            "assert!(" doc::type_str!($sign $bits) "::ONE.is_one());\n"
            "assert!(!" doc::type_str!($sign $bits) "::MAX.is_one());"
        }
    };
}

pub(crate) use is_one;

crate::doc::link_doc_comment! {
    unsigned_abs,
    div_euclid,
    rem_euclid,
    ilog2,
    ilog10,
    ilog,
    abs_diff,
    next_multiple_of,
    div_floor,
    div_ceil,
    abs,
    signum,
    is_positive,
    is_negative
}
