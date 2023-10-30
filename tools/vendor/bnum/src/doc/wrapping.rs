macro_rules! impl_desc {
    () => {
        doc::arithmetic_impl_desc!("Wrapping", "wrapping", "Each method returns of the calculation truncated to the number of bits of `self`, i.e. they each return the first item in the tuple returned by their overflowing equivalent.")
    };
}

pub(crate) use impl_desc;

crate::doc::link_doc_comment!(
    wrapping_abs,
    wrapping_add,
    wrapping_add_signed,
    wrapping_add_unsigned,
    wrapping_div,
    wrapping_div_euclid,
    wrapping_mul,
    wrapping_neg,
    wrapping_pow,
    wrapping_rem,
    wrapping_rem_euclid,
    wrapping_shl,
    wrapping_shr,
    wrapping_sub,
    wrapping_sub_unsigned
);

macro_rules! wrapping_next_power_of_two {
    ($sign: ident $bits: literal) => {
        doc::doc_comment! {
            #wrapping_next_power_of_two,
            $sign $bits,
            concat!("Returns the smallest power of two greater than or equal to `self`. If the next power of two is greater than `Self::MAX`, the return value is wrapped to `Self::MIN`."),

            "let n = " doc::type_str!($sign $bits) "::from(31u8);\n"
            "assert_eq!(n.wrapping_next_power_of_two(), 32u8.into());\n"
            "assert_eq!(" doc::type_str!($sign $bits) "::MAX.wrapping_next_power_of_two(), " doc::type_str!($sign $bits) "::MIN);"
        }
    };
}

pub(crate) use wrapping_next_power_of_two;
