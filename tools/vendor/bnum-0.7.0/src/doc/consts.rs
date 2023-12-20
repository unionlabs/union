macro_rules! impl_desc {
    () => {
        "Associated constants for this type."
    };
}

pub(crate) use impl_desc;

macro_rules! min {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "The minimum value that this type can represent.",

            "assert_eq!(!" doc::type_str!($sign $bits) "::MIN, " doc::type_str!($sign $bits) "::MAX);"
        }
    };
}

pub(crate) use min;

macro_rules! max {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "The maximum value that this type can represent.",

            "assert_eq!(" doc::type_str!($sign $bits) "::MAX.wrapping_add(" doc::type_str!($sign $bits) "::ONE), " doc::type_str!($sign $bits) "::MIN);"
        }
    };
}

pub(crate) use max;

macro_rules! zero {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            doc::consts::value_desc!(0),

            "assert_eq!(" doc::type_str!($sign $bits) "::ZERO, " doc::type_str!($sign $bits) "::from(0u8));"
        }
    }
}

pub(crate) use zero;

macro_rules! one {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            doc::consts::value_desc!(1),

            "assert_eq!(" doc::type_str!($sign $bits) "::ONE, " doc::type_str!($sign $bits) "::from(1u8));"
        }
    }
}

pub(crate) use one;

macro_rules! bits {
    ($sign: ident $bits: literal, $digit_bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "The total number of bits that this type contains.",

            "assert_eq!(" doc::type_str!($sign $bits) "::BITS, " $digit_bits ");"
        }
    };
}

pub(crate) use bits;

macro_rules! bytes {
    ($sign: ident $bits: literal, $digit_bits: literal) => {
        doc::doc_comment! {
            $sign $bits,
            "The total number of bytes that this type contains.",

            "assert_eq!(" doc::type_str!($sign $bits) "::BYTES, " $digit_bits " / 8);"
        }
    };
}

pub(crate) use bytes;

macro_rules! value_desc {
    ($($lit: literal) +) => {
        concat!("The value of `", $($lit,)+ "` represented by this type.")
    }
}

pub(crate) use value_desc;
