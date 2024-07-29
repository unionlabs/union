use std::fmt::Debug;

use ssz::{
    types::{typenum::U5, List},
    Ssz,
};

fn assert_encode<T: Ssz>(item: &T, bytes: &[u8]) {
    assert_eq!(item.as_ssz_bytes(), bytes);
}

fn assert_encode_decode<T: Ssz + PartialEq + Debug>(item: &T, bytes: &[u8]) {
    assert_encode(item, bytes);
    assert_eq!(T::from_ssz_bytes(bytes).unwrap(), *item);
}

#[derive(PartialEq, Debug, Ssz)]
#[ssz(union)]
enum TwoFixedUnion {
    U8(u8),
    U16(u16),
}

#[derive(PartialEq, Debug, Ssz)]
struct TwoFixedUnionStruct {
    a: TwoFixedUnion,
}

#[test]
fn two_fixed_union() {
    let eight = TwoFixedUnion::U8(1);
    let sixteen = TwoFixedUnion::U16(1);

    assert_encode_decode(&eight, &[0, 1]);
    assert_encode_decode(&sixteen, &[1, 1, 0]);

    assert_encode_decode(&TwoFixedUnionStruct { a: eight }, &[4, 0, 0, 0, 0, 1]);
    assert_encode_decode(&TwoFixedUnionStruct { a: sixteen }, &[4, 0, 0, 0, 1, 1, 0]);
}

#[derive(PartialEq, Debug, Ssz)]
struct VariableA {
    a: u8,
    b: List<u8, U5>,
}

#[derive(PartialEq, Debug, Ssz)]
struct VariableB {
    a: List<u8, U5>,
    b: u8,
}

#[derive(PartialEq, Debug, Ssz)]
#[ssz(union)]
enum TwoVariableUnion {
    A(VariableA),
    B(VariableB),
}

#[derive(PartialEq, Debug, Ssz)]
struct TwoVariableUnionStruct {
    a: TwoVariableUnion,
}

#[test]
fn two_variable_union() {
    let union_a = TwoVariableUnion::A(VariableA {
        a: 1,
        b: vec![2, 3].try_into().unwrap(),
    });
    let union_b = TwoVariableUnion::B(VariableB {
        a: vec![1, 2].try_into().unwrap(),
        b: 3,
    });

    assert_encode_decode(&union_a, &[0, 1, 5, 0, 0, 0, 2, 3]);
    assert_encode_decode(&union_b, &[1, 5, 0, 0, 0, 3, 1, 2]);

    assert_encode_decode(
        &TwoVariableUnionStruct { a: union_a },
        &[4, 0, 0, 0, 0, 1, 5, 0, 0, 0, 2, 3],
    );
    assert_encode_decode(
        &TwoVariableUnionStruct { a: union_b },
        &[4, 0, 0, 0, 1, 5, 0, 0, 0, 3, 1, 2],
    );
}

#[derive(PartialEq, Debug, Ssz)]
#[ssz(union)]
enum TwoListUnion {
    A(List<u8, U5>),
    B(List<u8, U5>),
}

#[test]
fn two_vec_union() {
    assert_encode_decode(&TwoListUnion::A(vec![0].try_into().unwrap()), &[0, 0]);
    assert_encode_decode(&TwoListUnion::B(vec![0].try_into().unwrap()), &[1, 0]);

    assert_encode_decode(&TwoListUnion::A(vec![0, 1].try_into().unwrap()), &[0, 0, 1]);
    assert_encode_decode(&TwoListUnion::B(vec![0, 1].try_into().unwrap()), &[1, 0, 1]);
}

#[derive(PartialEq, Debug, Ssz)]
#[ssz(transparent)]
struct TransparentStruct {
    inner: List<u8, U5>,
}

impl TransparentStruct {
    fn new(inner: u8) -> Self {
        Self {
            inner: vec![inner].try_into().unwrap(),
        }
    }
}

#[test]
fn transparent_struct() {
    assert_encode_decode(
        &TransparentStruct::new(42),
        &List::<u8, U5>::try_from(vec![42_u8])
            .unwrap()
            .as_ssz_bytes(),
    );
}

#[derive(PartialEq, Debug, Ssz)]
#[ssz(transparent)]
struct TransparentStructNewType(List<u8, U5>);

#[test]
fn transparent_struct_newtype() {
    assert_encode_decode(
        &TransparentStructNewType(vec![42_u8].try_into().unwrap()),
        &List::<u8, U5>::try_from(vec![42_u8])
            .unwrap()
            .as_ssz_bytes(),
    );
}
