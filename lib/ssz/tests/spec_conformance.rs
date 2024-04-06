#![allow(non_camel_case_types)]

use std::{
    fmt::Debug,
    fs::read_dir,
    path::{Path, PathBuf},
};

use ::tree_hash::TreeHash;
use serde::Deserialize;
use ssz::types::FixedVector;
use typenum::{Const, NonZero, ToUInt, Unsigned, U};
use unionlabs::hash::H256;

// alias primitive types to their respective eth names, as that's whats used in the testdata folder names
type uint8 = u8;
type uint16 = u16;
type uint32 = u32;
type uint64 = u64;
type uint256 = unionlabs::uint::U256;

// uint128 needs a bit more than an alias since it's serialized as a string in the testdata yaml files
#[derive(Debug, PartialEq, Deserialize, ssz::Encode, ssz::Decode)]
#[serde(transparent)]
#[ssz(struct_behaviour = "transparent")]
struct uint128(#[serde(with = "::serde_utils::string")] u128);

impl TreeHash for uint128 {
    fn tree_hash_type() -> tree_hash::TreeHashType {
        u128::tree_hash_type()
    }

    fn tree_hash_packed_encoding(&self) -> tree_hash::PackedEncoding {
        self.0.tree_hash_packed_encoding()
    }

    fn tree_hash_packing_factor() -> usize {
        u128::tree_hash_packing_factor()
    }

    fn tree_hash_root(&self) -> tree_hash::Hash256 {
        self.0.tree_hash_root()
    }
}

#[test]
fn suite() {
    let testdata_dir = PathBuf::from(std::env::var("ETHEREUM_CONSENSUS_SPECS_DIR").unwrap())
        .join("general/phase0/ssz_generic/");

    // basic_vector  bitlist  bitvector  boolean  containers  uints

    let valid = read_dir(testdata_dir.join("basic_vector/valid")).unwrap();

    for dir in valid {
        let dir = dir.unwrap();

        let file = dir.file_name().to_str().unwrap().to_owned();
        let ts = file.split('_').collect::<Vec<_>>();

        macro_rules! mk_vector_tests {
            ($([$ty:ty, $len:literal])*) => {
                match &*ts {
                    $(
                        ["vec", stringify!($ty), stringify!($len), ..] => {
                            println!("testing {file}...");
                            variable_list_test::<$ty, $len>(&dir.path());
                        }
                    )*
                    _ => {
                        println!("{ts:?}");

                    }
                }
            };
        }

        mk_vector_tests! {
            [bool, 1]
            [bool, 2]
            [bool, 3]
            [bool, 4]
            [bool, 5]
            [bool, 8]
            [bool, 16]
            [bool, 31]
            [bool, 512]
            [bool, 513]

            [uint8, 1]
            [uint8, 2]
            [uint8, 3]
            [uint8, 4]
            [uint8, 5]
            [uint8, 8]
            [uint8, 16]
            [uint8, 31]
            [uint8, 512]
            [uint8, 513]

            [uint16, 1]
            [uint16, 2]
            [uint16, 3]
            [uint16, 4]
            [uint16, 5]
            [uint16, 8]
            [uint16, 16]
            [uint16, 31]
            [uint16, 512]
            [uint16, 513]

            [uint32, 1]
            [uint32, 2]
            [uint32, 3]
            [uint32, 4]
            [uint32, 5]
            [uint32, 8]
            [uint32, 16]
            [uint32, 31]
            [uint32, 512]
            [uint32, 513]

            [uint64, 1]
            [uint64, 2]
            [uint64, 3]
            [uint64, 4]
            [uint64, 5]
            [uint64, 8]
            [uint64, 16]
            [uint64, 31]
            [uint64, 512]
            [uint64, 513]

            [uint128, 1]
            [uint128, 2]
            [uint128, 3]
            [uint128, 4]
            [uint128, 5]
            [uint128, 8]
            [uint128, 16]
            [uint128, 31]
            [uint128, 512]
            [uint128, 513]

            [uint256, 1]
            [uint256, 2]
            [uint256, 3]
            [uint256, 4]
            [uint256, 5]
            [uint256, 8]
            [uint256, 16]
            [uint256, 31]
            [uint256, 512]
            [uint256, 513]
        }
    }

    let invalid = read_dir(testdata_dir.join("basic_vector/invalid")).unwrap();

    for dir in invalid {
        let dir = dir.unwrap();

        let file = dir.file_name().to_str().unwrap().to_owned();
        let ts = file.split('_').collect::<Vec<_>>();

        macro_rules! mk_vector_tests {
            ($([$ty:ty, $len:literal])*) => {
                match &*ts {
                    $(
                        ["vec", stringify!($ty), stringify!($len), ..] => {
                            println!("testing {file}...");
                            <FixedVector<$ty, U<$len>> as ssz::Decode>::from_ssz_bytes(&read_snappy_file(dir.path().join("serialized.ssz_snappy"))).unwrap_err();
                        }
                    )*
                    ["vec", _, "0", ..] => {
                        println!("not testing {file}, as a length of 0 would not compile");
                    }
                    _ => {
                        println!("{ts:?}");

                    }
                }
            };
        }

        mk_vector_tests! {
            [bool, 1]
            [bool, 2]
            [bool, 3]
            [bool, 4]
            [bool, 5]
            [bool, 8]
            [bool, 16]
            [bool, 31]
            [bool, 512]
            [bool, 513]

            [uint8, 1]
            [uint8, 2]
            [uint8, 3]
            [uint8, 4]
            [uint8, 5]
            [uint8, 8]
            [uint8, 16]
            [uint8, 31]
            [uint8, 512]
            [uint8, 513]

            [uint16, 1]
            [uint16, 2]
            [uint16, 3]
            [uint16, 4]
            [uint16, 5]
            [uint16, 8]
            [uint16, 16]
            [uint16, 31]
            [uint16, 512]
            [uint16, 513]

            [uint32, 1]
            [uint32, 2]
            [uint32, 3]
            [uint32, 4]
            [uint32, 5]
            [uint32, 8]
            [uint32, 16]
            [uint32, 31]
            [uint32, 512]
            [uint32, 513]

            [uint64, 1]
            [uint64, 2]
            [uint64, 3]
            [uint64, 4]
            [uint64, 5]
            [uint64, 8]
            [uint64, 16]
            [uint64, 31]
            [uint64, 512]
            [uint64, 513]

            [uint128, 1]
            [uint128, 2]
            [uint128, 3]
            [uint128, 4]
            [uint128, 5]
            [uint128, 8]
            [uint128, 16]
            [uint128, 31]
            [uint128, 512]
            [uint128, 513]

            [uint256, 1]
            [uint256, 2]
            [uint256, 3]
            [uint256, 4]
            [uint256, 5]
            [uint256, 8]
            [uint256, 16]
            [uint256, 31]
            [uint256, 512]
            [uint256, 513]
        }
    }
}

#[derive(Deserialize)]
struct Meta {
    root: H256,
}

fn variable_list_test<T, const N: usize>(path: &Path)
where
    T: serde::de::DeserializeOwned
        + ssz::Encode
        + ssz::Decode
        + ::tree_hash::TreeHash
        + PartialEq
        + Debug,
    Const<N>: ToUInt,
    U<N>: Unsigned + NonZero,
{
    let expected_encoding = read_snappy_file(path.join("serialized.ssz_snappy"));
    let expected_value: FixedVector<T, U<N>> =
        serde_yaml::from_str::<Vec<T>>(&std::fs::read_to_string(path.join("value.yaml")).unwrap())
            .unwrap()
            .try_into()
            .unwrap();
    let expected_root =
        serde_yaml::from_str::<Meta>(&std::fs::read_to_string(path.join("meta.yaml")).unwrap())
            .unwrap();

    assert_eq!(
        expected_value,
        <FixedVector<T, U<N>> as ssz::Decode>::from_ssz_bytes(&expected_encoding).unwrap()
    );

    assert_eq!(
        expected_encoding,
        <FixedVector<T, U<N>> as ssz::Encode>::as_ssz_bytes(&expected_value)
    );

    assert_eq!(expected_root.root, expected_value.tree_hash_root().into());
}

fn read_snappy_file(path: impl AsRef<Path>) -> Vec<u8> {
    let mut decoder = snap::raw::Decoder::new();
    decoder
        .decompress_vec(&std::fs::read(path).expect("path exists"))
        .expect("snappy decoding should succeed")
}
