#[cfg(test)]
macro_rules! tests {
    ($($int: ty), *) => {
            use crate::test::types::*;
            use crate::test;
            #[allow(unused_imports)]
            use crate::test::types::*;
            use crate::cast::{CastTo, CastFrom};

            $(
                test::test_from! {
                    function: <$int as CastFrom>::cast_from,
                    from_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char)
                }

                /*#[cfg(feature = "u8_digit")]
                test::test_into! {
                    function: <$int as CastTo>::cast_to,
                    into_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, U8, I8, U16, I16, U32, I32, U64, I64, U128, I128)
                }*/

                test::test_into! {
                    function: <$int as CastTo>::cast_to,
                    into_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64/* U64, I64, I128, U128*/)
                }
            )*
    };
}

#[cfg(test)]
pub(crate) use tests;
