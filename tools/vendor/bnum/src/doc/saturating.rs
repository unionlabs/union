macro_rules! impl_desc {
    () => {
        doc::arithmetic_impl_desc!("Saturating", "saturating", "For each method, if overflow or underflow occurs, the largest or smallest value that can be represented by `Self` is returned instead.")
    };
}

pub(crate) use impl_desc;

crate::doc::link_doc_comment!(
    saturating_abs,
    saturating_add,
    saturating_add_signed,
    saturating_add_unsigned,
    saturating_div,
    saturating_mul,
    saturating_neg,
    saturating_pow,
    saturating_sub,
    saturating_sub_unsigned
);
