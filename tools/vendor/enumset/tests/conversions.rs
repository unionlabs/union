use enumset::*;

macro_rules! read_slice {
    ($set:expr, $size:expr) => {{
        let mut arr = [0; $size];
        $set.copy_into_slice(&mut arr);
        arr
    }};
}

macro_rules! try_read_slice {
    ($set:expr, $size:expr) => {{
        let mut arr = [0; $size];
        match $set.try_copy_into_slice(&mut arr) {
            Some(()) => Some(arr),
            None => None,
        }
    }};
}

macro_rules! read_slice_truncated {
    ($set:expr, $size:expr) => {{
        let mut arr = [0; $size];
        $set.copy_into_slice_truncated(&mut arr);
        arr
    }};
}

#[derive(EnumSetType, Debug)]
pub enum Enum8 {
    A, B, C, D, E, F, G,
    // H omitted for non-existent bit test
}

#[derive(EnumSetType, Debug)]
pub enum Enum16 {
    A, B, C, D, E=8, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum Enum32 {
    A, B, C, D, E=16, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum Enum64 {
    A, B, C, D, E=32, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum Enum128 {
    A, B, C, D, E=64, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum Enum192 {
    A, B, C, D, E=128, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum Enum256 {
    A, B, C, D, E=192, F, G, H,
}

macro_rules! check_simple_conversion {
    ($mod:ident, $e:ident) => {
        mod $mod {
            use super::*;

            #[test]
            fn to_integer() {
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_u8());
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_u16());
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_u32());
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_u64());
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_u128());
                assert_eq!(7,
                           ($e::A | $e::B | $e::C).as_usize());
            }

            #[test]
            fn try_from_integer() {
                assert_eq!(Some($e::A | $e::B | $e::C),
                           EnumSet::try_from_u8(7));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_u8(7 | (1 << 7)));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_u16(7 | (1 << 15)));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_u32(7 | (1 << 31)));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_usize(7 | (1 << 31)));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_u64(7 | (1 << 63)));
                assert_eq!(None,
                           EnumSet::<$e>::try_from_u128(7 | (1 << 127)));
            }

            #[test]
            fn from_integer_truncated() {
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u8_truncated(7));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u8_truncated(7 | (1 << 7)));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u16_truncated(7 | (1 << 15)));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u32_truncated(7 | (1 << 31)));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_usize_truncated(7 | (1 << 31)));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u64_truncated(7 | (1 << 63)));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::from_u128_truncated(7 | (1 << 127)));
            }

            #[test]
            fn basic_to_array() {
                // array tests
                assert_eq!(($e::A | $e::B | $e::C).as_array_truncated(), 
                           []);
                assert_eq!(EnumSet::<$e>::EMPTY.as_array_truncated(), 
                           []);
                assert_eq!(($e::A | $e::B | $e::C).as_array(), 
                           [7]);
                assert_eq!(($e::A | $e::B | $e::C).as_array(), 
                           [7, 0]);
                assert_eq!(($e::A | $e::B | $e::C).as_array(), 
                           [7, 0, 0]);
                assert_eq!(($e::A | $e::B | $e::C).as_array(), 
                           [7, 0, 0, 0]);
                assert_eq!(($e::A | $e::B | $e::C).as_array(),
                           [7, 0, 0, 0, 0]);
                
                // slice tests
                assert_eq!(read_slice!($e::A | $e::B | $e::C, 1), 
                           [7]);
                assert_eq!(read_slice!($e::A | $e::B | $e::C, 2), 
                           [7, 0]);
                assert_eq!(read_slice!($e::A | $e::B | $e::C, 3), 
                           [7, 0, 0]);
                assert_eq!(read_slice!($e::A | $e::B | $e::C, 4), 
                           [7, 0, 0, 0]);
                assert_eq!(read_slice!($e::A | $e::B | $e::C, 5),
                           [7, 0, 0, 0, 0]);

                // slice tests truncated
                assert_eq!(read_slice_truncated!($e::A | $e::B | $e::C, 1),
                           [7]);
                assert_eq!(read_slice_truncated!($e::A | $e::B | $e::C, 2),
                           [7, 0]);
                assert_eq!(read_slice_truncated!($e::A | $e::B | $e::C, 3),
                           [7, 0, 0]);
                assert_eq!(read_slice_truncated!($e::A | $e::B | $e::C, 4),
                           [7, 0, 0, 0]);
                assert_eq!(read_slice_truncated!($e::A | $e::B | $e::C, 5),
                           [7, 0, 0, 0, 0]);
            }

            #[test]
            fn basic_from_array() {
                // array tests
                assert_eq!(EnumSet::<$e>::EMPTY,
                           EnumSet::<$e>::from_array([]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array([7]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array([7, 0, 0]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array([7, 0, 0, 0]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array([7, 0, 0, 0, 0]));

                // array tests
                assert_eq!(EnumSet::<$e>::EMPTY,
                           EnumSet::<$e>::from_slice(&[]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice(&[7]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice(&[7, 0, 0]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice(&[7, 0, 0, 0]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice(&[7, 0, 0, 0, 0]));
            }

            #[test]
            fn basic_from_array_truncated() {
                // array tests
                assert_eq!(EnumSet::<$e>::EMPTY,
                           EnumSet::<$e>::from_array_truncated([]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array_truncated([7 | (1 << 31)]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array_truncated([7, 0, 16]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array_truncated([7, 0, 0, 16]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_array_truncated([7, 0, 0, 0, 16]));

                // array tests
                assert_eq!(EnumSet::<$e>::EMPTY,
                           EnumSet::<$e>::from_slice_truncated(&[]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice_truncated(&[7 | (1 << 31)]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice_truncated(&[7, 0, 16]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice_truncated(&[7, 0, 0, 16]));
                assert_eq!($e::A | $e::B | $e::C,
                           EnumSet::<$e>::from_slice_truncated(&[7, 0, 0, 0, 16]));
            }

            #[test]
            #[should_panic]
            fn fail_from_u8() {
                EnumSet::<$e>::from_u8(7 | (1 << 7));
            }

            #[test]
            #[should_panic]
            fn fail_from_u16() {
                EnumSet::<$e>::from_u16(7 | (1 << 15));
            }

            #[test]
            #[should_panic]
            fn fail_from_u32() {
                EnumSet::<$e>::from_u32(7 | (1 << 31));
            }

            #[test]
            #[should_panic]
            fn fail_from_usize() {
                EnumSet::<$e>::from_usize(7 | (1 << 31));
            }

            #[test]
            #[should_panic]
            fn fail_from_u64() {
                EnumSet::<$e>::from_u64(7 | (1 << 63));
            }

            #[test]
            #[should_panic]
            fn fail_from_u128() {
                EnumSet::<$e>::from_u128(7 | (1 << 127));
            }

            #[test]
            #[should_panic]
            fn fail_to_array() {
                ($e::A | $e::B | $e::C).as_array::<0>();
            }

            #[test]
            #[should_panic]
            fn fail_to_slice() {
                read_slice!($e::A | $e::B | $e::C, 0);
            }

            #[test]
            #[should_panic]
            fn fail_from_array_1() {
                EnumSet::<$e>::from_array([7 | (1 << 63), 0, 0, 0]);
            }

            #[test]
            #[should_panic]
            fn fail_from_slice_1() {
                EnumSet::<$e>::from_slice(&[7 | (1 << 63), 0, 0, 0]);
            }

            #[test]
            #[should_panic]
            fn fail_from_array_2() {
                EnumSet::<$e>::from_array([7, 0, 0, 0, 1]);
            }

            #[test]
            #[should_panic]
            fn fail_from_slice_2() {
                EnumSet::<$e>::from_slice(&[7, 0, 0, 0, 1]);
            }
        }
    };
}

check_simple_conversion!(enum_8_simple, Enum8);
check_simple_conversion!(enum_16_simple, Enum16);
check_simple_conversion!(enum_32_simple, Enum32);
check_simple_conversion!(enum_64_simple, Enum64);
check_simple_conversion!(enum_128_simple, Enum128);
check_simple_conversion!(enum_192_simple, Enum192);
check_simple_conversion!(enum_256_simple, Enum256);

macro_rules! check_oversized_64 {
    ($mod:ident, $e:ident) => {
        mod $mod {
            use super::*;

            #[test]
            fn downcast_to_u64() {
                assert_eq!(Some(7), ($e::A | $e::B | $e::C).try_as_u64());
                assert_eq!(Some([7]), ($e::A | $e::B | $e::C).try_as_array());
                assert_eq!(Some([7]), try_read_slice!($e::A | $e::B | $e::C, 1));

                assert_eq!(None, ($e::E | $e::F | $e::G).try_as_u64());
                assert_eq!(None, ($e::E | $e::F | $e::G).try_as_array::<1>());
                assert_eq!(None, try_read_slice!($e::E | $e::F | $e::G, 1));
            }

            #[test]
            fn downcast_to_u64_truncated() {
                assert_eq!(0, ($e::E | $e::F | $e::G).as_u64_truncated());
                assert_eq!([0], ($e::E | $e::F | $e::G).as_array_truncated());
                assert_eq!([0], read_slice_truncated!($e::E | $e::F | $e::G, 1));
            }

            #[test]
            #[should_panic]
            fn fail_to_u64() {
                ($e::E | $e::F | $e::G).as_u64();
            }
        }
    };
}

check_oversized_64!(enum_128_oversized_64, Enum128);
check_oversized_64!(enum_192_oversized_64, Enum192);
check_oversized_64!(enum_256_oversized_64, Enum256);

macro_rules! check_oversized_128 {
    ($mod:ident, $e:ident) => {
        mod $mod {
            use super::*;

            #[test]
            fn downcast_to_u128() {
                assert_eq!(Some(7), ($e::A | $e::B | $e::C).try_as_u128());
                assert_eq!(None, ($e::E | $e::F | $e::G).try_as_u128());
                assert_eq!(0, ($e::E | $e::F | $e::G).as_u128_truncated());
            }

            #[test]
            #[should_panic]
            fn fail_to_u128() {
                ($e::E | $e::F | $e::G).as_u128();
            }
        }
    };
}

check_oversized_128!(enum_192_oversized_128, Enum192);
check_oversized_128!(enum_256_oversized_128, Enum256);
