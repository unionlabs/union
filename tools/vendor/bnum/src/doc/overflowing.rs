macro_rules! impl_desc {
    () => {
        doc::arithmetic_impl_desc!("Overflowing", "overflowing", "Each method returns a tuple of type `(Self, bool)` where the first item of the tuple is the result of the calculation truncated to the number of bits of `self`, and the second item is a boolean which indicates whether overflow occurred (i.e. if the number of bits of the result of the calculation exceeded the number of bits of `self`).")
    };
}

pub(crate) use impl_desc;

crate::doc::link_doc_comment!(
    overflowing_abs,
    overflowing_add,
    overflowing_add_signed,
    overflowing_add_unsigned,
    overflowing_div,
    overflowing_div_euclid,
    overflowing_mul,
    overflowing_neg,
    overflowing_pow,
    overflowing_rem,
    overflowing_rem_euclid,
    overflowing_shl,
    overflowing_shr,
    overflowing_sub,
    overflowing_sub_unsigned
);
