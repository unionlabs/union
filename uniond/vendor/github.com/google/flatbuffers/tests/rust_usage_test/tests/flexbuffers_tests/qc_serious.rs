use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use super::rwyw::NonNullString;
use flexbuffers::*;
use quickcheck::{Arbitrary, Gen};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
enum Enum {
    Unit,
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    Us(u8, u16, u32, u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Is(i8, i16, i32, i64),
    F32(f32),
    F64(f64),
    Fs(f32, f64),
    String(String),
    Strings(String, String),
    Everything(u8, u16, u32, u64, i8, i16, i32, i64, f32, f64, String),
    Arrays {
        a: Array3<u16>,
        b: Array4<i32>,
        c: Array2<f64>,
    },
    Blobs(#[serde(with = "serde_bytes")] Vec<u8>),
}

// There is some upstream bug in deriving Arbitrary for Enum so we manually implement it here.
impl Arbitrary for Enum {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        match g.gen_range(0, 18) {
            0 => Enum::Unit,
            1 => Enum::U8(<u8>::arbitrary(g)),
            2 => Enum::U16(<u16>::arbitrary(g)),
            3 => Enum::U32(<u32>::arbitrary(g)),
            4 => Enum::U64(<u64>::arbitrary(g)),
            5 => {
                let (a, b, c, d) = <(u8, u16, u32, u64)>::arbitrary(g);
                Enum::Us(a, b, c, d)
            }
            6 => Enum::I8(<i8>::arbitrary(g)),
            7 => Enum::I16(<i16>::arbitrary(g)),
            8 => Enum::I32(<i32>::arbitrary(g)),
            9 => Enum::I64(<i64>::arbitrary(g)),
            10 => {
                let (a, b, c, d) = <(i8, i16, i32, i64)>::arbitrary(g);
                Enum::Is(a, b, c, d)
            }
            11 => Enum::F32(<f32>::arbitrary(g)),
            12 => Enum::F64(<f64>::arbitrary(g)),
            13 => {
                let (a, b) = <(f32, f64)>::arbitrary(g);
                Enum::Fs(a, b)
            }
            14 => Enum::String(String::arbitrary(g)),
            15 => {
                let (a, b) = <(String, String)>::arbitrary(g);
                Enum::Strings(a, b)
            }
            16 => Enum::Everything(
                <u8>::arbitrary(g),
                <u16>::arbitrary(g),
                <u32>::arbitrary(g),
                <u64>::arbitrary(g),
                <i8>::arbitrary(g),
                <i16>::arbitrary(g),
                <i32>::arbitrary(g),
                <i64>::arbitrary(g),
                <f32>::arbitrary(g),
                <f64>::arbitrary(g),
                <String>::arbitrary(g),
            ),
            17 => {
                let a = Array3::arbitrary(g);
                let b = Array4::arbitrary(g);
                let c = Array2::arbitrary(g);
                Enum::Arrays { a, b, c }
            }
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Arbitrary, PartialEq, Serialize, Deserialize)]
struct Unit;

#[derive(Debug, Clone, Arbitrary, PartialEq, Serialize, Deserialize)]
struct NewType(bool);

#[derive(Debug, Clone, Arbitrary, PartialEq, Serialize, Deserialize)]
struct Tuple(bool, u8, i16, f32, String);

#[derive(Debug, Clone, Arbitrary, PartialEq, Serialize, Deserialize)]
struct Struct {
    a: Vec<Enum>,
    b: BTreeMap<NonNullString, Enum>,
    c: Tuple,
    d: (Unit, Unit),
    e: Array4<NewType>,
}

#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
struct Array2<A: Arbitrary>([A; 2]);
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
struct Array3<A: Arbitrary>([A; 3]);
#[derive(Debug, Clone, PartialOrd, Ord, PartialEq, Eq, Serialize, Deserialize)]
struct Array4<A: Arbitrary>([A; 4]);

impl<A: Arbitrary> Arbitrary for Array2<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Array2([A::arbitrary(g), A::arbitrary(g)])
    }
}
impl<A: Arbitrary> Arbitrary for Array3<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Array3([A::arbitrary(g), A::arbitrary(g), A::arbitrary(g)])
    }
}
impl<A: Arbitrary> Arbitrary for Array4<A> {
    fn arbitrary<G: Gen>(g: &mut G) -> Self {
        Array4([
            A::arbitrary(g),
            A::arbitrary(g),
            A::arbitrary(g),
            A::arbitrary(g),
        ])
    }
}

quickcheck! {
    fn qc_serious(original: Struct) -> bool {
        let struct_buf = flexbuffers::to_vec(&original).unwrap();
        let root = Reader::get_root(&*struct_buf).unwrap();
        let reader_buf = flexbuffers::to_vec(&root).unwrap();
        let deserialized: Struct = flexbuffers::from_slice(&struct_buf).unwrap();
        let reserialized: Struct = flexbuffers::from_slice(&reader_buf).unwrap();

        original == deserialized && original == reserialized
    }
}
