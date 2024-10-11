#![no_std]

#[cfg(not(feature = "no_std"))]
extern crate std;

extern crate alloc;

extern crate array_init;

#[allow(dead_code, unused_imports)]
#[path = "../../arrays_test/mod.rs"]
mod arrays_test_generated;

use alloc::format;
use core::fmt::Debug;

use crate::arrays_test_generated::my_game::example::*;

extern crate quickcheck;

use array_init::array_init;
use core::mem::size_of;
use quickcheck::{Arbitrary, Gen};

fn create_serialized_example_with_generated_code(builder: &mut flatbuffers::FlatBufferBuilder) {
    let nested_struct1 = NestedStruct::new(
        &[-1, 2],
        TestEnum::A,
        &[TestEnum::C, TestEnum::B],
        &[0x1122334455667788, -0x1122334455667788],
    );
    let nested_struct2 = NestedStruct::new(
        &[3, -4],
        TestEnum::B,
        &[TestEnum::B, TestEnum::A],
        &[-0x1122334455667788, 0x1122334455667788],
    );
    let array_struct = ArrayStruct::new(
        12.34,
        &[1, 2, 3, 4, 5, 6, 7, 8, 9, 0xA, 0xB, 0xC, 0xD, 0xE, 0xF],
        -127,
        &[nested_struct1, nested_struct2],
        1,
        &[-0x8000000000000000, 0x7FFFFFFFFFFFFFFF],
    );
    // Test five makes sense when specified.
    let ss = ArrayTable::create(
        builder,
        &ArrayTableArgs {
            a: Some(&array_struct),
        },
    );
    finish_array_table_buffer(builder, ss);
}

fn serialized_example_is_accessible_and_correct(
    bytes: &[u8],
    identifier_required: bool,
    size_prefixed: bool,
) {
    if identifier_required {
        let correct = if size_prefixed {
            array_table_size_prefixed_buffer_has_identifier(bytes)
        } else {
            array_table_buffer_has_identifier(bytes)
        };

        assert_eq!(correct, true);
    }

    let array_table = if size_prefixed {
        size_prefixed_root_as_array_table(bytes).unwrap()
    } else {
        root_as_array_table(bytes).unwrap()
    };

    let array_struct = array_table.a().unwrap();
    assert_eq!(array_struct.a(), 12.34);
    assert_eq!(array_struct.b().len(), 0xF);
    assert_eq!(array_struct.b().iter().sum::<i32>(), 120);
    assert_eq!(array_struct.c(), -127);

    assert_eq!(array_struct.d().len(), 2);
    let nested_struct1 = array_struct.d().get(0);
    assert_eq!(nested_struct1.a().len(), 2);
    assert_eq!(nested_struct1.a().iter().sum::<i32>(), 1);
    assert_eq!(nested_struct1.b(), TestEnum::A);
    assert_eq!(nested_struct1.c().len(), 2);
    assert_eq!(nested_struct1.c().get(0), TestEnum::C);
    assert_eq!(nested_struct1.c().get(1), TestEnum::B);
    assert_eq!(nested_struct1.d().len(), 2);
    assert_eq!(
        [nested_struct1.d().get(0), nested_struct1.d().get(1)],
        [0x1122334455667788, -0x1122334455667788]
    );
    let nested_struct2 = array_struct.d().get(1);
    assert_eq!(nested_struct2.a().len(), 2);
    assert_eq!(nested_struct2.a().iter().sum::<i32>(), -1);
    assert_eq!(nested_struct2.b(), TestEnum::B);
    assert_eq!(nested_struct2.c().len(), 2);
    assert_eq!(nested_struct2.c().get(0), TestEnum::B);
    assert_eq!(nested_struct2.c().get(1), TestEnum::A);
    assert_eq!(nested_struct2.d().len(), 2);
    let arr: [i64; 2] = nested_struct2.d().into();
    assert_eq!(
        arr,
        [-0x1122334455667788, 0x1122334455667788]
    );

    assert_eq!(array_struct.e(), 1);
    assert_eq!(array_struct.f().len(), 2);
    assert_eq!(array_struct.f().get(0), -0x8000000000000000);
    assert_eq!(array_struct.f().get(1), 0x7FFFFFFFFFFFFFFF);
}

#[test]
fn generated_code_creates_correct_example() {
    let mut b = flatbuffers::FlatBufferBuilder::new();
    create_serialized_example_with_generated_code(&mut b);
    let buf = b.finished_data();
    serialized_example_is_accessible_and_correct(&buf[..], true, false);
}

#[test]
fn struct_netsted_struct_is_32_bytes() {
    assert_eq!(32, ::core::mem::size_of::<NestedStruct>());
}

#[test]
fn struct_array_struct_is_160_bytes() {
    assert_eq!(160, ::core::mem::size_of::<ArrayStruct>());
}

#[test]
fn test_object_api_reads_correctly() {
    let mut b = flatbuffers::FlatBufferBuilder::new();
    create_serialized_example_with_generated_code(&mut b);

    let array_table = root_as_array_table(b.finished_data()).unwrap().unpack();

    let array_struct = array_table.a.unwrap();
    assert_eq!(array_struct.a, 12.34);
    assert_eq!(array_struct.b.len(), 0xF);
    assert_eq!(array_struct.b.iter().sum::<i32>(), 120);
    assert_eq!(array_struct.c, -127);

    assert_eq!(array_struct.d.len(), 2);
    let nested_struct1 = &array_struct.d[0];
    assert_eq!(nested_struct1.a.len(), 2);
    assert_eq!(nested_struct1.a.iter().sum::<i32>(), 1);
    assert_eq!(nested_struct1.b, TestEnum::A);
    assert_eq!(nested_struct1.c.len(), 2);
    assert_eq!(nested_struct1.c[0], TestEnum::C);
    assert_eq!(nested_struct1.c[1], TestEnum::B);
    assert_eq!(nested_struct1.d.len(), 2);
    assert_eq!(nested_struct1.d, [0x1122334455667788, -0x1122334455667788]);
    let nested_struct2 = &array_struct.d[1];
    assert_eq!(nested_struct2.a.len(), 2);
    assert_eq!(nested_struct2.a.iter().sum::<i32>(), -1);
    assert_eq!(nested_struct2.b, TestEnum::B);
    assert_eq!(nested_struct2.c.len(), 2);
    assert_eq!(nested_struct2.c[0], TestEnum::B);
    assert_eq!(nested_struct2.c[1], TestEnum::A);
    assert_eq!(nested_struct2.d.len(), 2);
    assert_eq!(nested_struct2.d, [-0x1122334455667788, 0x1122334455667788]);

    assert_eq!(array_struct.e, 1);
    assert_eq!(array_struct.f.len(), 2);
    assert_eq!(array_struct.f[0], -0x8000000000000000);
    assert_eq!(array_struct.f[1], 0x7FFFFFFFFFFFFFFF);
}

#[test]
fn object_api_defaults() {
    use arrays_test_generated::my_game::example::*;

    assert_eq!(
        NestedStructT::default(),
        NestedStructT {
            a: [0, 0],
            b: TestEnum::default(),
            c: [TestEnum::default(), TestEnum::default()],
            d: [0, 0],
        }
    );

    assert_eq!(
        ArrayStructT::default(),
        ArrayStructT {
            a: 0.0,
            b: [0; 0xF],
            c: 0,
            d: [NestedStructT::default(), NestedStructT::default()],
            e: 0,
            f: [0, 0],
        }
    );
}

#[test]
fn generated_code_debug_prints_correctly() {
    let b = &mut flatbuffers::FlatBufferBuilder::new();
    create_serialized_example_with_generated_code(b);
    let buf = b.finished_data();
    let array_table = root_as_array_table(buf).unwrap();
    assert_eq!(
        format!("{:.5?}", &array_table),
        "ArrayTable { a: Some(ArrayStruct { a: 12.34000, \
         b: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15], \
         c: -127, d: [NestedStruct { a: [-1, 2], b: A, c: [C, B], \
         d: [1234605616436508552, -1234605616436508552] }, \
         NestedStruct { a: [3, -4], b: B, c: [B, A], d: [-1234605616436508552, 1234605616436508552] }], \
         e: 1, f: [-9223372036854775808, 9223372036854775807] }) }"
    );
}

#[test]
#[should_panic]
fn assert_on_too_small_array_buf() {
    let a = [0u8; 19];
    unsafe { flatbuffers::Array::<i32, 5>::new(&a) };
}

#[test]
#[should_panic]
fn assert_on_too_big_array_buf() {
    let a = [0u8; 21];
    unsafe { flatbuffers::Array::<i32, 5>::new(&a) };
}

#[test]
#[cfg(target_endian = "little")]
fn verify_struct_array_alignment() {
    let mut b = flatbuffers::FlatBufferBuilder::new();
    create_serialized_example_with_generated_code(&mut b);
    let buf = b.finished_data();
    let array_table = root_as_array_table(buf).unwrap();
    let array_struct = array_table.a().unwrap();
    let struct_start_ptr = array_struct.0.as_ptr() as usize;
    let b_start_ptr = array_struct.b().as_ptr() as usize;
    let d_start_ptr = array_struct.d().as_ptr() as usize;
    // The T type of b
    let b_aln = ::core::mem::align_of::<i32>();
    assert_eq!((b_start_ptr - struct_start_ptr) % b_aln, 0);
    assert_eq!((d_start_ptr - b_start_ptr) % b_aln, 0);
    assert_eq!((d_start_ptr - struct_start_ptr) % 8, 0);
}

#[derive(Clone, Debug)]
struct FakeArray<T, const N: usize>([T; N]);

impl<T: Arbitrary + Debug + PartialEq, const N: usize> Arbitrary for FakeArray<T, N> {
    fn arbitrary<G: Gen>(g: &mut G) -> FakeArray<T, N> {
        let x: [T; N] = array_init(|_| {
            loop {
                let generated_scalar = T::arbitrary(g);
                // Verify that generated scalar is not Nan, which is not equals to itself, 
                // therefore we can't use it to validate input == output
                if generated_scalar == generated_scalar { return generated_scalar; }
            }
        });
        FakeArray { 0: x }
    }
}

#[cfg(test)]
mod array_fuzz {
    #[cfg(not(miri))]  // slow.
    extern crate quickcheck;
    extern crate flatbuffers;

    use self::flatbuffers::{Follow, Push};
    use super::*;

    const MAX_TESTS: u64 = 20;
    const ARRAY_SIZE: usize = 29;

    // This uses a macro because lifetimes for the trait-bounded function get too
    // complicated.
    macro_rules! impl_prop {
        ($test_name:ident, $fn_name:ident, $ty:ident) => (
            fn $fn_name(xs: FakeArray<$ty, ARRAY_SIZE>) {
                let mut test_buf = [0 as u8; 1024];
                let arr: flatbuffers::Array<$ty, ARRAY_SIZE> =  unsafe {
                    flatbuffers::emplace_scalar_array(&mut test_buf, 0, &xs.0);
                    flatbuffers::Array::follow(&test_buf, 0)
                };
                let got: [$ty; ARRAY_SIZE] = arr.into();
                assert_eq!(got, xs.0);
            }
            #[test]
            fn $test_name() { 
                quickcheck::QuickCheck::new().max_tests(MAX_TESTS).quickcheck($fn_name as fn(FakeArray<$ty, ARRAY_SIZE>));
            }
        )
    }

    impl_prop!(test_bool, prop_bool, bool);
    impl_prop!(test_u8, prop_u8, u8);
    impl_prop!(test_i8, prop_i8, i8);
    impl_prop!(test_u16, prop_u16, u16);
    impl_prop!(test_u32, prop_u32, u32);
    impl_prop!(test_u64, prop_u64, u64);
    impl_prop!(test_i16, prop_i16, i16);
    impl_prop!(test_i32, prop_i32, i32);
    impl_prop!(test_i64, prop_i64, i64);
    impl_prop!(test_f32, prop_f32, f32);
    impl_prop!(test_f64, prop_f64, f64);

    const NESTED_STRUCT_SIZE: usize = size_of::<NestedStruct>();

    #[derive(Clone, Debug, PartialEq)]
    struct NestedStructWrapper(NestedStruct);

    impl Arbitrary for NestedStructWrapper {
        fn arbitrary<G: Gen>(g: &mut G) -> NestedStructWrapper {
            let mut x = NestedStruct::default();
            x.0 = FakeArray::<u8, NESTED_STRUCT_SIZE>::arbitrary(g).0;
            NestedStructWrapper { 0: x }
        }
    }

    fn prop_struct(xs: FakeArray<NestedStructWrapper, ARRAY_SIZE>) {
        let mut test_buf = [0 as u8; 1024];
        let native_struct_array: [&NestedStruct; ARRAY_SIZE] = array_init::from_iter(xs.0.iter().map(|x| &x.0)).unwrap();
        for i in 0..ARRAY_SIZE {
            let offset = i * NESTED_STRUCT_SIZE;
            unsafe { native_struct_array[i].push(&mut test_buf[offset..offset + NESTED_STRUCT_SIZE], 0) };
        }
        let arr: flatbuffers::Array<NestedStruct, ARRAY_SIZE> = unsafe { flatbuffers::Array::follow(&test_buf, 0) };
        let got: [&NestedStruct; ARRAY_SIZE] = arr.into();
        assert_eq!(got, native_struct_array);
    }

    #[test]
    #[cfg(not(miri))]  // slow.
    fn test_struct() {
        quickcheck::QuickCheck::new().max_tests(MAX_TESTS).quickcheck(prop_struct as fn(FakeArray<NestedStructWrapper, ARRAY_SIZE>));
    }
}
