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
fn valid_false() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: bool = false;

    assert_eq!(
        expected_value,
        <bool as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <bool as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_true() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0100000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: bool = true;

    assert_eq!(
        expected_value,
        <bool as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <bool as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
