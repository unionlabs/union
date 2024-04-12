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
fn invalid_bitlist_1_but_2() {
    const INVALID_ENCODING: &[u8] = &[7_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_1_but_8() {
    const INVALID_ENCODING: &[u8] = &[247_u8, 1_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_1_but_9() {
    const INVALID_ENCODING: &[u8] = &[124_u8, 3_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_2_but_3() {
    const INVALID_ENCODING: &[u8] = &[14_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_32_but_33() {
    const INVALID_ENCODING: &[u8] = &[106_u8, 197_u8, 239_u8, 143_u8, 2_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_32_but_64() {
    const INVALID_ENCODING: &[u8] = &[
        147_u8, 85_u8, 160_u8, 48_u8, 163_u8, 101_u8, 59_u8, 106_u8, 1_u8,
    ];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_3_but_4() {
    const INVALID_ENCODING: &[u8] = &[21_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_4_but_5() {
    const INVALID_ENCODING: &[u8] = &[44_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_512_but_513() {
    const INVALID_ENCODING: &[u8] = &[
        24_u8, 36_u8, 41_u8, 131_u8, 196_u8, 52_u8, 3_u8, 13_u8, 206_u8, 30_u8, 209_u8, 230_u8,
        83_u8, 217_u8, 88_u8, 52_u8, 254_u8, 2_u8, 202_u8, 133_u8, 54_u8, 1_u8, 205_u8, 6_u8,
        161_u8, 151_u8, 118_u8, 32_u8, 113_u8, 0_u8, 218_u8, 165_u8, 71_u8, 244_u8, 44_u8, 193_u8,
        90_u8, 61_u8, 61_u8, 131_u8, 88_u8, 153_u8, 238_u8, 86_u8, 211_u8, 74_u8, 201_u8, 41_u8,
        38_u8, 181_u8, 198_u8, 186_u8, 145_u8, 247_u8, 200_u8, 193_u8, 115_u8, 147_u8, 92_u8, 2_u8,
        165_u8, 69_u8, 23_u8, 32_u8, 3_u8,
    ];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_5_but_6() {
    const INVALID_ENCODING: &[u8] = &[87_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_8_but_9() {
    const INVALID_ENCODING: &[u8] = &[247_u8, 3_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_no_delimiter_empty() {
    const INVALID_ENCODING: &[u8] = &[];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_no_delimiter_zero_byte() {
    const INVALID_ENCODING: &[u8] = &[0_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
#[test]
fn invalid_bitlist_no_delimiter_zeroes() {
    const INVALID_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8];

    <BitList<U<1>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
}
