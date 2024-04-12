#![allow(non_camel_case_types)]

use std::{
    collections::BTreeMap,
    fmt::Write,
    fs::{self, read_dir},
    path::{Path, PathBuf},
};

use hex::ToHex;
use serde::{de::DeserializeOwned, Deserialize};
use ssz::types::{BitList, BitVector, List, Vector};
use typenum::{NonZero, Unsigned};
use unionlabs::{hash::H256, uint::U256};

// alias primitive types to their respective eth names, as that's whats used in the testdata folder names
type uint8 = u8;
type uint16 = u16;
type uint32 = u32;
type uint64 = u64;
type uint256 = U256;

pub mod container_types;
use container_types::*;

fn main() {
    let [ethereum_consensus_specs_dir, spec_conformance_out_file, out_dir] = std::env::args_os()
        .skip(1)
        .map(PathBuf::from)
        .collect::<Vec<_>>()
        .try_into()
        .expect("expected two arguments [ethereum_consensus_specs_dir, spec_conformance_out_file, out_dir]");

    let testdata_dir = ethereum_consensus_specs_dir.join("general/phase0/ssz_generic/");

    let mods = [
        basic_vector(&testdata_dir, &out_dir),
        bitlist(&testdata_dir, &out_dir),
        bitvector(&testdata_dir, &out_dir),
        boolean(&testdata_dir, &out_dir),
        uint(&testdata_dir, &out_dir),
        container(&testdata_dir, &out_dir),
    ]
    .iter()
    .fold(String::new(), |mut s, [a, b]| {
        writeln!(s, "mod {a}; mod {b};").unwrap();
        s
    });

    fs::write(
        spec_conformance_out_file,
        format!("mod spec_conformance {{ {mods} }}"),
    )
    .unwrap();
}

#[must_use]
fn write_mod(
    testdata_dir: &Path,
    out_dir: &Path,
    mod_name: &str,
    valid_f: impl FnMut(&[&str], Vec<u8>, serde_yaml::Value, Meta) -> String,
    invalid_f: impl FnMut(&[&str], Vec<u8>) -> String,
) -> [String; 2] {
    let valid_mod = format!("{mod_name}_valid");
    fs::write(
        out_dir.join(format!("{valid_mod}.rs")),
        format!("{HEADER} {}", read_valid(testdata_dir, mod_name, valid_f)),
    )
    .unwrap();

    let invalid_mod = format!("{mod_name}_invalid");
    fs::write(
        out_dir.join(format!("{invalid_mod}.rs")),
        format!(
            "{HEADER} {}",
            read_invalid(testdata_dir, mod_name, invalid_f)
        ),
    )
    .unwrap();

    [valid_mod, invalid_mod]
}

// uint128 needs a bit more than an alias since it's serialized as a string in the testdata yaml files
#[derive(Debug, PartialEq, Deserialize)]
#[serde(transparent)]
struct uint128(#[serde(with = "::serde_utils::string")] u128);

const HEADER: &str = r#"
#![allow(non_snake_case, unused_imports)]

use hex_literal::hex;
use ssz::types::*;
use ssz::Ssz;
use unionlabs::uint::U256;
use unionlabs::hash::H256;
use typenum::U;

pub mod container_types {
    include!("../../tests-generator/src/container_types.rs");
}
use container_types::*;
"#;

// from yaml
#[track_caller]
fn fy<T: DeserializeOwned>(value: serde_yaml::Value) -> T {
    serde_yaml::from_value(value).unwrap()
}

fn read_valid(
    testdata_dir: &Path,
    dir: &str,
    // ts, expected_encoding, expected_value, meta
    mut f: impl FnMut(&[&str], Vec<u8>, serde_yaml::Value, Meta) -> String,
) -> String {
    let valid = read_dir(testdata_dir.join(format!("{dir}/valid"))).unwrap();

    let mut valid_map = BTreeMap::new();

    for dir in valid {
        let dir = dir.unwrap();

        let file = dir.file_name().to_str().unwrap().to_owned();
        let ts = file.split('_').collect::<Vec<_>>();

        let path = dir.path();

        let expected_encoding = read_snappy_file(&path);
        let expected_value = read_value::<serde_yaml::Value>(&path);
        let meta = read_meta(&path);

        valid_map.insert(
            file.clone(),
            f(&ts, expected_encoding, expected_value, meta),
        );
    }

    valid_map
        .into_iter()
        .fold(String::new(), |mut s, (name, body)| {
            writeln!(s, "#[test] fn valid_{name}() {{ {body} }}").unwrap();
            s
        })
}

fn read_invalid(
    testdata_dir: &Path,
    dir: &str,
    // ts, invalid_encoding
    mut f: impl FnMut(&[&str], Vec<u8>) -> String,
) -> String {
    let invalid = read_dir(testdata_dir.join(format!("{dir}/invalid"))).unwrap();

    let mut invalid_map = BTreeMap::new();

    for dir in invalid {
        let dir = dir.unwrap();

        let file = dir.file_name().to_str().unwrap().to_owned();
        let ts = file.split('_').collect::<Vec<_>>();

        let path = dir.path();

        let invalid_encoding = read_snappy_file(&path);

        invalid_map.insert(file.clone(), f(&ts, invalid_encoding));
    }

    invalid_map
        .into_iter()
        .fold(String::new(), |mut s, (name, body)| {
            writeln!(s, "#[test] fn invalid_{name}() {{ {body} }}").unwrap();
            s
        })
}

fn basic_vector(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "basic_vector",
        |ts, expected_encoding, expected_value, meta| {
            macro_rules! mk_vector_tests {
                ($($ty:ty)*) => {
                    match ts {
                        $(
                            ["vec", stringify!($ty), len, ..] => {
                                basic_vector_valid::<$ty>(len.parse().unwrap(), expected_encoding, fy(expected_value), meta)
                            }
                        )*
                        _ => {
                            panic!("unhandled: {ts:?}");
                        }
                    }
                };
            }

            mk_vector_tests!(bool uint8 uint16 uint32 uint64 uint128 uint256)
        },
        |ts, invalid_encoding| {
            macro_rules! mk_vector_tests {
                ($($ty:ty)*) => {
                    match ts {
                        $(
                            ["vec", stringify!($ty), len, ..] => {
                                basic_vector_invalid::<$ty>(len.parse().unwrap(), invalid_encoding)
                            }
                        )*
                        _ => {
                            panic!("unhandled: {ts:?}");
                        }
                    }
                };
            }
            mk_vector_tests!(bool uint8 uint16 uint32 uint64 uint128 uint256)
        },
    )
}

fn bitlist(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "bitlist",
        |ts, expected_encoding, expected_value, meta| match ts {
            ["bitlist", len, ..] => bitlist_valid(
                len.parse().unwrap(),
                expected_encoding,
                fy(expected_value),
                meta,
            ),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
        |ts, invalid_encoding| match ts {
            ["bitlist", _, ..] => bitlist_invalid(1, invalid_encoding),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
    )
}

fn bitvector(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "bitvector",
        |ts, expected_encoding, expected_value, meta| match ts {
            ["bitvec", len, ..] => bitvector_valid(
                len.parse().unwrap(),
                expected_encoding,
                fy(expected_value),
                meta,
            ),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
        |ts, invalid_encoding| match ts {
            ["bitvec", len, ..] => bitvector_invalid(len.parse().unwrap(), invalid_encoding),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
    )
}

fn boolean(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "boolean",
        |ts, expected_encoding, expected_value, meta| match ts {
            [b] => boolean_valid(
                b.parse().unwrap(),
                expected_encoding,
                fy(expected_value),
                meta,
            ),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
        |ts, invalid_encoding| match ts {
            ["byte", _, ..] => boolean_invalid(invalid_encoding),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
    )
}

fn uint(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "uints",
        |ts, expected_encoding, expected_value, meta| match ts {
            ["uint", "8", ..] => uint_valid::<u8>(expected_encoding, fy(expected_value), meta),
            ["uint", "16", ..] => uint_valid::<u16>(expected_encoding, fy(expected_value), meta),
            ["uint", "32", ..] => uint_valid::<u32>(expected_encoding, fy(expected_value), meta),
            ["uint", "64", ..] => uint_valid::<u64>(expected_encoding, fy(expected_value), meta),
            ["uint", "128", ..] => {
                uint_valid::<uint128>(expected_encoding, fy(expected_value), meta)
            }
            ["uint", "256", ..] => uint_valid::<U256>(expected_encoding, fy(expected_value), meta),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
        |ts, invalid_encoding| match ts {
            ["uint", "8", ..] => uint_invalid::<u8>(invalid_encoding),
            ["uint", "16", ..] => uint_invalid::<u16>(invalid_encoding),
            ["uint", "32", ..] => uint_invalid::<u32>(invalid_encoding),
            ["uint", "64", ..] => uint_invalid::<u64>(invalid_encoding),
            ["uint", "128", ..] => uint_invalid::<uint128>(invalid_encoding),
            ["uint", "256", ..] => uint_invalid::<U256>(invalid_encoding),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
    )
}

fn container(testdata_dir: &Path, out_dir: &Path) -> [String; 2] {
    write_mod(
        testdata_dir,
        out_dir,
        "containers",
        |ts, expected_encoding, expected_value, meta| match ts {
            ["SingleFieldTestStruct", ..] => container_valid::<SingleFieldTestStruct>(
                expected_encoding,
                fy(expected_value),
                meta,
            ),
            ["SmallTestStruct", ..] => {
                container_valid::<SmallTestStruct>(expected_encoding, fy(expected_value), meta)
            }
            ["FixedTestStruct", ..] => {
                container_valid::<FixedTestStruct>(expected_encoding, fy(expected_value), meta)
            }
            ["VarTestStruct", ..] => {
                container_valid::<VarTestStruct>(expected_encoding, fy(expected_value), meta)
            }
            ["ComplexTestStruct", ..] => {
                container_valid::<ComplexTestStruct>(expected_encoding, fy(expected_value), meta)
            }
            ["BitsStruct", ..] => {
                container_valid::<BitsStruct>(expected_encoding, fy(expected_value), meta)
            }
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
        |ts, invalid_encoding| match ts {
            ["SingleFieldTestStruct", ..] => {
                container_invalid::<SingleFieldTestStruct>(invalid_encoding)
            }
            ["SmallTestStruct", ..] => container_invalid::<SmallTestStruct>(invalid_encoding),
            ["FixedTestStruct", ..] => container_invalid::<FixedTestStruct>(invalid_encoding),
            ["VarTestStruct", ..] => container_invalid::<VarTestStruct>(invalid_encoding),
            ["ComplexTestStruct", ..] => container_invalid::<ComplexTestStruct>(invalid_encoding),
            ["BitsStruct", ..] => container_invalid::<BitsStruct>(invalid_encoding),
            _ => {
                panic!("unhandled: {ts:?}");
            }
        },
    )
}

#[derive(Deserialize)]
struct Meta {
    root: H256,
}

fn basic_vector_valid<T>(
    expected_len: usize,
    expected_encoding: Vec<u8>,
    expected_value: Vec<T>,
    meta: Meta,
) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    let type_name = T::type_name();
    let n = expected_value.len();

    assert_eq!(n, expected_len);

    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            {{
                let expected_value: Vector<{type_name}, U<{n}>> = {expected_value}.into();

                assert_eq!(
                    expected_value,
                    <Vector<{type_name}, U<{n}>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
                );

                assert_eq!(
                    EXPECTED_ENCODING,
                    <Vector<{type_name}, U<{n}>> as Ssz>::as_ssz_bytes(&expected_value)
                );

                assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
            }}

            {{
                let expected_value: [{type_name}; {n}] = {expected_value};

                assert_eq!(
                    expected_value,
                    <[{type_name}; {n}] as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
                );

                assert_eq!(
                    EXPECTED_ENCODING,
                    <[{type_name}; {n}] as Ssz>::as_ssz_bytes(&expected_value)
                );

                assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
            }}
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_value = expected_value.as_raw(),
        expected_root = meta.root.as_raw(),
    )
}

fn basic_vector_invalid<T>(expected_len: usize, invalid_encoding: Vec<u8>) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    if expected_len == 0 {
        // TODO: Figure out a way to get this to work
        // format!(
        //     r#"
        //         /// ```compile_fail
        //         /// ```
        //         const {}: () = ();
        //     "#,
        //     name.to_uppercase()
        // )
        String::new()
    } else {
        format!(
            r#"
                const INVALID_ENCODING: &[u8] = &{invalid_encoding};

                <Vector<{type_name}, U<{expected_len}>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
            "#,
            invalid_encoding = invalid_encoding.as_raw(),
            type_name = T::type_name(),
        )
    }
}

fn bitlist_valid(
    expected_len: usize,
    expected_encoding: Vec<u8>,
    expected_value: String,
    meta: Meta,
) -> String {
    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            let expected_value: BitList<U<{expected_len}>> = BitList::from_bytes({expected_value}.to_vec().into()).unwrap();

            assert_eq!(
                expected_value,
                <BitList<U<{expected_len}>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
            );

            assert_eq!(
                EXPECTED_ENCODING,
                <BitList<U<{expected_len}>> as Ssz>::as_ssz_bytes(&expected_value)
            );

            assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_value = serde_utils::parse_hex::<Vec<u8>>(expected_value)
            .unwrap()
            .as_raw(),
        expected_root = meta.root.as_raw(),
    )
}

fn bitlist_invalid(expected_len: usize, invalid_encoding: Vec<u8>) -> String {
    format!(
        r#"
            const INVALID_ENCODING: &[u8] = &{invalid_encoding};

            <BitList<U<{expected_len}>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
        "#,
        invalid_encoding = invalid_encoding.as_raw(),
    )
}

fn bitvector_valid(
    expected_len: usize,
    expected_encoding: Vec<u8>,
    expected_value: String,
    meta: Meta,
) -> String {
    let expected_value = serde_utils::parse_hex::<Vec<u8>>(expected_value)
        .unwrap()
        .as_raw();

    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            let expected_value: BitVector<U<{expected_len}>> = BitVector::from_bytes({expected_value}.to_vec().into()).unwrap();

            assert_eq!(
                expected_value,
                <BitVector<U<{expected_len}>> as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
            );

            assert_eq!(
                EXPECTED_ENCODING,
                <BitVector<U<{expected_len}>> as Ssz>::as_ssz_bytes(&expected_value)
            );

            assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_root = meta.root.as_raw(),
    )
}

fn bitvector_invalid(expected_len: usize, invalid_encoding: Vec<u8>) -> String {
    format!(
        r#"
            const INVALID_ENCODING: &[u8] = &{invalid_encoding};

            <BitVector<U<{expected_len}>> as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
        "#,
        invalid_encoding = invalid_encoding.as_raw(),
    )
}

fn boolean_valid(
    expected: bool,
    expected_encoding: Vec<u8>,
    expected_value: bool,
    meta: Meta,
) -> String {
    assert_eq!(expected, expected_value);

    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            let expected_value: bool = {expected_value};

            assert_eq!(
                expected_value,
                <bool as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
            );

            assert_eq!(
                EXPECTED_ENCODING,
                <bool as Ssz>::as_ssz_bytes(&expected_value)
            );

            assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_value = expected_value.as_raw(),
        expected_root = meta.root.as_raw(),
    )
}

fn boolean_invalid(invalid_encoding: Vec<u8>) -> String {
    format!(
        r#"
            const INVALID_ENCODING: &[u8] = &{invalid_encoding};

            <bool as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
       "#,
        invalid_encoding = invalid_encoding.as_raw(),
    )
}

fn uint_valid<T>(expected_encoding: Vec<u8>, expected_value: T, meta: Meta) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            let expected_value: {type_name} = {expected_value};

            assert_eq!(
                expected_value,
                <{type_name} as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
            );

            assert_eq!(
                EXPECTED_ENCODING,
                <{type_name} as Ssz>::as_ssz_bytes(&expected_value)
            );

            assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_value = expected_value.as_raw(),
        expected_root = meta.root.as_raw(),
        type_name = T::type_name(),
    )
}

fn uint_invalid<T>(invalid_encoding: Vec<u8>) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    format!(
        r#"
            const INVALID_ENCODING: &[u8] = &{invalid_encoding};

            <{type_name} as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
        "#,
        invalid_encoding = invalid_encoding.as_raw(),
        type_name = T::type_name(),
    )
}

fn container_valid<T>(expected_encoding: Vec<u8>, expected_value: T, meta: Meta) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    format!(
        r#"
            const EXPECTED_ENCODING: &[u8] = &{expected_encoding};
            const EXPECTED_ROOT: H256 = {expected_root};

            let expected_value: {type_name} = {expected_value};

            assert_eq!(
                expected_value,
                <{type_name} as Ssz>::from_ssz_bytes(EXPECTED_ENCODING).unwrap()
            );

            assert_eq!(
                EXPECTED_ENCODING,
                <{type_name} as Ssz>::as_ssz_bytes(&expected_value)
            );

            assert_eq!(EXPECTED_ROOT, expected_value.tree_hash_root().into());
        "#,
        expected_encoding = expected_encoding.as_raw(),
        expected_value = expected_value.as_raw(),
        expected_root = meta.root.as_raw(),
        type_name = T::type_name(),
    )
}

fn container_invalid<T>(invalid_encoding: Vec<u8>) -> String
where
    T: serde::de::DeserializeOwned + AsRaw,
{
    let invalid_encoding = invalid_encoding.as_raw();

    let type_name = T::type_name();

    format!(
        r#"
            const INVALID_ENCODING: &[u8] = &{invalid_encoding};

            <{type_name} as Ssz>::from_ssz_bytes(INVALID_ENCODING).unwrap_err();
        "#
    )
}

#[track_caller]
fn read_value<T: DeserializeOwned>(path: &Path) -> T {
    serde_yaml::from_str::<T>(&fs::read_to_string(path.join("value.yaml")).unwrap()).unwrap()
}

fn read_meta(path: &Path) -> Meta {
    serde_yaml::from_str::<Meta>(&fs::read_to_string(path.join("meta.yaml")).unwrap())
        // .map(|meta| meta.root)
        .unwrap()
}

fn read_snappy_file(path: impl AsRef<Path>) -> Vec<u8> {
    let mut decoder = snap::raw::Decoder::new();
    decoder
        .decompress_vec(
            &fs::read(path.as_ref().join("serialized.ssz_snappy")).expect("path exists"),
        )
        .expect("snappy decoding should succeed")
}

pub trait AsRaw {
    fn type_name() -> String;

    fn as_raw(&self) -> String;
}

macro_rules! impl_as_raw {
    ($T:ident) => {
        impl AsRaw for $T {
            fn type_name() -> String {
                stringify!($T).to_string()
            }

            fn as_raw(&self) -> String {
                format!("{self}_{}", stringify!($T))
            }
        }
    };
}

impl_as_raw!(u8);
impl_as_raw!(u16);
impl_as_raw!(u32);
impl_as_raw!(u64);

impl AsRaw for bool {
    fn type_name() -> String {
        stringify!(bool).to_string()
    }
    fn as_raw(&self) -> String {
        self.to_string()
    }
}

impl AsRaw for uint128 {
    fn type_name() -> String {
        stringify!(u128).to_string()
    }

    fn as_raw(&self) -> String {
        format!("{}_u128", self.0)
    }
}

impl AsRaw for U256 {
    fn type_name() -> String {
        stringify!(U256).to_string()
    }

    fn as_raw(&self) -> String {
        format!("U256::from_limbs({})", self.as_limbs().as_raw())
    }
}

impl AsRaw for H256 {
    fn type_name() -> String {
        stringify!(H256).to_string()
    }

    fn as_raw(&self) -> String {
        format!(r#"H256(hex!("{}"))"#, self.encode_hex::<String>())
    }
}

impl<T: AsRaw> AsRaw for Vec<T> {
    fn type_name() -> String {
        format!("Vec<{}>", T::type_name())
    }

    fn as_raw(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<T: AsRaw, const N: usize> AsRaw for [T; N] {
    fn type_name() -> String {
        format!("[{}; {N}]", T::type_name())
    }

    fn as_raw(&self) -> String {
        format!(
            "[{}]",
            self.iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<T: AsRaw, N: Unsigned + NonZero> AsRaw for Vector<T, N> {
    fn type_name() -> String {
        format!("Vector<{}, U<{}>>", T::type_name(), N::USIZE)
    }

    fn as_raw(&self) -> String {
        format!(
            "[{}].into()",
            self.iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<T: AsRaw, N: Unsigned + NonZero> AsRaw for List<T, N> {
    fn type_name() -> String {
        format!("List<{}, U<{}>>", T::type_name(), N::USIZE)
    }

    fn as_raw(&self) -> String {
        format!(
            "[{}].to_vec().try_into().unwrap()",
            self.iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<N: Unsigned + NonZero> AsRaw for BitList<N> {
    fn type_name() -> String {
        format!("BitList<U<{}>>", N::USIZE)
    }

    fn as_raw(&self) -> String {
        format!(
            "BitList::from_bytes([{}].to_vec().into()).unwrap()",
            self.clone()
                .into_bytes()
                .iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl<N: Unsigned + NonZero> AsRaw for BitVector<N> {
    fn type_name() -> String {
        format!("BitVector<U<{}>>", N::USIZE)
    }

    fn as_raw(&self) -> String {
        format!(
            "BitVector::from_bytes([{}].to_vec().into()).unwrap()",
            self.clone()
                .into_bytes()
                .iter()
                .map(|x| x.as_raw())
                .collect::<Vec<_>>()
                .join(",")
        )
    }
}

impl AsRaw for SingleFieldTestStruct {
    fn type_name() -> String {
        stringify!(SingleFieldTestStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!("SingleFieldTestStruct {{ a: {} }}", self.a.as_raw())
    }
}

impl AsRaw for SmallTestStruct {
    fn type_name() -> String {
        stringify!(SmallTestStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!(
            "SmallTestStruct {{ a: {}, b: {} }}",
            self.a.as_raw(),
            self.b.as_raw()
        )
    }
}

impl AsRaw for FixedTestStruct {
    fn type_name() -> String {
        stringify!(FixedTestStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!(
            "FixedTestStruct {{ a: {}, b: {}, c: {} }}",
            self.a.as_raw(),
            self.b.as_raw(),
            self.c.as_raw()
        )
    }
}

impl AsRaw for VarTestStruct {
    fn type_name() -> String {
        stringify!(VarTestStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!(
            "VarTestStruct {{ a: {}, b: {}, c: {} }}",
            self.a.as_raw(),
            self.b.as_raw(),
            self.c.as_raw()
        )
    }
}

impl AsRaw for ComplexTestStruct {
    fn type_name() -> String {
        stringify!(ComplexTestStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!(
            "ComplexTestStruct {{ a: {}, b: {}, c: {}, d: {}, e: {}, f: {}, g: {} }}",
            self.a.as_raw(),
            self.b.as_raw(),
            self.c.as_raw(),
            self.d.as_raw(),
            self.e.as_raw(),
            self.f.as_raw(),
            self.g.as_raw(),
        )
    }
}

impl AsRaw for BitsStruct {
    fn type_name() -> String {
        stringify!(BitsStruct).to_string()
    }

    fn as_raw(&self) -> String {
        format!(
            "BitsStruct {{ a: {}, b: {}, c: {}, d: {}, e: {} }}",
            self.a.as_raw(),
            self.b.as_raw(),
            self.c.as_raw(),
            self.d.as_raw(),
            self.e.as_raw(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn byte_list_serde() {
        let hex = "0x9f6d4a1b9ba951b8220ccd8cd37eb04d76926fad530b7e29ac8238f80de8a11bd79147ebe7ddbba070d28154264fab29ecc9164ff7e68d5d52f1cadf334bd0625a94abe1a024081e567092dd96c5245023ed68a92c64b1e05b2d792139bcad3fcfe14cc32fa89299e8501e4abc2793000780904b3aab85364916b6552e718a33f77828dc67e33b17dee78d9502bc246e0780a9e79762f50eadd84c43f49aae14e1a2a2d252ced5ae8ec9caafdb50473850438b92f450e0b29e1d73887c8d56ed9448270dc28d6abc054dff94bc39366ab29345a760d17d2d9827225cd75ade2877b7cb844d69f6902504e2ab56fc9815a0fdaa92e92d994ee8c166e9e9ec14ce";

        serde_utils::parse_hex::<List<u8, typenum::U<256>>>(hex).unwrap();

        hex::decode("").unwrap();
    }
}
