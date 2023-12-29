#[cfg(test)]
macro_rules! tests {
    ($($int: ty), *) => {
        use crate::test::types::*;
        use crate::test;
        #[allow(unused_imports)]
        use crate::test::types::*;
        use crate::cast::{CastTo, CastFrom};

        use crate::test::cast_types::*;

        $(
            test::test_from! {
                function: <$int as CastFrom>::cast_from,
                from_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64, bool, char, UTESTD8, UTESTD16, UTESTD32, UTESTD64, TestUint1, TestUint2, TestUint3, TestUint4, TestUint5, TestUint6, TestUint7, TestUint8, TestUint9, TestUint10, ITESTD8, ITESTD16, ITESTD32, ITESTD64, TestInt1, TestInt2, TestInt3, TestInt4, TestInt5, TestInt6, TestInt7, TestInt8, TestInt9, TestInt10)
            }

            test::test_into! {
                function: <$int as CastTo>::cast_to,
                into_types: (u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64)
            }

            crate::int::cast::test_cast_to_bigint!($int; UTESTD8, UTESTD16, UTESTD32, UTESTD64, TestUint1, TestUint2, TestUint3, TestUint4, TestUint5, TestUint6, TestUint7, TestUint8, ITESTD8, ITESTD16, ITESTD32, ITESTD64, TestInt1, TestInt2, TestInt3, TestInt4, TestInt5, TestInt6, TestInt7, TestInt8);
        )*
    };
}

#[cfg(test)]
pub(crate) use tests;

#[cfg(test)]
macro_rules! test_cast_to_bigint {
    ($primitive: ty; $($Int: ty), *) => {
        paste::paste! {
            quickcheck::quickcheck! {
                $(
                    #[allow(non_snake_case)]
                    fn [<quickcheck_ $primitive _CastTo_ $Int _cast_to>](a: $primitive) -> bool {
                        // use crate::test::types;
                        use crate::cast::CastTo;

                        let primitive = <$primitive as CastTo<$Int>>::cast_to(a);
                        let big = <[<$primitive:upper>] as CastTo<$Int>>::cast_to(Into::into(a));

                        primitive == big
                    }
                )*
            }
        }
    };
}

#[cfg(test)]
pub(crate) use test_cast_to_bigint;