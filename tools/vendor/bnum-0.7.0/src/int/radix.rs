macro_rules! assert_range {
    ($radix: expr, $max: expr) => {
        assert!(
            $radix >= 2 && $radix <= $max,
            crate::errors::err_msg!(concat!(
                "Radix must be in range [2, ",
                stringify!($max),
                "]"
            ))
        )
    };
}

pub(crate) use assert_range;
