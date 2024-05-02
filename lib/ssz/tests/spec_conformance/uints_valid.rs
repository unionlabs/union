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
fn valid_uint_128_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffffffffffffffffffff0000000000000000000000000000000000"
    ));

    let expected_value: u128 = 1329227995784915872903807060280344575_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_max() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffffffffffffffffffffff00000000000000000000000000000000"
    ));

    let expected_value: u128 = 340282366920938463463374607431768211455_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        98_u8, 88_u8, 54_u8, 68_u8, 230_u8, 110_u8, 200_u8, 63_u8, 194_u8, 166_u8, 205_u8, 167_u8,
        35_u8, 223_u8, 250_u8, 238_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "62583644e66ec83fc2a6cda723dffaee00000000000000000000000000000000"
    ));

    let expected_value: u128 = 317658863013703600909281237913711302754_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        176_u8, 60_u8, 17_u8, 116_u8, 235_u8, 227_u8, 101_u8, 224_u8, 24_u8, 165_u8, 184_u8,
        135_u8, 81_u8, 105_u8, 88_u8, 170_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "b03c1174ebe365e018a5b887516958aa00000000000000000000000000000000"
    ));

    let expected_value: u128 = 226427817519480008631815531407103573168_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        166_u8, 138_u8, 4_u8, 241_u8, 198_u8, 247_u8, 18_u8, 130_u8, 202_u8, 19_u8, 18_u8, 18_u8,
        81_u8, 208_u8, 122_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "a68a04f1c6f71282ca13121251d07a0100000000000000000000000000000000"
    ));

    let expected_value: u128 = 1966913376797472348559631900882537126_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        225_u8, 1_u8, 206_u8, 36_u8, 193_u8, 110_u8, 195_u8, 181_u8, 124_u8, 47_u8, 11_u8, 121_u8,
        97_u8, 98_u8, 72_u8, 168_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "e101ce24c16ec3b57c2f0b79616248a800000000000000000000000000000000"
    ));

    let expected_value: u128 = 223686144064414504608552983434269426145_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        218_u8, 225_u8, 199_u8, 42_u8, 8_u8, 109_u8, 222_u8, 13_u8, 235_u8, 17_u8, 132_u8, 19_u8,
        170_u8, 68_u8, 104_u8, 150_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "dae1c72a086dde0deb118413aa44689600000000000000000000000000000000"
    ));

    let expected_value: u128 = 199925590919705556758473559487562637786_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_128_zero() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u128 = 0_u128;

    assert_eq!(
        expected_value,
        <u128 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u128 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ff00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 255_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffff000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 65535_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[249_u8, 42_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "f92a000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 11001_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[100_u8, 50_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "6432000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 12900_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[146_u8, 181_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "92b5000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 46482_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[63_u8, 121_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "3f79000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 31039_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[236_u8, 8_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ec08000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 2284_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_16_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u16 = 0_u16;

    assert_eq!(
        expected_value,
        <u16 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u16 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff00"
    ));

    let expected_value: U256 = U256::from_limbs([
        18446744073709551615_u64,
        18446744073709551615_u64,
        18446744073709551615_u64,
        72057594037927935_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_max() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    ));

    let expected_value: U256 = U256::from_limbs([
        18446744073709551615_u64,
        18446744073709551615_u64,
        18446744073709551615_u64,
        18446744073709551615_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        58_u8, 55_u8, 99_u8, 28_u8, 168_u8, 145_u8, 249_u8, 244_u8, 255_u8, 81_u8, 153_u8, 135_u8,
        170_u8, 128_u8, 39_u8, 36_u8, 202_u8, 1_u8, 166_u8, 171_u8, 97_u8, 55_u8, 46_u8, 78_u8,
        36_u8, 161_u8, 66_u8, 116_u8, 168_u8, 139_u8, 34_u8, 10_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "3a37631ca891f9f4ff519987aa802724ca01a6ab61372e4e24a14274a88b220a"
    ));

    let expected_value: U256 = U256::from_limbs([
        17652300365672167226_u64,
        2605192379364889087_u64,
        5633501076518207946_u64,
        730299645208731940_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        160_u8, 200_u8, 243_u8, 199_u8, 115_u8, 30_u8, 235_u8, 132_u8, 127_u8, 224_u8, 146_u8,
        208_u8, 192_u8, 97_u8, 24_u8, 112_u8, 2_u8, 157_u8, 177_u8, 75_u8, 95_u8, 22_u8, 105_u8,
        70_u8, 180_u8, 97_u8, 182_u8, 31_u8, 39_u8, 79_u8, 21_u8, 199_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "a0c8f3c7731eeb847fe092d0c0611870029db14b5f166946b461b61f274f15c7"
    ));

    let expected_value: U256 = U256::from_limbs([
        9577782515158206624_u64,
        8077313412449886335_u64,
        5073611053757209858_u64,
        14345459217512948148_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        145_u8, 36_u8, 54_u8, 124_u8, 134_u8, 65_u8, 119_u8, 96_u8, 224_u8, 3_u8, 87_u8, 209_u8,
        164_u8, 118_u8, 23_u8, 209_u8, 5_u8, 72_u8, 9_u8, 168_u8, 251_u8, 195_u8, 102_u8, 65_u8,
        122_u8, 101_u8, 27_u8, 164_u8, 66_u8, 115_u8, 0_u8, 49_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "9124367c86417760e00357d1a47617d1054809a8fbc366417a651ba442730031"
    ));

    let expected_value: U256 = U256::from_limbs([
        6951096595734996113_u64,
        15066641528650138592_u64,
        4712669545713190917_u64,
        3530948837916763514_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        9_u8, 220_u8, 230_u8, 65_u8, 45_u8, 6_u8, 68_u8, 219_u8, 208_u8, 26_u8, 176_u8, 18_u8,
        183_u8, 94_u8, 87_u8, 176_u8, 157_u8, 70_u8, 34_u8, 109_u8, 52_u8, 201_u8, 18_u8, 243_u8,
        217_u8, 129_u8, 175_u8, 51_u8, 196_u8, 80_u8, 238_u8, 25_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "09dce6412d0644dbd01ab012b75e57b09d46226d34c912f3d981af33c450ee19"
    ));

    let expected_value: U256 = U256::from_limbs([
        15799760184171486217_u64,
        12706729014034701008_u64,
        17515283127803725469_u64,
        1868519699016286681_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        236_u8, 44_u8, 123_u8, 92_u8, 134_u8, 169_u8, 87_u8, 238_u8, 98_u8, 219_u8, 210_u8, 219_u8,
        26_u8, 37_u8, 128_u8, 52_u8, 156_u8, 71_u8, 217_u8, 131_u8, 206_u8, 187_u8, 193_u8, 227_u8,
        34_u8, 128_u8, 209_u8, 179_u8, 17_u8, 9_u8, 210_u8, 107_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ec2c7b5c86a957ee62dbd2db1a2580349c47d983cebbc1e32280d1b31109d26b"
    ));

    let expected_value: U256 = U256::from_limbs([
        17174382098542963948_u64,
        3783064484278623074_u64,
        16411605012764510108_u64,
        7769282278803472418_u64,
    ]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_256_zero() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: U256 = U256::from_limbs([0_u64, 0_u64, 0_u64, 0_u64]);

    assert_eq!(
        expected_value,
        <U256 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <U256 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffff0000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 16777215_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 255_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffff00000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 4294967295_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[72_u8, 2_u8, 237_u8, 201_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "4802edc900000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 3387753032_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[251_u8, 95_u8, 143_u8, 159_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "fb5f8f9f00000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 2676973563_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[253_u8, 24_u8, 166_u8, 157_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "fd18a69d00000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 2644908285_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[95_u8, 173_u8, 7_u8, 38_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "5fad072600000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 638037343_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 201_u8, 3_u8, 247_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffc903f700000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 4144220671_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_32_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u32 = 0_u32;

    assert_eq!(
        expected_value,
        <u32 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u32 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] =
        &[255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffff00000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 72057594037927935_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_max() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
    ];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ffffffffffffffff000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 18446744073709551615_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[53_u8, 124_u8, 141_u8, 233_u8, 215_u8, 32_u8, 69_u8, 119_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "357c8de9d7204577000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 8594311575614880821_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_random_1() {
    const EXPECTED_ENCODING: &[u8] =
        &[60_u8, 130_u8, 249_u8, 153_u8, 102_u8, 30_u8, 213_u8, 172_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "3c82f999661ed5ac000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 12453893770581738044_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_random_2() {
    const EXPECTED_ENCODING: &[u8] =
        &[167_u8, 252_u8, 217_u8, 131_u8, 32_u8, 133_u8, 57_u8, 148_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "a7fcd98320853994000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 10680714365983390887_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[229_u8, 219_u8, 37_u8, 16_u8, 197_u8, 191_u8, 6_u8, 165_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "e5db2510c5bf06a5000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 11891402719218752485_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8, 51_u8, 37_u8, 123_u8, 13_u8, 74_u8, 165_u8, 217_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "1f33257b0d4aa5d9000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 15683022699148686111_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_64_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u64 = 0_u64;

    assert_eq!(
        expected_value,
        <u64 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u64 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_last_byte_empty() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 0_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_max() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "ff00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 255_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[225_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "e100000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 225_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[59_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "3b00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 59_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0300000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 3_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[46_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "2e00000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 46_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[17_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "1100000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 17_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
#[test]
fn valid_uint_8_zero() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8];
    const EXPECTED_ROOT: H256 = H256(hex!(
        "0000000000000000000000000000000000000000000000000000000000000000"
    ));

    let expected_value: u8 = 0_u8;

    assert_eq!(
        expected_value,
        <u8 as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <u8 as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
}
