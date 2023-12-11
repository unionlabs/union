#[cfg(test)]
macro_rules! format_trait {
    ($($method: ident), *) => {
        use alloc::string::String;

        // This trait allows us to use the default tester macro instead of creating a custom one
        pub trait Format {
            $(
                fn $method(&self) -> String;
            )*
        }
    };
}

#[cfg(test)]
format_trait!(binary, lower_hex, upper_hex, octal, display, debug, lower_exp, upper_exp);

#[cfg(test)]
macro_rules! impl_format_method {
    { $($name: ident : $format: literal), * } => {
        $(
            fn $name(&self) -> String {
                format!(concat!("{:", $format, "}"), self)
            }
        )*
    };
}

#[cfg(test)]
pub(crate) use impl_format_method;

#[cfg(test)]
macro_rules! impl_format {
    ($($ty: ty), *) => {
        $(
            impl Format for $ty {
                crate::int::fmt::impl_format_method! {
                    binary: "b",
                    lower_hex: "x",
                    upper_hex: "X",
                    octal: "o",
                    display: "",
                    debug: "?",
                    lower_exp: "e",
                    upper_exp: "E"
                }
            }
        )*
    };
}

#[cfg(test)]
pub(crate) use impl_format;

#[cfg(test)]
macro_rules! test_formats {
    ($ty: ty; $($name: ident), *) => {
        $(
            test_bignum! {
                function: <$ty as Format>::$name(a: ref &$ty)
            }
        )*
    };
}

#[cfg(test)]
pub(crate) use test_formats;

#[cfg(test)]
macro_rules! tests {
    ($ty: ty) => {
        use crate::int::fmt::{Format, self};
        use crate::test::{test_bignum, types::*};
        use alloc::string::String;

        paste::paste! {
            fmt::impl_format!([<$ty:upper>]);
        }

        fmt::test_formats!($ty; binary, lower_hex, upper_hex, octal, display, debug, lower_exp, upper_exp);
    };
}

#[cfg(test)]
pub(crate) use tests;

#[cfg(test)]
crate::int::fmt::impl_format!(u128, i128, u64, i64);
