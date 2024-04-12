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
fn invalid_byte_0x80() {
    const INVALID_ENCODING: &[u8] = &[128_u8];

    <bool as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_byte_2() {
    const INVALID_ENCODING: &[u8] = &[2_u8];

    <bool as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_byte_full() {
    const INVALID_ENCODING: &[u8] = &[255_u8];

    <bool as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_byte_rev_nibble() {
    const INVALID_ENCODING: &[u8] = &[16_u8];

    <bool as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
