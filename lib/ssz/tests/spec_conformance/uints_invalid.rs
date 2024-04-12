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
fn invalid_uint_128_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 0_u8,
    ];

    <u128 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_128_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8,
    ];

    <u128 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_128_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 1_u8,
    ];

    <u128 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_16_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 255_u8, 0_u8];

    <u16 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_16_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[255_u8];

    <u16 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_16_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 0_u8, 1_u8];

    <u16 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_256_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 0_u8,
    ];

    <U256 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_256_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
    ];

    <U256 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_256_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 1_u8,
    ];

    <U256 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_32_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 255_u8, 0_u8];

    <u32 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_32_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8];

    <u32 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_32_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 1_u8];

    <u32 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_64_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 0_u8,
    ];

    <u64 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_64_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8];

    <u64 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_64_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8];

    <u64 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_8_one_byte_longer() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 0_u8];

    <u8 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_8_one_byte_shorter() {
    const INVALID_ENCODING: &[u8] = &[];

    <u8 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_uint_8_one_too_high() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 1_u8];

    <u8 as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
