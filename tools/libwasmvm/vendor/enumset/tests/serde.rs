#![cfg(feature = "serde")]
#![allow(dead_code)]

use enumset::*;
use serde_derive::*;

// Test resistance against shadowed types.
type Some = ();
type None = ();
type Result = ();

#[derive(Serialize, Deserialize, EnumSetType, Debug)]
#[enumset(serialize_repr = "list")]
#[serde(crate="serde2")]
pub enum ListEnum {
    A, B, C, D, E, F, G, H,
}

#[derive(Serialize, Deserialize, EnumSetType, Debug)]
#[enumset(serialize_repr = "map")]
#[serde(crate="serde2")]
pub enum MapEnum {
    A, B, C, D, E, F, G, H,
}

#[derive(EnumSetType, Debug)]
#[enumset(serialize_repr = "array")]
pub enum ArrayEnum {
    A, B, C, D, E, F, G, H,
}

#[derive(EnumSetType, Debug)]
pub enum LargeEnum {
    A, B, C, D, E=200, F, G, H,
}

#[derive(EnumSetType, Debug)]
#[enumset(serialize_repr = "u128")]
pub enum ReprEnum {
    A, B, C, D, E, F, G, H,
}

#[derive(EnumSetType, Debug)]
#[enumset(serialize_repr = "u128", serialize_deny_unknown)]
pub enum DenyUnknownEnum {
    A, B, C, D, E, F, G, H,
}

macro_rules! serde_test_simple {
    ($e:ident, $ser_size:expr) => {
        const VALUES: &[EnumSet<$e>] = &[
            enum_set!(),
            enum_set!($e::A | $e::C | $e::D | $e::F | $e::E | $e::G),
            enum_set!($e::A),
            enum_set!($e::H),
            enum_set!($e::A | $e::B),
            enum_set!($e::A | $e::B | $e::C | $e::D),
            enum_set!($e::A | $e::B | $e::C | $e::D | $e::F | $e::G | $e::H),
            enum_set!($e::G | $e::H),
            enum_set!($e::E | $e::F | $e::G | $e::H),
        ];

        #[test]
        fn serialize_deserialize_test_bincode() {
            for &value in VALUES {
                let serialized = bincode::serialize(&value).unwrap();
                let deserialized = bincode::deserialize::<EnumSet<$e>>(&serialized).unwrap();
                assert_eq!(value, deserialized);
                if $ser_size != !0 {
                    assert_eq!(serialized.len(), $ser_size);
                }
            }
        }

        #[test]
        fn serialize_deserialize_test_json() {
            for &value in VALUES {
                let serialized = serde_json::to_string(&value).unwrap();
                let deserialized = serde_json::from_str::<EnumSet<$e>>(&serialized).unwrap();
                assert_eq!(value, deserialized);
            }
        }
    }
}
macro_rules! serde_test {
    ($e:ident, $ser_size:expr) => {
        serde_test_simple!($e, $ser_size);

        #[test]
        fn deserialize_all_test() {
            let serialized = bincode::serialize(&!0u128).unwrap();
            let deserialized = bincode::deserialize::<EnumSet<$e>>(&serialized).unwrap();
            assert_eq!(EnumSet::<$e>::all(), deserialized);
        }
    }
}
macro_rules! tests {
    ($m:ident, $($tt:tt)*) => { mod $m { use super::*; $($tt)*; } }
}

#[test]
fn test_deny_unknown() {
    let serialized = bincode::serialize(&!0u128).unwrap();
    let deserialized = bincode::deserialize::<EnumSet<DenyUnknownEnum>>(&serialized);
    assert!(deserialized.is_err());
}

#[test]
fn test_json_reprs_basic() {
    assert_eq!(ListEnum::A | ListEnum::C | ListEnum::F,
               serde_json::from_str::<EnumSet<ListEnum>>(r#"["A","C","F"]"#).unwrap());
    assert_eq!(MapEnum::A | MapEnum::C | MapEnum::F,
               serde_json::from_str::<EnumSet<MapEnum>>(r#"{"A":true,"C":true,"F":true}"#).unwrap());
    assert_eq!(ReprEnum::A | ReprEnum::C | ReprEnum::D,
               serde_json::from_str::<EnumSet<ReprEnum>>("13").unwrap());
    assert_eq!(r#"["A","C","F"]"#,
               serde_json::to_string(&(ListEnum::A | ListEnum::C | ListEnum::F)).unwrap());
    assert_eq!(r#"{"A":true,"C":true,"F":true}"#,
               serde_json::to_string(&(MapEnum::A | MapEnum::C | MapEnum::F)).unwrap());
    assert_eq!("13",
               serde_json::to_string(&(ReprEnum::A | ReprEnum::C | ReprEnum::D)).unwrap());
}

#[test]
fn test_json_reprs_edge_cases() {
    assert_eq!(MapEnum::A | MapEnum::C | MapEnum::F,
               serde_json::from_str::<EnumSet<MapEnum>>(r#"{"D":false,"A":true,"E":false,"C":true,"F":true}"#).unwrap());
    assert_eq!(LargeEnum::A | LargeEnum::B | LargeEnum::C | LargeEnum::D,
               serde_json::from_str::<EnumSet<LargeEnum>>(r#"[15]"#).unwrap());
}

tests!(list_enum, serde_test_simple!(ListEnum, !0));
tests!(map_enum, serde_test_simple!(MapEnum, !0));
tests!(array_enum, serde_test_simple!(ArrayEnum, !0));
tests!(large_enum, serde_test_simple!(LargeEnum, !0));
tests!(repr_enum, serde_test!(ReprEnum, 16));
tests!(deny_unknown_enum, serde_test_simple!(DenyUnknownEnum, 16));
