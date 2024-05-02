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
fn invalid_bitvec_0() {
    const INVALID_ENCODING: &[u8] = &[];

    <BitVector<U<0>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_16_max_8() {
    const INVALID_ENCODING: &[u8] = &[255_u8];

    <BitVector<U<16>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_16_random_8() {
    const INVALID_ENCODING: &[u8] = &[203_u8];

    <BitVector<U<16>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_16_zero_8() {
    const INVALID_ENCODING: &[u8] = &[0_u8];

    <BitVector<U<16>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_1_max_2() {
    const INVALID_ENCODING: &[u8] = &[3_u8];

    <BitVector<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_1_random_2() {
    const INVALID_ENCODING: &[u8] = &[2_u8];

    <BitVector<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_1_zero_2() {
    const INVALID_ENCODING: &[u8] = &[2_u8];

    <BitVector<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_2_max_3() {
    const INVALID_ENCODING: &[u8] = &[7_u8];

    <BitVector<U<2>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_2_random_3() {
    const INVALID_ENCODING: &[u8] = &[7_u8];

    <BitVector<U<2>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_2_zero_3() {
    const INVALID_ENCODING: &[u8] = &[4_u8];

    <BitVector<U<2>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_32_max_33() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 255_u8, 1_u8];

    <BitVector<U<32>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_32_random_33() {
    const INVALID_ENCODING: &[u8] = &[125_u8, 147_u8, 85_u8, 160_u8, 1_u8];

    <BitVector<U<32>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_32_zero_33() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 1_u8];

    <BitVector<U<32>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_3_max_4() {
    const INVALID_ENCODING: &[u8] = &[15_u8];

    <BitVector<U<3>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_3_random_4() {
    const INVALID_ENCODING: &[u8] = &[11_u8];

    <BitVector<U<3>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_3_zero_4() {
    const INVALID_ENCODING: &[u8] = &[8_u8];

    <BitVector<U<3>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_4_max_5() {
    const INVALID_ENCODING: &[u8] = &[31_u8];

    <BitVector<U<4>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_4_random_5() {
    const INVALID_ENCODING: &[u8] = &[23_u8];

    <BitVector<U<4>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_4_zero_5() {
    const INVALID_ENCODING: &[u8] = &[16_u8];

    <BitVector<U<4>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_512_max_513() {
    const INVALID_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 1_u8,
    ];

    <BitVector<U<512>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_512_random_513() {
    const INVALID_ENCODING: &[u8] = &[
        152_u8, 209_u8, 178_u8, 29_u8, 181_u8, 106_u8, 197_u8, 239_u8, 143_u8, 96_u8, 144_u8,
        164_u8, 12_u8, 18_u8, 211_u8, 12_u8, 52_u8, 56_u8, 123_u8, 68_u8, 155_u8, 79_u8, 101_u8,
        99_u8, 209_u8, 248_u8, 11_u8, 40_u8, 23_u8, 218_u8, 4_u8, 52_u8, 27_u8, 132_u8, 94_u8,
        218_u8, 129_u8, 196_u8, 1_u8, 104_u8, 151_u8, 30_u8, 209_u8, 179_u8, 4_u8, 107_u8, 245_u8,
        244_u8, 12_u8, 98_u8, 101_u8, 186_u8, 91_u8, 77_u8, 43_u8, 37_u8, 167_u8, 152_u8, 212_u8,
        26_u8, 235_u8, 70_u8, 222_u8, 35_u8, 1_u8,
    ];

    <BitVector<U<512>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_512_zero_513() {
    const INVALID_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 1_u8,
    ];

    <BitVector<U<512>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_5_max_6() {
    const INVALID_ENCODING: &[u8] = &[63_u8];

    <BitVector<U<5>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_5_random_6() {
    const INVALID_ENCODING: &[u8] = &[62_u8];

    <BitVector<U<5>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_5_zero_6() {
    const INVALID_ENCODING: &[u8] = &[32_u8];

    <BitVector<U<5>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_8_max_9() {
    const INVALID_ENCODING: &[u8] = &[255_u8, 1_u8];

    <BitVector<U<8>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_8_random_9() {
    const INVALID_ENCODING: &[u8] = &[118_u8, 1_u8];

    <BitVector<U<8>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_8_zero_9() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 1_u8];

    <BitVector<U<8>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_9_max_8() {
    const INVALID_ENCODING: &[u8] = &[255_u8];

    <BitVector<U<9>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_9_random_8() {
    const INVALID_ENCODING: &[u8] = &[176_u8];

    <BitVector<U<9>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitvec_9_zero_8() {
    const INVALID_ENCODING: &[u8] = &[0_u8];

    <BitVector<U<9>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
