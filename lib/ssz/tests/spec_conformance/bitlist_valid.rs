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
fn valid_bitlist_16_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[160_u8, 92_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "6232812aa34ca3e9ce77374f8915f059832b1671edbbe38e8816196b2be450d5"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([160_u8, 92_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[64_u8, 179_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "8c2b7bd1b88a7d1be36dad5c3734873af45f38d2d4618f83211b394aa65a665e"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([64_u8, 179_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[161_u8, 151_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "fc0027195d4d241e8d3111d41d749a46f62e2d0e78aa503b856774abe6b7e6c3"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([161_u8, 151_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[137_u8, 3_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "50fea858f788bbc2f17f809e05682bf855493a7b8c594f4c2342b469ac7bdb53"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([137_u8, 3_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[30_u8, 209_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "983039dcf7ee961e2a2c1b1d0b57ad04491b8674c0f9f6dc326244e48dacd851"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([30_u8, 209_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "017d2fa0f6934ed2354e4cdb7a2230ccf8f31fe758c7a47442e37fdea1d68bfe"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([255_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "dc8212e2404720c98554dfddc81733f88cbbe307a1d4ca5eae4b88e55e382392"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([255_u8, 255_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "5879404f965b9356ffe1e124c2ef7aef85a31eda844aa967aa74d3422a7e2b2e"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([255_u8, 7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 63_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "16472e350c0d8e0cf112307b5cfa66561668ffef5f9f3281c9ad0af85122ba2c"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([255_u8, 63_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4b07c3799db025f3aa92ced1e8545367a2b6e44960f479d3f9d62b61812892d5"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[180_u8, 3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "eec57ef94d128f67c545a95b84f97501237ed672f583769110409b2df50bce84"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([180_u8, 3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[59_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "8bd00e1a82454504a094276182544df713103259ba3f96133871a55281b44d18"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([59_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[104_u8, 23_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "160937bf5c6f4256c285385214969c965a8c841be474c62d7ed3c184ec3cdb69"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([104_u8, 23_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[25_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "53de69c30b9c07be9cba006e32db34dc1e4ebfe649bc94aa7c8aae0ef419aeed"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([25_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "69713c9ac33bde909bd8763512e69a7f523d544adcfb8c892e24bc8f6341ea16"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([0_u8, 64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7b460f51b362b95b384743dda74f56fbcd35f4d8e7ebda7206632e60c91e663d"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([0_u8, 2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "b04cc2cb8ea6754f94c2e7403cf58e20c9023a98350c84282966e0bd6729d3ca"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([0_u8, 4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7d360196d14b15261c9e5f576df8dc8b48d18d79b4198f167741052747704352"
    ));

    let expected_value: BitList<U<16>> = BitList::from_bytes([64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_16_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "a44a029e04493b8d2fe7893391c2b3ceefec1603c585aad6203f2d14e07bfead"
    ));

    let expected_value: BitList<U<16>> =
        BitList::from_bytes([0_u8, 0_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<16>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<16>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_1_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<1>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<1>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<1>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[5_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ff55c97976a840b4ced964ed49e3794594ba3f675238b5fd25d282b60f70a194"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([5_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[5_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ff55c97976a840b4ced964ed49e3794594ba3f675238b5fd25d282b60f70a194"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([5_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[5_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ff55c97976a840b4ced964ed49e3794594ba3f675238b5fd25d282b60f70a194"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([5_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[6_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0e01f8d9a6720610a44a70c2c91bbe750ec6cd67892d92b1016394abfc382cf9"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([6_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_2_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<2>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<2>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<2>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8, 107_u8, 245_u8, 244_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "2b4e175a3cabe516e47026098d7a07a105d94c6e1d7859c5f8e99d81d5fb73e5"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([4_u8, 107_u8, 245_u8, 244_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8, 43_u8, 211_u8, 221_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5054e572357a7c57c9f05e8f79208348c9dfe9f28461d7935700459b1ae2307"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([16_u8, 43_u8, 211_u8, 221_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[149_u8, 146_u8, 83_u8, 204_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "57bdf36005bb9113c2b89db95c10946d97609b3173d4397a1a74755d0c6490f8"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([149_u8, 146_u8, 83_u8, 204_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[188_u8, 71_u8, 14_u8, 158_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0faa1049c965bf5a37db3b457dcc3a2ee179ef704c42a29722641b2ec3bb3658"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([188_u8, 71_u8, 14_u8, 158_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[46_u8, 129_u8, 210_u8, 162_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "152b52ebbfc701c7a39758748e1f14b4361ae37dd480b6914aa725824cde97f2"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([46_u8, 129_u8, 210_u8, 162_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ebe018d5287ea5be7d789946da9587c27f5dd82d8c120a594ae0e8ddd2e21802"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([255_u8, 255_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "e78c29807c3f3ced69109d22d734a1c69d361e0671c21b8681a1761333e95537"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([255_u8, 255_u8, 255_u8, 3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "b8570b9c932d5fd3d2bd727a64d527f790d8261acd9f6ce2786cc1fa34dd2fa8"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([255_u8, 255_u8, 255_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4b5bcf109d8b0381e1ca551794c9fb864838f5b07057e05da75830f7999d96de"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([255_u8, 31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[106_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "62681102fbb14f3973d9db3e302be35e5bbd79984aed6a85c532c63189afb38a"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([106_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[106_u8, 141_u8, 117_u8, 7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1037ee25750a944efe9b3dc796628f6468a9f242bd791013c439ca785c134482"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([106_u8, 141_u8, 117_u8, 7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[155_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "5940967aaa293730d0e7876047dfceb9cf5512fafb5d4be3d05c776902163786"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([155_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "6463f4376faab07e62e5a4737d2d95ad690892f8fae0b9559c0ed3ae96bb2790"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([0_u8, 0_u8, 0_u8, 16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7d934ef6667cff3afea0633d57baa9a82a7009f89b0f8c12f47150047098b396"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([0_u8, 0_u8, 0_u8, 64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "967293ee9d7ba679c3ef076bef139e2ceb96d45d19a624cc59bb5a3c1649ce38"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([0_u8, 0_u8, 2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 8_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "04997ec49450b710d4d92e2e6e92c47b193b0ec6f841d7d692bf0f410cbc7269"
    ));

    let expected_value: BitList<U<31>> =
        BitList::from_bytes([0_u8, 0_u8, 0_u8, 8_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_31_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<31>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<31>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<31>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[8_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d86ae2ca925345bf2412bde450ac175742d979c1ea7b961bd1efe10beb9500cf"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([8_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[8_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d86ae2ca925345bf2412bde450ac175742d979c1ea7b961bd1efe10beb9500cf"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([8_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[9_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "caea92341df83aa8d4225099f16e86cbf457ec7ea97ccddb4ba5560062eee695"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([9_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[12_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d3156136ef0ebd0cb8945f7c18cfe8ad539d08d8703744bc11371e49e6a4d9ad"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([12_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[5_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ff55c97976a840b4ced964ed49e3794594ba3f675238b5fd25d282b60f70a194"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([5_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[8_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d86ae2ca925345bf2412bde450ac175742d979c1ea7b961bd1efe10beb9500cf"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([8_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_3_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<3>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<3>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<3>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[27_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "9d2816f451512382c000156fad1578555537321084d091d3c7b228aa705c36aa"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([27_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[21_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "e90722eb4d2a891700f1f3aa2e95661e707b19e60e147a96f8cf089e8cbc4bec"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([21_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[23_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "374bd7c88680671ad4be6e1b576db80646d992d893a5eeb1d1d0f403c3331b32"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([23_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[17_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f9c5ada16029ed1580188989686f19e749c006b2eac37d3ef087b824b31ba997"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([17_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[22_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "894e8a2ce460c6c6ba12d467634e6c34ce2a1b58d0c6dfe3d98b532898c58611"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([22_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4b07c3799db025f3aa92ced1e8545367a2b6e44960f479d3f9d62b61812892d5"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[13_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cf8ca64c265b9b6234fb7573a200745204fd04fecf680f1157f27367ee8f4aa2"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([13_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[17_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f9c5ada16029ed1580188989686f19e749c006b2eac37d3ef087b824b31ba997"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([17_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d647eb2598d33d7216256356596d29cecd31c1ba7a7ff25ccb5be4a453410b9d"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_4_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d647eb2598d33d7216256356596d29cecd31c1ba7a7ff25ccb5be4a453410b9d"
    ));

    let expected_value: BitList<U<4>> = BitList::from_bytes([16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<4>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<4>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 41_u8, 128_u8, 156_u8, 65_u8, 44_u8, 152_u8, 213_u8, 145_u8, 249_u8, 231_u8, 3_u8,
        54_u8, 176_u8, 161_u8, 20_u8, 245_u8, 248_u8, 201_u8, 32_u8, 174_u8, 50_u8, 123_u8, 243_u8,
        222_u8, 100_u8, 4_u8, 130_u8, 7_u8, 198_u8, 31_u8, 26_u8, 234_u8, 146_u8, 83_u8, 208_u8,
        194_u8, 122_u8, 250_u8, 115_u8, 56_u8, 13_u8, 15_u8, 133_u8, 57_u8, 200_u8, 124_u8, 122_u8,
        73_u8, 170_u8, 207_u8, 31_u8, 194_u8, 63_u8, 136_u8, 137_u8, 67_u8, 132_u8, 226_u8, 136_u8,
        214_u8, 24_u8, 88_u8, 128_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "bc152fc83f6fefea40b3b3fdf626dc1af7eaea74e6bce7aba12a6602679004e1"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            0_u8, 41_u8, 128_u8, 156_u8, 65_u8, 44_u8, 152_u8, 213_u8, 145_u8, 249_u8, 231_u8,
            3_u8, 54_u8, 176_u8, 161_u8, 20_u8, 245_u8, 248_u8, 201_u8, 32_u8, 174_u8, 50_u8,
            123_u8, 243_u8, 222_u8, 100_u8, 4_u8, 130_u8, 7_u8, 198_u8, 31_u8, 26_u8, 234_u8,
            146_u8, 83_u8, 208_u8, 194_u8, 122_u8, 250_u8, 115_u8, 56_u8, 13_u8, 15_u8, 133_u8,
            57_u8, 200_u8, 124_u8, 122_u8, 73_u8, 170_u8, 207_u8, 31_u8, 194_u8, 63_u8, 136_u8,
            137_u8, 67_u8, 132_u8, 226_u8, 136_u8, 214_u8, 24_u8, 88_u8, 128_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        94_u8, 184_u8, 1_u8, 99_u8, 49_u8, 235_u8, 82_u8, 205_u8, 193_u8, 182_u8, 48_u8, 217_u8,
        245_u8, 230_u8, 218_u8, 190_u8, 200_u8, 144_u8, 130_u8, 53_u8, 80_u8, 167_u8, 110_u8,
        209_u8, 85_u8, 249_u8, 213_u8, 138_u8, 164_u8, 151_u8, 175_u8, 125_u8, 200_u8, 249_u8,
        196_u8, 207_u8, 214_u8, 174_u8, 199_u8, 72_u8, 133_u8, 65_u8, 202_u8, 173_u8, 121_u8, 9_u8,
        100_u8, 29_u8, 6_u8, 211_u8, 31_u8, 222_u8, 177_u8, 39_u8, 51_u8, 58_u8, 37_u8, 173_u8,
        90_u8, 222_u8, 24_u8, 104_u8, 3_u8, 232_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "2b28c2217c3f1f99e0c5ad46c77be392323ae7c6e68612e6b1701e762a0285e7"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            94_u8, 184_u8, 1_u8, 99_u8, 49_u8, 235_u8, 82_u8, 205_u8, 193_u8, 182_u8, 48_u8,
            217_u8, 245_u8, 230_u8, 218_u8, 190_u8, 200_u8, 144_u8, 130_u8, 53_u8, 80_u8, 167_u8,
            110_u8, 209_u8, 85_u8, 249_u8, 213_u8, 138_u8, 164_u8, 151_u8, 175_u8, 125_u8, 200_u8,
            249_u8, 196_u8, 207_u8, 214_u8, 174_u8, 199_u8, 72_u8, 133_u8, 65_u8, 202_u8, 173_u8,
            121_u8, 9_u8, 100_u8, 29_u8, 6_u8, 211_u8, 31_u8, 222_u8, 177_u8, 39_u8, 51_u8, 58_u8,
            37_u8, 173_u8, 90_u8, 222_u8, 24_u8, 104_u8, 3_u8, 232_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        161_u8, 54_u8, 224_u8, 29_u8, 250_u8, 19_u8, 154_u8, 61_u8, 234_u8, 110_u8, 180_u8, 240_u8,
        239_u8, 156_u8, 21_u8, 64_u8, 107_u8, 106_u8, 17_u8, 233_u8, 80_u8, 37_u8, 181_u8, 67_u8,
        39_u8, 44_u8, 105_u8, 232_u8, 196_u8, 133_u8, 9_u8, 158_u8, 189_u8, 240_u8, 53_u8, 62_u8,
        87_u8, 32_u8, 162_u8, 166_u8, 95_u8, 142_u8, 186_u8, 239_u8, 32_u8, 16_u8, 225_u8, 190_u8,
        107_u8, 170_u8, 72_u8, 24_u8, 81_u8, 252_u8, 124_u8, 121_u8, 154_u8, 69_u8, 9_u8, 186_u8,
        172_u8, 147_u8, 135_u8, 220_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "04d0ff41239e5365cafa09c58dedb823eb13cb4912afea9fc26a658b955a4594"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            161_u8, 54_u8, 224_u8, 29_u8, 250_u8, 19_u8, 154_u8, 61_u8, 234_u8, 110_u8, 180_u8,
            240_u8, 239_u8, 156_u8, 21_u8, 64_u8, 107_u8, 106_u8, 17_u8, 233_u8, 80_u8, 37_u8,
            181_u8, 67_u8, 39_u8, 44_u8, 105_u8, 232_u8, 196_u8, 133_u8, 9_u8, 158_u8, 189_u8,
            240_u8, 53_u8, 62_u8, 87_u8, 32_u8, 162_u8, 166_u8, 95_u8, 142_u8, 186_u8, 239_u8,
            32_u8, 16_u8, 225_u8, 190_u8, 107_u8, 170_u8, 72_u8, 24_u8, 81_u8, 252_u8, 124_u8,
            121_u8, 154_u8, 69_u8, 9_u8, 186_u8, 172_u8, 147_u8, 135_u8, 220_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        7_u8, 77_u8, 193_u8, 97_u8, 6_u8, 198_u8, 213_u8, 2_u8, 67_u8, 206_u8, 135_u8, 31_u8,
        250_u8, 188_u8, 197_u8, 117_u8, 43_u8, 158_u8, 239_u8, 119_u8, 214_u8, 235_u8, 178_u8,
        157_u8, 44_u8, 138_u8, 110_u8, 35_u8, 81_u8, 248_u8, 201_u8, 84_u8, 190_u8, 88_u8, 184_u8,
        122_u8, 203_u8, 27_u8, 77_u8, 169_u8, 14_u8, 235_u8, 229_u8, 112_u8, 105_u8, 217_u8, 5_u8,
        207_u8, 151_u8, 239_u8, 101_u8, 112_u8, 49_u8, 80_u8, 124_u8, 19_u8, 32_u8, 179_u8, 8_u8,
        81_u8, 221_u8, 223_u8, 225_u8, 108_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "57d984dd8dc742665160586d43e684d59f48ea2fbf7ff6fc6742cdcf050bea09"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            7_u8, 77_u8, 193_u8, 97_u8, 6_u8, 198_u8, 213_u8, 2_u8, 67_u8, 206_u8, 135_u8, 31_u8,
            250_u8, 188_u8, 197_u8, 117_u8, 43_u8, 158_u8, 239_u8, 119_u8, 214_u8, 235_u8, 178_u8,
            157_u8, 44_u8, 138_u8, 110_u8, 35_u8, 81_u8, 248_u8, 201_u8, 84_u8, 190_u8, 88_u8,
            184_u8, 122_u8, 203_u8, 27_u8, 77_u8, 169_u8, 14_u8, 235_u8, 229_u8, 112_u8, 105_u8,
            217_u8, 5_u8, 207_u8, 151_u8, 239_u8, 101_u8, 112_u8, 49_u8, 80_u8, 124_u8, 19_u8,
            32_u8, 179_u8, 8_u8, 81_u8, 221_u8, 223_u8, 225_u8, 108_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        18_u8, 50_u8, 181_u8, 231_u8, 116_u8, 52_u8, 232_u8, 102_u8, 4_u8, 191_u8, 32_u8, 1_u8,
        249_u8, 166_u8, 102_u8, 194_u8, 183_u8, 79_u8, 119_u8, 8_u8, 192_u8, 206_u8, 36_u8, 45_u8,
        150_u8, 71_u8, 35_u8, 141_u8, 185_u8, 152_u8, 251_u8, 167_u8, 194_u8, 89_u8, 90_u8, 22_u8,
        11_u8, 201_u8, 96_u8, 126_u8, 130_u8, 62_u8, 114_u8, 237_u8, 146_u8, 167_u8, 254_u8, 2_u8,
        60_u8, 134_u8, 87_u8, 216_u8, 0_u8, 37_u8, 182_u8, 251_u8, 107_u8, 210_u8, 41_u8, 13_u8,
        21_u8, 196_u8, 118_u8, 247_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28933deb812002abaf34c610f6b2f77cb8acbc617d5a8f8a320ca4813c29fea2"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            18_u8, 50_u8, 181_u8, 231_u8, 116_u8, 52_u8, 232_u8, 102_u8, 4_u8, 191_u8, 32_u8, 1_u8,
            249_u8, 166_u8, 102_u8, 194_u8, 183_u8, 79_u8, 119_u8, 8_u8, 192_u8, 206_u8, 36_u8,
            45_u8, 150_u8, 71_u8, 35_u8, 141_u8, 185_u8, 152_u8, 251_u8, 167_u8, 194_u8, 89_u8,
            90_u8, 22_u8, 11_u8, 201_u8, 96_u8, 126_u8, 130_u8, 62_u8, 114_u8, 237_u8, 146_u8,
            167_u8, 254_u8, 2_u8, 60_u8, 134_u8, 87_u8, 216_u8, 0_u8, 37_u8, 182_u8, 251_u8,
            107_u8, 210_u8, 41_u8, 13_u8, 21_u8, 196_u8, 118_u8, 247_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 7_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "bbf3224946b87b12d7c3c24d4887a1a1bdb6afd356e3fb40bfa7a42cd0a7d478"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 7_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ca6250f3556974d64650a327c0551859f706d9778399caff8a6be920d88fb39f"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 127_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "08e61443f630601ca65f47622a47ef029baad7a757f3f1d10de0098c9add4589"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 127_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "8cbf50b584a296a316a71c486b4d4e1fd94edae9bf75f1aff71b8f609dc8352c"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8]
            .to_vec()
            .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 31_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1bb7ab569c8b46d1e40884241195c1369ea760bf957583d3a78a4315c0e2f495"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 31_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7a0501f5957bdf9cb3a8ff4966f02265f968658b7a9c62642cba1165e86642f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7a0501f5957bdf9cb3a8ff4966f02265f968658b7a9c62642cba1165e86642f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7a0501f5957bdf9cb3a8ff4966f02265f968658b7a9c62642cba1165e86642f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7a0501f5957bdf9cb3a8ff4966f02265f968658b7a9c62642cba1165e86642f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7a0501f5957bdf9cb3a8ff4966f02265f968658b7a9c62642cba1165e86642f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        159_u8, 193_u8, 65_u8, 83_u8, 153_u8, 201_u8, 138_u8, 172_u8, 149_u8, 135_u8, 213_u8,
        123_u8, 40_u8, 144_u8, 145_u8, 240_u8, 5_u8, 101_u8, 30_u8, 231_u8, 103_u8, 140_u8, 116_u8,
        89_u8, 117_u8, 244_u8, 128_u8, 217_u8, 64_u8, 252_u8, 136_u8, 195_u8, 115_u8, 226_u8,
        161_u8, 234_u8, 156_u8, 221_u8, 9_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d01782fa00046d31ecef1828d806bc82a0635ba68a829abaea5bc5e83cfc3b39"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            159_u8, 193_u8, 65_u8, 83_u8, 153_u8, 201_u8, 138_u8, 172_u8, 149_u8, 135_u8, 213_u8,
            123_u8, 40_u8, 144_u8, 145_u8, 240_u8, 5_u8, 101_u8, 30_u8, 231_u8, 103_u8, 140_u8,
            116_u8, 89_u8, 117_u8, 244_u8, 128_u8, 217_u8, 64_u8, 252_u8, 136_u8, 195_u8, 115_u8,
            226_u8, 161_u8, 234_u8, 156_u8, 221_u8, 9_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[138_u8, 214_u8, 36_u8, 127_u8, 4_u8, 4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4979bcefe3ded00d52ea1342595d1390e372a93c4acf10ed2c3c1fc604d1a92e"
    ));

    let expected_value: BitList<U<512>> =
        BitList::from_bytes([138_u8, 214_u8, 36_u8, 127_u8, 4_u8, 4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        55_u8, 212_u8, 241_u8, 100_u8, 234_u8, 188_u8, 0_u8, 34_u8, 27_u8, 192_u8, 190_u8, 127_u8,
        91_u8, 209_u8, 229_u8, 142_u8, 167_u8, 28_u8, 205_u8, 84_u8, 238_u8, 210_u8, 242_u8,
        138_u8, 164_u8, 125_u8, 156_u8, 167_u8, 143_u8, 125_u8, 166_u8, 47_u8, 243_u8, 117_u8,
        126_u8, 131_u8, 242_u8, 82_u8, 247_u8, 247_u8, 109_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0885e8d339f7016801875ef256eb180be417810e6151703137877f68926952f5"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            55_u8, 212_u8, 241_u8, 100_u8, 234_u8, 188_u8, 0_u8, 34_u8, 27_u8, 192_u8, 190_u8,
            127_u8, 91_u8, 209_u8, 229_u8, 142_u8, 167_u8, 28_u8, 205_u8, 84_u8, 238_u8, 210_u8,
            242_u8, 138_u8, 164_u8, 125_u8, 156_u8, 167_u8, 143_u8, 125_u8, 166_u8, 47_u8, 243_u8,
            117_u8, 126_u8, 131_u8, 242_u8, 82_u8, 247_u8, 247_u8, 109_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        114_u8, 184_u8, 230_u8, 175_u8, 17_u8, 28_u8, 190_u8, 227_u8, 166_u8, 99_u8, 92_u8, 23_u8,
        192_u8, 50_u8, 85_u8, 247_u8, 116_u8, 227_u8, 60_u8, 162_u8, 196_u8, 86_u8, 135_u8, 217_u8,
        176_u8, 85_u8, 95_u8, 52_u8, 220_u8, 1_u8, 21_u8, 183_u8, 204_u8, 52_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0c24b4aa44483bc91415618c8d23fa1ec87cbbf57dd1747ac001513f3ddeea8c"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            114_u8, 184_u8, 230_u8, 175_u8, 17_u8, 28_u8, 190_u8, 227_u8, 166_u8, 99_u8, 92_u8,
            23_u8, 192_u8, 50_u8, 85_u8, 247_u8, 116_u8, 227_u8, 60_u8, 162_u8, 196_u8, 86_u8,
            135_u8, 217_u8, 176_u8, 85_u8, 95_u8, 52_u8, 220_u8, 1_u8, 21_u8, 183_u8, 204_u8,
            52_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        96_u8, 46_u8, 1_u8, 71_u8, 234_u8, 203_u8, 14_u8, 250_u8, 120_u8, 192_u8, 34_u8, 73_u8,
        169_u8, 192_u8, 182_u8, 165_u8, 161_u8, 213_u8, 114_u8, 128_u8, 250_u8, 72_u8, 183_u8,
        182_u8, 164_u8, 2_u8, 138_u8, 211_u8, 54_u8, 233_u8, 137_u8, 237_u8, 21_u8, 53_u8, 242_u8,
        229_u8, 217_u8, 36_u8, 123_u8, 228_u8, 172_u8, 63_u8, 139_u8, 69_u8, 245_u8, 130_u8,
        136_u8, 245_u8, 70_u8, 77_u8, 6_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "813ccb937403bbd02d4ce9cd7e101c3bf3214ed4a1d8c11199288fbcdca45860"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            96_u8, 46_u8, 1_u8, 71_u8, 234_u8, 203_u8, 14_u8, 250_u8, 120_u8, 192_u8, 34_u8, 73_u8,
            169_u8, 192_u8, 182_u8, 165_u8, 161_u8, 213_u8, 114_u8, 128_u8, 250_u8, 72_u8, 183_u8,
            182_u8, 164_u8, 2_u8, 138_u8, 211_u8, 54_u8, 233_u8, 137_u8, 237_u8, 21_u8, 53_u8,
            242_u8, 229_u8, 217_u8, 36_u8, 123_u8, 228_u8, 172_u8, 63_u8, 139_u8, 69_u8, 245_u8,
            130_u8, 136_u8, 245_u8, 70_u8, 77_u8, 6_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 16_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "b9622f5ac7a4f2982e31494019e6fc83a8510ba1313084df18fe74cfd63fff28"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 16_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 4_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4d8dddd9769fcea91305afd9f96b9b187ad7dbd994a67cea4eeb7e2c0348c292"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 4_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0f4ea9e6bc6fce537e76838bafa08072aec839c4acc1d3a8c62bb4a253a0a451"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 4_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ec7fd7a922a87b641e3c8e0f2b092b1f470050c14409fcd95985c07024a429f4"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 4_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_512_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c3f35acbdbda16dc35969a4b0c817b2a7c9f8b037ace72cae4efb76797d8d4c4"
    ));

    let expected_value: BitList<U<512>> = BitList::from_bytes(
        [0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 64_u8]
            .to_vec()
            .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<512>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<512>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        104_u8, 108_u8, 147_u8, 82_u8, 247_u8, 10_u8, 41_u8, 236_u8, 81_u8, 255_u8, 119_u8, 145_u8,
        12_u8, 63_u8, 124_u8, 131_u8, 21_u8, 211_u8, 196_u8, 125_u8, 161_u8, 242_u8, 222_u8,
        193_u8, 156_u8, 133_u8, 68_u8, 174_u8, 17_u8, 238_u8, 8_u8, 37_u8, 152_u8, 152_u8, 73_u8,
        25_u8, 20_u8, 138_u8, 193_u8, 14_u8, 66_u8, 255_u8, 230_u8, 219_u8, 249_u8, 201_u8, 221_u8,
        230_u8, 58_u8, 9_u8, 88_u8, 144_u8, 122_u8, 242_u8, 200_u8, 100_u8, 120_u8, 253_u8, 195_u8,
        182_u8, 78_u8, 240_u8, 62_u8, 161_u8, 3_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "77184930e328732d5413240f6114e269a9df6573d8b177f03d328eda7d3ffae2"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            104_u8, 108_u8, 147_u8, 82_u8, 247_u8, 10_u8, 41_u8, 236_u8, 81_u8, 255_u8, 119_u8,
            145_u8, 12_u8, 63_u8, 124_u8, 131_u8, 21_u8, 211_u8, 196_u8, 125_u8, 161_u8, 242_u8,
            222_u8, 193_u8, 156_u8, 133_u8, 68_u8, 174_u8, 17_u8, 238_u8, 8_u8, 37_u8, 152_u8,
            152_u8, 73_u8, 25_u8, 20_u8, 138_u8, 193_u8, 14_u8, 66_u8, 255_u8, 230_u8, 219_u8,
            249_u8, 201_u8, 221_u8, 230_u8, 58_u8, 9_u8, 88_u8, 144_u8, 122_u8, 242_u8, 200_u8,
            100_u8, 120_u8, 253_u8, 195_u8, 182_u8, 78_u8, 240_u8, 62_u8, 161_u8, 3_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        57_u8, 91_u8, 36_u8, 187_u8, 64_u8, 156_u8, 244_u8, 32_u8, 187_u8, 84_u8, 133_u8, 47_u8,
        148_u8, 184_u8, 58_u8, 241_u8, 3_u8, 82_u8, 62_u8, 202_u8, 39_u8, 171_u8, 189_u8, 7_u8,
        100_u8, 10_u8, 34_u8, 56_u8, 58_u8, 210_u8, 187_u8, 14_u8, 160_u8, 147_u8, 70_u8, 195_u8,
        165_u8, 234_u8, 63_u8, 243_u8, 142_u8, 222_u8, 152_u8, 229_u8, 18_u8, 60_u8, 152_u8,
        242_u8, 150_u8, 6_u8, 32_u8, 123_u8, 82_u8, 162_u8, 21_u8, 224_u8, 58_u8, 228_u8, 227_u8,
        130_u8, 41_u8, 232_u8, 50_u8, 185_u8, 2_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "10041d4cf07da1077e84c9b5c01fa6d5f29ba8feb934ebdf7ca184a2857cdf55"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            57_u8, 91_u8, 36_u8, 187_u8, 64_u8, 156_u8, 244_u8, 32_u8, 187_u8, 84_u8, 133_u8,
            47_u8, 148_u8, 184_u8, 58_u8, 241_u8, 3_u8, 82_u8, 62_u8, 202_u8, 39_u8, 171_u8,
            189_u8, 7_u8, 100_u8, 10_u8, 34_u8, 56_u8, 58_u8, 210_u8, 187_u8, 14_u8, 160_u8,
            147_u8, 70_u8, 195_u8, 165_u8, 234_u8, 63_u8, 243_u8, 142_u8, 222_u8, 152_u8, 229_u8,
            18_u8, 60_u8, 152_u8, 242_u8, 150_u8, 6_u8, 32_u8, 123_u8, 82_u8, 162_u8, 21_u8,
            224_u8, 58_u8, 228_u8, 227_u8, 130_u8, 41_u8, 232_u8, 50_u8, 185_u8, 2_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        33_u8, 218_u8, 166_u8, 15_u8, 249_u8, 225_u8, 154_u8, 205_u8, 6_u8, 215_u8, 100_u8, 77_u8,
        14_u8, 16_u8, 207_u8, 196_u8, 191_u8, 216_u8, 118_u8, 102_u8, 63_u8, 223_u8, 211_u8, 47_u8,
        81_u8, 105_u8, 148_u8, 151_u8, 230_u8, 101_u8, 199_u8, 64_u8, 63_u8, 199_u8, 177_u8,
        176_u8, 74_u8, 254_u8, 199_u8, 162_u8, 209_u8, 75_u8, 55_u8, 176_u8, 61_u8, 59_u8, 169_u8,
        236_u8, 252_u8, 244_u8, 75_u8, 182_u8, 82_u8, 229_u8, 252_u8, 234_u8, 204_u8, 44_u8,
        217_u8, 9_u8, 45_u8, 153_u8, 103_u8, 197_u8, 2_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "fcc1fb245d5eae1370c4cfaf51a23a68d24fc931eb75d8e3b337eadf1c94b4be"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            33_u8, 218_u8, 166_u8, 15_u8, 249_u8, 225_u8, 154_u8, 205_u8, 6_u8, 215_u8, 100_u8,
            77_u8, 14_u8, 16_u8, 207_u8, 196_u8, 191_u8, 216_u8, 118_u8, 102_u8, 63_u8, 223_u8,
            211_u8, 47_u8, 81_u8, 105_u8, 148_u8, 151_u8, 230_u8, 101_u8, 199_u8, 64_u8, 63_u8,
            199_u8, 177_u8, 176_u8, 74_u8, 254_u8, 199_u8, 162_u8, 209_u8, 75_u8, 55_u8, 176_u8,
            61_u8, 59_u8, 169_u8, 236_u8, 252_u8, 244_u8, 75_u8, 182_u8, 82_u8, 229_u8, 252_u8,
            234_u8, 204_u8, 44_u8, 217_u8, 9_u8, 45_u8, 153_u8, 103_u8, 197_u8, 2_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        244_u8, 116_u8, 125_u8, 246_u8, 185_u8, 122_u8, 37_u8, 121_u8, 40_u8, 135_u8, 61_u8, 96_u8,
        114_u8, 88_u8, 163_u8, 166_u8, 254_u8, 68_u8, 177_u8, 16_u8, 10_u8, 122_u8, 24_u8, 95_u8,
        35_u8, 204_u8, 170_u8, 225_u8, 207_u8, 108_u8, 158_u8, 179_u8, 248_u8, 186_u8, 81_u8,
        253_u8, 120_u8, 163_u8, 148_u8, 221_u8, 49_u8, 253_u8, 102_u8, 4_u8, 241_u8, 70_u8, 171_u8,
        228_u8, 149_u8, 244_u8, 237_u8, 214_u8, 195_u8, 11_u8, 122_u8, 51_u8, 158_u8, 191_u8,
        206_u8, 5_u8, 207_u8, 44_u8, 43_u8, 160_u8, 3_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "eb5acc36387e3d3e44187bd6c086e4409fab204daa33ad40a99226dd2c487d8e"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            244_u8, 116_u8, 125_u8, 246_u8, 185_u8, 122_u8, 37_u8, 121_u8, 40_u8, 135_u8, 61_u8,
            96_u8, 114_u8, 88_u8, 163_u8, 166_u8, 254_u8, 68_u8, 177_u8, 16_u8, 10_u8, 122_u8,
            24_u8, 95_u8, 35_u8, 204_u8, 170_u8, 225_u8, 207_u8, 108_u8, 158_u8, 179_u8, 248_u8,
            186_u8, 81_u8, 253_u8, 120_u8, 163_u8, 148_u8, 221_u8, 49_u8, 253_u8, 102_u8, 4_u8,
            241_u8, 70_u8, 171_u8, 228_u8, 149_u8, 244_u8, 237_u8, 214_u8, 195_u8, 11_u8, 122_u8,
            51_u8, 158_u8, 191_u8, 206_u8, 5_u8, 207_u8, 44_u8, 43_u8, 160_u8, 3_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        61_u8, 93_u8, 109_u8, 239_u8, 248_u8, 61_u8, 228_u8, 130_u8, 190_u8, 124_u8, 243_u8, 93_u8,
        119_u8, 224_u8, 242_u8, 90_u8, 116_u8, 204_u8, 103_u8, 69_u8, 123_u8, 151_u8, 6_u8, 222_u8,
        218_u8, 77_u8, 32_u8, 82_u8, 17_u8, 10_u8, 27_u8, 133_u8, 187_u8, 42_u8, 87_u8, 77_u8,
        193_u8, 194_u8, 215_u8, 216_u8, 128_u8, 136_u8, 229_u8, 169_u8, 43_u8, 182_u8, 75_u8,
        81_u8, 98_u8, 190_u8, 45_u8, 113_u8, 149_u8, 97_u8, 252_u8, 45_u8, 140_u8, 150_u8, 133_u8,
        12_u8, 255_u8, 125_u8, 96_u8, 222_u8, 3_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4edc0e0f8cb3511f8e89e5a9d73fdd50270e49aa8bfa62ffe8c8e99c161e76ba"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            61_u8, 93_u8, 109_u8, 239_u8, 248_u8, 61_u8, 228_u8, 130_u8, 190_u8, 124_u8, 243_u8,
            93_u8, 119_u8, 224_u8, 242_u8, 90_u8, 116_u8, 204_u8, 103_u8, 69_u8, 123_u8, 151_u8,
            6_u8, 222_u8, 218_u8, 77_u8, 32_u8, 82_u8, 17_u8, 10_u8, 27_u8, 133_u8, 187_u8, 42_u8,
            87_u8, 77_u8, 193_u8, 194_u8, 215_u8, 216_u8, 128_u8, 136_u8, 229_u8, 169_u8, 43_u8,
            182_u8, 75_u8, 81_u8, 98_u8, 190_u8, 45_u8, 113_u8, 149_u8, 97_u8, 252_u8, 45_u8,
            140_u8, 150_u8, 133_u8, 12_u8, 255_u8, 125_u8, 96_u8, 222_u8, 3_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "3116c9a3fab7c6ebf0978f8ef07aa2c27ea9c79887d773980a39b95e5c035593"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 7_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[127_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "b00f282b126680bcbd302d657b117dc32294c4cb586f76c244932141012e6a82"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([127_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "848557322ff06141bbb7ac657b15c24e6300986a5ff8ce878ef4b198c0bd51b0"
    ));

    let expected_value: BitList<U<513>> =
        BitList::from_bytes([255_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 31_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "a575735c9960d438c8bdd59d05fedefce22f8e5b77b09efb5b4e9942b847468e"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 31_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
        255_u8, 255_u8, 255_u8, 255_u8, 1_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "e05d10ac23b945573dca5263c13a7eaf50854397bf48f920175a10509bf65ecf"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8, 255_u8,
            255_u8, 255_u8, 255_u8, 255_u8, 1_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "28ba1834a3a7b657460ce79fa3a1d909ab8828fd557659d4d0554a9bdbc0ec30"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        176_u8, 215_u8, 228_u8, 180_u8, 55_u8, 102_u8, 22_u8, 56_u8, 171_u8, 0_u8, 210_u8, 26_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "62110ea980c0e8b321149e2681d66a3c9ca6d2af615ed3f7b2ea1f950519cee3"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            176_u8, 215_u8, 228_u8, 180_u8, 55_u8, 102_u8, 22_u8, 56_u8, 171_u8, 0_u8, 210_u8,
            26_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        245_u8, 161_u8, 103_u8, 135_u8, 184_u8, 153_u8, 184_u8, 107_u8, 212_u8, 115_u8, 240_u8,
        116_u8, 67_u8, 139_u8, 183_u8, 180_u8, 192_u8, 56_u8, 42_u8, 59_u8, 93_u8, 140_u8, 253_u8,
        38_u8, 235_u8, 169_u8, 190_u8, 107_u8, 72_u8, 235_u8, 179_u8, 66_u8, 239_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "339f84a3e78443af74c3ea49f06c6d1933f3b4e3440dc631820662651085a306"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            245_u8, 161_u8, 103_u8, 135_u8, 184_u8, 153_u8, 184_u8, 107_u8, 212_u8, 115_u8, 240_u8,
            116_u8, 67_u8, 139_u8, 183_u8, 180_u8, 192_u8, 56_u8, 42_u8, 59_u8, 93_u8, 140_u8,
            253_u8, 38_u8, 235_u8, 169_u8, 190_u8, 107_u8, 72_u8, 235_u8, 179_u8, 66_u8, 239_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[
        186_u8, 79_u8, 229_u8, 99_u8, 225_u8, 114_u8, 253_u8, 235_u8, 154_u8, 116_u8, 179_u8,
        60_u8, 152_u8, 109_u8, 119_u8, 170_u8, 93_u8, 63_u8, 174_u8, 162_u8, 177_u8, 182_u8, 89_u8,
        12_u8, 145_u8, 140_u8, 95_u8, 44_u8, 142_u8, 239_u8, 49_u8, 74_u8, 73_u8, 181_u8, 146_u8,
        240_u8, 65_u8, 166_u8, 251_u8, 121_u8, 119_u8, 100_u8, 3_u8, 106_u8, 2_u8, 88_u8, 57_u8,
        197_u8, 1_u8, 244_u8, 98_u8, 134_u8, 222_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "bb76eb1bab23fc2865c84717251e4305221771924259082d793d3bbaa6444ba1"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            186_u8, 79_u8, 229_u8, 99_u8, 225_u8, 114_u8, 253_u8, 235_u8, 154_u8, 116_u8, 179_u8,
            60_u8, 152_u8, 109_u8, 119_u8, 170_u8, 93_u8, 63_u8, 174_u8, 162_u8, 177_u8, 182_u8,
            89_u8, 12_u8, 145_u8, 140_u8, 95_u8, 44_u8, 142_u8, 239_u8, 49_u8, 74_u8, 73_u8,
            181_u8, 146_u8, 240_u8, 65_u8, 166_u8, 251_u8, 121_u8, 119_u8, 100_u8, 3_u8, 106_u8,
            2_u8, 88_u8, 57_u8, 197_u8, 1_u8, 244_u8, 98_u8, 134_u8, 222_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[218_u8, 86_u8, 210_u8, 218_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "32370b95731ef776a513ca5ef154a83ba935260f2f4bdbba23c21b33e12f7b62"
    ));

    let expected_value: BitList<U<513>> =
        BitList::from_bytes([218_u8, 86_u8, 210_u8, 218_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        104_u8, 76_u8, 185_u8, 82_u8, 136_u8, 235_u8, 228_u8, 30_u8, 187_u8, 193_u8, 232_u8, 34_u8,
        2_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "ba02d7073304a825d35943f503cb081434b0b49713afdff5b5a6ab1f46d14171"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            104_u8, 76_u8, 185_u8, 82_u8, 136_u8, 235_u8, 228_u8, 30_u8, 187_u8, 193_u8, 232_u8,
            34_u8, 2_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 2_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "3f398072fb9acafba24683799d8250de322a96a12e3016134220db24526b372d"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 2_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 4_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "25f3b33649409489b22232a7706a5ae5c4f5b62cadee098a758d3fa16d1087d2"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 4_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[0_u8, 0_u8, 0_u8, 0_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "38ab4aeb5726a3fb78af0101063f2586905c3e8466206bfc8777f44ed9e6ef20"
    ));

    let expected_value: BitList<U<513>> =
        BitList::from_bytes([0_u8, 0_u8, 0_u8, 0_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 4_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "54bfe2c647e52bf3897cff9675165d53f277e1f7dbd7c620f630a2deb02ce0c8"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 4_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_513_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
        0_u8, 0_u8, 0_u8, 0_u8, 64_u8,
    ];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "9b7d4ffa3720c8ea2c66e59f1890a83c86ef2b4442a5ebe6d757fb4cbe0b3231"
    ));

    let expected_value: BitList<U<513>> = BitList::from_bytes(
        [
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8,
            0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 0_u8, 64_u8,
        ]
        .to_vec()
        .into(),
    )
    .unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<513>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<513>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[36_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "bd50456d5ad175ae99a1612a53ca229124b65d3eaabd9ff9c7ab979a385cf6b3"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([36_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[57_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7000b9bd26fb753d24a4ed870faee659894843b795377a89ade25b649246e773"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([57_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[34_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d13061c7b549c86b29ad2389cbe4fb2fd05bbdf3170da634e67f77ab981b82cb"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([34_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[58_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "5d40a4acd8c5f8b674c29a7b7814a546fade497a96d0e7bb51c3a4951fb1fa7e"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([58_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[48_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "88b744d02033bbb6a4ebc2dc3f31c4910681596c7bcb9349d9483a33e45899c7"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([48_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4b07c3799db025f3aa92ced1e8545367a2b6e44960f479d3f9d62b61812892d5"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[31_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "4b07c3799db025f3aa92ced1e8545367a2b6e44960f479d3f9d62b61812892d5"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([31_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[7_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "c397e31994d6b872c69af43765ab16a1cef673be726a820dacd2637bea2f5fbb"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([7_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[8_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d86ae2ca925345bf2412bde450ac175742d979c1ea7b961bd1efe10beb9500cf"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([8_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[3_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "56d8a66fbae0300efba7ec2c531973aaae22e7a2ed6ded081b5b32d07a32780a"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([3_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[6_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "0e01f8d9a6720610a44a70c2c91bbe750ec6cd67892d92b1016394abfc382cf9"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([6_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[22_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "894e8a2ce460c6c6ba12d467634e6c34ce2a1b58d0c6dfe3d98b532898c58611"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([22_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[32_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "16aaf795af421b6156d4c3319879d422a0c3ffd26db07207a54d6cafcbef0b10"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([32_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d647eb2598d33d7216256356596d29cecd31c1ba7a7ff25ccb5be4a453410b9d"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_5_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d647eb2598d33d7216256356596d29cecd31c1ba7a7ff25ccb5be4a453410b9d"
    ));

    let expected_value: BitList<U<5>> = BitList::from_bytes([16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<5>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<5>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_lengthy_0() {
    const EXPECTED_ENCODING: &[u8] = &[206_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "095847dd477b5ac2b2a5930d0633975f09e835630c2d4a832b6469e8c0d106d1"
    ));

    let expected_value: BitList<U<8>> =
        BitList::from_bytes([206_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_lengthy_1() {
    const EXPECTED_ENCODING: &[u8] = &[180_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "5b6af4c3df02247b90fc3736e0a2ff746b5a7f7dc54e7edc66bbb0f68f1b7206"
    ));

    let expected_value: BitList<U<8>> =
        BitList::from_bytes([180_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_lengthy_2() {
    const EXPECTED_ENCODING: &[u8] = &[83_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "eeb7a380c63f2182c38a556ee4170cb9fd06b86b5014181e7a01ce0097627cf0"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([83_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_lengthy_3() {
    const EXPECTED_ENCODING: &[u8] = &[99_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "b8148b13b48faa79622d9a6975e7abdf85dd4639a25e53412eb0aa5c34386019"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([99_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_lengthy_4() {
    const EXPECTED_ENCODING: &[u8] = &[227_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "6d1fd4c1b192e8aeb35074214855c593805c2ed1ff79f7aa7c6128814fa41bf3"
    ));

    let expected_value: BitList<U<8>> =
        BitList::from_bytes([227_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_max_0() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "017d2fa0f6934ed2354e4cdb7a2230ccf8f31fe758c7a47442e37fdea1d68bfe"
    ));

    let expected_value: BitList<U<8>> =
        BitList::from_bytes([255_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_max_1() {
    const EXPECTED_ENCODING: &[u8] = &[255_u8, 1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "017d2fa0f6934ed2354e4cdb7a2230ccf8f31fe758c7a47442e37fdea1d68bfe"
    ));

    let expected_value: BitList<U<8>> =
        BitList::from_bytes([255_u8, 1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_max_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_max_3() {
    const EXPECTED_ENCODING: &[u8] = &[63_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb9e73cb5c2e4ef66fa63540f8220301d31eea7edfccedb2b47b9bdf849ccee7"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([63_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_max_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_nil_0() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_nil_1() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_nil_2() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_nil_3() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_nil_4() {
    const EXPECTED_ENCODING: &[u8] = &[1_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "f5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([1_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_random_0() {
    const EXPECTED_ENCODING: &[u8] = &[79_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "385e8de0fb7865579bcaf9d0a9c86e4cca08a6911d1ce85530f96ce202a38d21"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([79_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_random_1() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_random_2() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_random_3() {
    const EXPECTED_ENCODING: &[u8] = &[2_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "cb592844121d926f1ca3ad4e1d6fb9d8e260ed6e3216361f7732e975a0e8bbf6"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([2_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_random_4() {
    const EXPECTED_ENCODING: &[u8] = &[15_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "251d8bd955c85219bb8f6de682810b4aafe3e0c3d3c624020fb39f81dbb85910"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([15_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_zero_0() {
    const EXPECTED_ENCODING: &[u8] = &[64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7d360196d14b15261c9e5f576df8dc8b48d18d79b4198f167741052747704352"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_zero_1() {
    const EXPECTED_ENCODING: &[u8] = &[4_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "1205f4789155711e2542dba1a64d226626fe3eb43baa854752d0b59077e010fc"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([4_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_zero_2() {
    const EXPECTED_ENCODING: &[u8] = &[64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7d360196d14b15261c9e5f576df8dc8b48d18d79b4198f167741052747704352"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_zero_3() {
    const EXPECTED_ENCODING: &[u8] = &[64_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "7d360196d14b15261c9e5f576df8dc8b48d18d79b4198f167741052747704352"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([64_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
#[test]
fn valid_bitlist_8_zero_4() {
    const EXPECTED_ENCODING: &[u8] = &[16_u8];
    const EXPECTED_ROOT: H256 = <H256>::new(hex!(
        "d647eb2598d33d7216256356596d29cecd31c1ba7a7ff25ccb5be4a453410b9d"
    ));

    let expected_value: BitList<U<8>> = BitList::from_bytes([16_u8].to_vec().into()).unwrap();

    assert_eq!(
        expected_value,
        <BitList<U<8>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
    );

    assert_eq!(
        EXPECTED_ENCODING,
        <BitList<U<8>> as Ssz>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(EXPECTED_ROOT, <H256>::new(expected_value.tree_hash_root()));
}
