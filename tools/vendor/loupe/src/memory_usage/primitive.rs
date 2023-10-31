#[cfg(test)]
use crate::{assert_size_of_val_eq, POINTER_BYTE_SIZE};
use crate::{MemoryUsage, MemoryUsageTracker};
use std::mem;

macro_rules! impl_memory_usage_for_numeric {
    ( $type:ty ) => {
        impl MemoryUsage for $type {
            fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
                mem::size_of_val(self)
            }
        }
    };

    ( $( $type:ty ),+ $(,)* ) => {
        $( impl_memory_usage_for_numeric!( $type ); )+
    }
}

impl_memory_usage_for_numeric!(
    bool, char, f32, f64, i8, i16, i32, i64, isize, u8, u16, u32, u64, usize
);

#[cfg(test)]
mod test_numeric_types {
    use super::*;

    macro_rules! test_memory_usage_for_numeric {
        ($test_name:ident: ($value:expr) == $expected:expr) => {
            #[test]
            fn $test_name() {
                assert_size_of_val_eq!($value, $expected);
            }
        };

        ( $( $test_name:ident: ($value:expr) == $expected:expr );+ $(;)* ) => {
            $( test_memory_usage_for_numeric!( $test_name: ($value) == $expected); )+
        }
    }

    test_memory_usage_for_numeric!(
        test_bool: (true) == 1;
        test_char: ('a') == 4;
        test_f32: (4.2f32) == 4;
        test_f64: (4.2f64) == 8;
        test_i8: (1i8) == 1;
        test_i16: (1i16) == 2;
        test_i32: (1i32) == 4;
        test_i64: (1i64) == 8;
        test_isize: (1isize) == POINTER_BYTE_SIZE;
        test_u8: (1u8) == 1;
        test_u16: (1u16) == 2;
        test_u32: (1u32) == 4;
        test_u64: (1u64) == 8;
        test_usize: (1usize) == POINTER_BYTE_SIZE;
    );
}

#[rustversion::any(stable(1.51), since(2021-02-01))]
impl<T, const N: usize> MemoryUsage for [T; N]
where
    T: MemoryUsage,
{
    fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
        mem::size_of_val(self)
            + self
                .iter()
                .map(|value| value.size_of_val(tracker) - mem::size_of_val(value))
                .sum::<usize>()
    }
}

#[rustversion::any(stable(1.51), since(2021-02-01))]
#[cfg(test)]
mod test_array_types {
    use super::*;

    #[test]
    fn test_array() {
        let array: [i16; 0] = [0; 0];
        assert_size_of_val_eq!(array, 2 * 0);

        let array: [i16; 1] = [0; 1];
        assert_size_of_val_eq!(array, 2 * 1);

        let array: [i16; 2] = [0; 2];
        assert_size_of_val_eq!(array, 2 * 2);

        let array: [i16; 3] = [0; 3];
        assert_size_of_val_eq!(array, 2 * 3);

        let array: [[i16; 3]; 5] = [[0; 3]; 5];
        assert_size_of_val_eq!(array, 2 * 3 * 5);
    }
}

impl MemoryUsage for () {
    fn size_of_val(&self, _: &mut dyn MemoryUsageTracker) -> usize {
        0
    }
}

macro_rules! impl_memory_usage_for_tuple {
    ( $first_type:ident $(,)* ) => {};

    ( $first_type:ident $( , $types:ident )+ $(,)* ) => {
        impl< $first_type $( , $types )+ > MemoryUsage for ( $first_type $( , $types )+ )
        where
            $first_type: MemoryUsage,
            $( $types: MemoryUsage ),*
        {
            fn size_of_val(&self, tracker: &mut dyn MemoryUsageTracker) -> usize {
                #[allow(non_snake_case)]
                let ( $first_type $( , $types )+ ) = self;

                mem::size_of_val(self)
                    + $first_type.size_of_val(tracker) - mem::size_of_val($first_type)
                    $( + $types.size_of_val(tracker) - mem::size_of_val($types) )+
            }
        }

        impl_memory_usage_for_tuple!( $( $types ),+ );
    };
}

impl_memory_usage_for_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

#[cfg(test)]
mod test_tuple_types {
    use super::*;

    #[test]
    fn test_empty_tuple() {
        let empty = ();
        assert_size_of_val_eq!(empty, 0);
    }

    #[test]
    fn test_tuple() {
        let tuple: (i8, i8) = (1, 2);
        assert_size_of_val_eq!(tuple, 1 /* i8 */ + 1 /* i8 */);

        let tuple: (i8, i16) = (1, 2);
        assert_size_of_val_eq!(tuple, 1 /* i8 */ + 2 /* i16 */ + 1 /* padding */);

        let tuple: (i8, i16, i32) = (1, 2, 3);
        assert_size_of_val_eq!(
            tuple,
            1 /* i8 */ + 2 /* i16 */ + 4 /* i32 */ + 1, /* padding */
        );

        let tuple: (i32, i32) = (1, 2);
        assert_size_of_val_eq!(tuple, 4 /* i32 */ + 4 /* i32 */);

        let tuple: (&str, &str) = ("", "");
        assert_size_of_val_eq!(
            tuple,
            2 * POINTER_BYTE_SIZE + 1 * 0 /* str */ + 2 * POINTER_BYTE_SIZE + 1 * 0, /* str */
        );

        let tuple: (&str, &str) = ("a", "bc");
        assert_size_of_val_eq!(
            tuple,
            2 * POINTER_BYTE_SIZE + 1 * 1 /* str */ + 2 * POINTER_BYTE_SIZE + 1 * 2, /* str */
        );

        let tuple: (&str, (i64, i64, i8)) = ("abc", (1, 2, 3));
        assert_size_of_val_eq!(
            tuple,
            2 * POINTER_BYTE_SIZE + 1 * 3 /* str */ + 8 /* i64 */ + 8 /* i64 */ + 1 /* i8 */ + 7, /* padding */
        );
    }
}
