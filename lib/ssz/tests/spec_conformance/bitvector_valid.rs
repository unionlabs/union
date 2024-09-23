#![allow(non_snake_case, unused_imports)]

use hex_literal::hex;
use ssz::{types::*, Ssz};
use typenum::U;
use unionlabs::{hash::H256, uint::U256};

pub mod container_types {
    include!("../../tests-generator/src/container_types.rs");
}
use container_types::*;
#[test]
fn valid_bitvec_16_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ffff000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<16>> =
        BitVector::from_bytes([255_u8, 255_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_16_random() {
    const EXPECTED_ENCODING: &[u8] = &[46_u8, 236_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "2eec000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<16>> =
        BitVector::from_bytes([46_u8, 236_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_16_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<16>> =
        BitVector::from_bytes([0_u8, 0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_1_max() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0100000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<1>> = BitVector::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_1_random() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<1>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_1_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<1>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_2_max() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0300000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<2>> = BitVector::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_2_random() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0300000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<2>> = BitVector::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_2_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<2>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_31_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 127_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ffffff7f00000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<31>> =
        BitVector::from_bytes([255_u8, 255_u8, 255_u8, 127_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_31_random() {
    const EXPECTED_ENCODING: &[u8] = &[114_u8, 223_u8, 100_u8, 21_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "72df641500000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<31>> =
        BitVector::from_bytes([114_u8, 223_u8, 100_u8, 21_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_31_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<31>> =
        BitVector::from_bytes([0_u8, 0_u8, 0_u8, 0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_3_max() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0700000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<3>> = BitVector::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_3_random() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0700000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<3>> = BitVector::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_3_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<3>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_4_max() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0f00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<4>> = BitVector::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_4_random() {
    const EXPECTED_ENCODING: &[u8] = &[13_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0d00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<4>> = BitVector::from_bytes([13_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_4_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<4>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_512_max() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "8667e718294e9e0df1d30600ba3eeb201f764aad2dad72748643e4a285e1d1f7"
    ));

    let expected_value: BitVector<U<512>> = BitVector::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_512_random() {
    const EXPECTED_ENCODING: &[u8] = &[
        80_u8, 152_u8, 209_u8, 178_u8, 29_u8, 181_u8, 106_u8, 197_u8, 239_u8, 143_u8, 96_u8,
        144_u8, 164_u8, 12_u8, 18_u8, 211_u8, 12_u8, 52_u8, 56_u8, 123_u8, 68_u8, 155_u8, 79_u8,
        101_u8, 99_u8, 209_u8, 248_u8, 11_u8, 40_u8, 23_u8, 218_u8, 4_u8, 52_u8, 27_u8, 132_u8,
        94_u8, 218_u8, 129_u8, 196_u8, 1_u8, 104_u8, 151_u8, 30_u8, 209_u8, 179_u8, 4_u8, 107_u8,
        245_u8, 244_u8, 12_u8, 98_u8, 101_u8, 186_u8, 91_u8, 77_u8, 43_u8, 37_u8, 167_u8, 152_u8,
        212_u8, 26_u8, 235_u8, 70_u8, 222_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "fbdb71e991457c4fd956e16be1ae1dc959bceaf00f692fec9431de3f0175655a"
    ));

    let expected_value: BitVector<U<512>> = BitVector::from_bytes(
        [
            80_u8, 152_u8, 209_u8, 178_u8, 29_u8, 181_u8, 106_u8, 197_u8, 239_u8, 143_u8, 96_u8,
            144_u8, 164_u8, 12_u8, 18_u8, 211_u8, 12_u8, 52_u8, 56_u8, 123_u8, 68_u8, 155_u8,
            79_u8, 101_u8, 99_u8, 209_u8, 248_u8, 11_u8, 40_u8, 23_u8, 218_u8, 4_u8, 52_u8, 27_u8,
            132_u8, 94_u8, 218_u8, 129_u8, 196_u8, 1_u8, 104_u8, 151_u8, 30_u8, 209_u8, 179_u8,
            4_u8, 107_u8, 245_u8, 244_u8, 12_u8, 98_u8, 101_u8, 186_u8, 91_u8, 77_u8, 43_u8, 37_u8,
            167_u8, 152_u8, 212_u8, 26_u8, 235_u8, 70_u8, 222_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_512_zero() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitVector<U<512>> = BitVector::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_513_max() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "222dd9eebc6467de9788eb1c05ce9c2da8ecc89abdd38810925ce061d91236ef"
    ));

    let expected_value: BitVector<U<513>> = BitVector::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_513_random() {
    const EXPECTED_ENCODING: &[u8] = &[
        35_u8, 7_u8, 207_u8, 77_u8, 114_u8, 9_u8, 148_u8, 22_u8, 93_u8, 128_u8, 20_u8, 64_u8,
        206_u8, 32_u8, 22_u8, 204_u8, 234_u8, 200_u8, 252_u8, 243_u8, 1_u8, 27_u8, 216_u8, 80_u8,
        138_u8, 122_u8, 252_u8, 100_u8, 16_u8, 87_u8, 153_u8, 189_u8, 121_u8, 111_u8, 50_u8, 2_u8,
        193_u8, 3_u8, 227_u8, 15_u8, 13_u8, 117_u8, 201_u8, 41_u8, 104_u8, 97_u8, 61_u8, 253_u8,
        57_u8, 156_u8, 134_u8, 135_u8, 194_u8, 28_u8, 100_u8, 62_u8, 189_u8, 36_u8, 213_u8, 231_u8,
        15_u8, 225_u8, 31_u8, 196_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "84f06e5024cc71b8162c3a96f4b743505481722da5a281a6aaa69791b9f79283"
    ));

    let expected_value: BitVector<U<513>> = BitVector::from_bytes(
        [
            35_u8, 7_u8, 207_u8, 77_u8, 114_u8, 9_u8, 148_u8, 22_u8, 93_u8, 128_u8, 20_u8, 64_u8,
            206_u8, 32_u8, 22_u8, 204_u8, 234_u8, 200_u8, 252_u8, 243_u8, 1_u8, 27_u8, 216_u8,
            80_u8, 138_u8, 122_u8, 252_u8, 100_u8, 16_u8, 87_u8, 153_u8, 189_u8, 121_u8, 111_u8,
            50_u8, 2_u8, 193_u8, 3_u8, 227_u8, 15_u8, 13_u8, 117_u8, 201_u8, 41_u8, 104_u8, 97_u8,
            61_u8, 253_u8, 57_u8, 156_u8, 134_u8, 135_u8, 194_u8, 28_u8, 100_u8, 62_u8, 189_u8,
            36_u8, 213_u8, 231_u8, 15_u8, 225_u8, 31_u8, 196_u8, 0_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_513_zero() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "db56114e00fdd4c1f85c892bf35ac9a89289aaecb1ebd0a96cde606a748b5d71"
    ));

    let expected_value: BitVector<U<513>> = BitVector::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_5_max() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1f00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<5>> = BitVector::from_bytes([31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_5_random() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0300000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<5>> = BitVector::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_5_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<5>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_8_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ff00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<8>> = BitVector::from_bytes([255_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_8_random() {
    const EXPECTED_ENCODING: &[u8] = &[223_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "df00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<8>> = BitVector::from_bytes([223_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitvec_8_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: BitVector<U<8>> = BitVector::from_bytes([0_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitVector<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitVector<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
