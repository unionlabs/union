use js_sys::{BigInt, JsString, Number, Object};
use maplit::{btreemap, hashmap, hashset};
use proptest::prelude::*;
use serde::de::DeserializeOwned;
use serde::ser::Error as SerError;
use serde::{Deserialize, Serialize};
use serde_wasm_bindgen::{from_value, to_value, Error, Serializer};
use std::collections::{BTreeMap, HashMap};
use std::fmt::Debug;
use std::hash::Hash;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_test::wasm_bindgen_test;

const SERIALIZER: Serializer = Serializer::new();

const BIGINT_SERIALIZER: Serializer =
    Serializer::new().serialize_large_number_types_as_bigints(true);

const MAP_OBJECT_SERIALIZER: Serializer = Serializer::new().serialize_maps_as_objects(true);

fn test_via_round_trip_with_config<T>(value: T, serializer: &Serializer) -> JsValue
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    let serialized = value.serialize(serializer).unwrap();
    let round_trip = from_value(serialized.clone()).unwrap();
    assert_eq!(
        value, round_trip,
        "{:?} == from_value({:?})",
        value, serialized
    );
    serialized
}

fn test_via_into_with_config<L, R>(lhs: L, rhs: R, serializer: &Serializer)
where
    L: Serialize + DeserializeOwned + PartialEq + Clone + Debug,
    R: Into<JsValue> + Clone + Debug,
{
    let serialized = test_via_round_trip_with_config(lhs.clone(), serializer);
    assert_eq!(
        serialized,
        rhs.clone().into(),
        "to_value({:?}) == JsValue::from({:?})",
        lhs,
        rhs
    );
}

fn test_via_into<L, R>(lhs: L, rhs: R)
where
    L: Serialize + DeserializeOwned + PartialEq + Clone + Debug,
    R: Into<JsValue> + Clone + Debug,
{
    test_via_into_with_config(lhs, rhs, &SERIALIZER)
}

fn test_primitive_with_config<T>(value: T, serializer: &Serializer)
where
    T: Copy + Serialize + Into<JsValue> + DeserializeOwned + PartialEq + Debug,
{
    test_via_into_with_config(value, value, serializer)
}

fn test_primitive<T>(value: T)
where
    T: Copy + Serialize + Into<JsValue> + DeserializeOwned + PartialEq + Debug,
{
    test_via_into(value, value)
}

#[derive(PartialEq)]
enum ValueKind {
    Null,
    Undefined,
    Boolean,
    PosFloat,
    NegFloat,
    NaN,
    PosInfinity,
    NegInfinity,
    PosInt,
    NegInt,
    PosBigInt,
    NegBigInt,
    String,
    Object,
}

fn sample_js_values() -> Vec<(ValueKind, JsValue)> {
    vec![
        (ValueKind::Null, JsValue::NULL),
        (ValueKind::Undefined, JsValue::UNDEFINED),
        (ValueKind::Boolean, JsValue::TRUE),
        (ValueKind::PosFloat, JsValue::from(0.5)),
        (ValueKind::NegFloat, JsValue::from(-0.5)),
        (ValueKind::NaN, JsValue::from(std::f64::NAN)),
        (ValueKind::PosInfinity, JsValue::from(std::f64::INFINITY)),
        (ValueKind::NegInfinity, JsValue::from(-std::f64::INFINITY)),
        (ValueKind::PosInt, JsValue::from(1)),
        (ValueKind::NegInt, JsValue::from(-1)),
        (ValueKind::PosBigInt, JsValue::from(BigInt::from(1_i64))),
        (ValueKind::NegBigInt, JsValue::from(BigInt::from(-1_i64))),
        (ValueKind::String, JsValue::from("1")),
        (ValueKind::Object, Object::new().into()),
    ]
}

macro_rules! test_value_compatibility {
    ($ty:ty { $(ValueKind::$kind:ident $delim:tt $rust_value:expr,)* }) => {
        for (kind, js_value) in sample_js_values() {
            match kind {
                $(ValueKind::$kind => {
                    let rust_value: $ty = $rust_value;
                    test_value_compatibility!(@compare $ty, $kind $delim rust_value, js_value);
                })*
                _ => {
                    from_value::<$ty>(js_value).unwrap_err();
                }
            }
        }
    };

    (@compare $ty:ty, NaN => $rust_value:ident, $js_value:ident) => {{
        let mut rust_value: $ty = $rust_value;
        assert!(rust_value.is_nan(), "{:?} is not NaN", rust_value);

        let js_value = to_value(&rust_value).unwrap();
        assert!(Number::is_nan(&js_value), "{:?} is not NaN", js_value);

        rust_value = from_value(js_value).unwrap();
        assert!(rust_value.is_nan(), "{:?} is not NaN", rust_value);
    }};

    (@compare $ty:ty, $kind:ident -> $rust_value:ident, $js_value:ident) => {{
        assert_ne!(to_value(&$rust_value).ok().as_ref(), Some(&$js_value), "to_value({:?}) != Ok({:?})", $rust_value, $js_value);
        let rust_value: $ty = from_value($js_value.clone()).unwrap();
        assert_eq!(rust_value, $rust_value, "from_value from {:?}", $js_value);
    }};

    (@compare $ty:ty, $kind:ident => $rust_value:ident, $js_value:ident) => (
        test_via_into::<$ty, JsValue>($rust_value, $js_value)
    );
}

#[wasm_bindgen_test]
fn unit() {
    test_via_into((), JsValue::UNDEFINED);
    test_value_compatibility!(() {
        ValueKind::Undefined => (),
        // Special case: one-way only conversion.
        ValueKind::Null -> (),
    });
}

mod proptests {
    use super::*;

    proptest! {
        #[wasm_bindgen_test]
        fn bool(value: bool) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn i8(value: i8) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn i16(value: i16) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn i32(value: i32) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn u8(value: u8) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn u16(value: u16) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn u32(value: u32) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn f32(value: f32) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn f64(value: f64) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn i64_bigints(value: i64) {
            test_primitive_with_config(value, &BIGINT_SERIALIZER);
        }

        #[wasm_bindgen_test]
        fn u64_bigints(value: i64) {
            test_primitive_with_config(value, &BIGINT_SERIALIZER);
        }

        #[wasm_bindgen_test]
        fn i64_numbers(value in Number::MIN_SAFE_INTEGER as i64..=Number::MAX_SAFE_INTEGER as i64) {
            test_via_into(value, value as f64);
        }

        #[wasm_bindgen_test]
        fn u64_numbers(value in 0..=Number::MAX_SAFE_INTEGER as u64) {
            test_via_into(value, value as f64);
        }

        #[wasm_bindgen_test]
        fn isize(value: isize) {
            test_via_into(value, value as f64);
            test_via_into_with_config(value, value as i64, &BIGINT_SERIALIZER);
        }

        #[wasm_bindgen_test]
        fn usize(value: usize) {
            test_via_into(value, value as f64);
            test_via_into_with_config(value, value as u64, &BIGINT_SERIALIZER);
        }

        #[wasm_bindgen_test]
        fn i128(value: i128) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn u128(value: u128) {
            test_primitive(value);
        }

        #[wasm_bindgen_test]
        fn char(c: char) {
            test_via_into(c, String::from(c));
        }

        #[wasm_bindgen_test]
        fn string(s: String) {
            test_via_into(s.clone(), s);
        }
    }
}

mod compat {
    use super::*;

    macro_rules! test_int_boundaries {
        ($ty:ident) => {
            test_primitive::<$ty>($ty::MIN);
            test_primitive::<$ty>($ty::MAX);

            let too_small = f64::from($ty::MIN) - 1.0;
            from_value::<$ty>(too_small.into()).unwrap_err();

            let too_large = f64::from($ty::MAX) + 1.0;
            from_value::<$ty>(too_large.into()).unwrap_err();
        };
    }

    macro_rules! test_bigint_boundaries {
        ($ty:ident $(as $as:ident)?) => {
            test_via_into_with_config($ty::MIN, $ty::MIN $(as $as)?, &BIGINT_SERIALIZER);
            test_via_into_with_config($ty::MAX, $ty::MAX $(as $as)?, &BIGINT_SERIALIZER);

            let too_small = BigInt::from($ty::MIN $(as $as)?) - BigInt::from(1);
            from_value::<$ty>(too_small.into()).unwrap_err();

            let too_large = BigInt::from($ty::MAX $(as $as)?) + BigInt::from(1);
            from_value::<$ty>(too_large.into()).unwrap_err();
        };
    }

    macro_rules! test_safe_int_boundaries {
        (signed $ty:ident) => {
            test_via_into(Number::MIN_SAFE_INTEGER as $ty, Number::MIN_SAFE_INTEGER);
            from_value::<$ty>(JsValue::from(Number::MIN_SAFE_INTEGER - 1.)).unwrap_err();
            test_primitive_with_config((Number::MIN_SAFE_INTEGER - 1.) as $ty, &BIGINT_SERIALIZER);

            test_safe_int_boundaries!(unsigned $ty);
        };

        (unsigned $ty:ident) => {
            test_via_into(Number::MAX_SAFE_INTEGER as $ty, Number::MAX_SAFE_INTEGER);
            from_value::<$ty>(JsValue::from(Number::MAX_SAFE_INTEGER + 1.)).unwrap_err();
            test_primitive_with_config((Number::MAX_SAFE_INTEGER + 1.) as $ty, &BIGINT_SERIALIZER);
        };
    }

    #[wasm_bindgen_test]
    fn bool() {
        test_value_compatibility!(bool {
            ValueKind::Boolean => true,
        });
    }

    #[wasm_bindgen_test]
    fn i8() {
        test_int_boundaries!(i8);
        test_value_compatibility!(i8 {
            ValueKind::PosInt => 1,
            ValueKind::NegInt => -1,
        });
    }

    #[wasm_bindgen_test]
    fn i16() {
        test_int_boundaries!(i16);
        test_value_compatibility!(i16 {
            ValueKind::PosInt => 1,
            ValueKind::NegInt => -1,
        });
    }

    #[wasm_bindgen_test]
    fn i32() {
        test_int_boundaries!(i32);
        test_value_compatibility!(i32 {
            ValueKind::PosInt => 1,
            ValueKind::NegInt => -1,
        });
    }

    #[wasm_bindgen_test]
    fn u8() {
        test_int_boundaries!(u8);
        test_value_compatibility!(u8 {
            ValueKind::PosInt => 1,
        });
    }

    #[wasm_bindgen_test]
    fn u16() {
        test_int_boundaries!(u16);
        test_value_compatibility!(u16 {
            ValueKind::PosInt => 1,
        });
    }

    #[wasm_bindgen_test]
    fn u32() {
        test_int_boundaries!(u32);
        test_value_compatibility!(u32 {
            ValueKind::PosInt => 1,
        });
    }

    #[wasm_bindgen_test]
    fn i64() {
        test_bigint_boundaries!(i64);
        test_safe_int_boundaries!(signed i64);
        test_value_compatibility!(i64 {
            ValueKind::PosInt => 1,
            ValueKind::NegInt => -1,
            ValueKind::PosBigInt -> 1,
            ValueKind::NegBigInt -> -1,
        });
    }

    #[wasm_bindgen_test]
    fn u64() {
        test_bigint_boundaries!(u64);
        test_safe_int_boundaries!(unsigned u64);
        test_value_compatibility!(u64 {
            ValueKind::PosInt => 1,
            ValueKind::PosBigInt -> 1,
        });
    }

    #[wasm_bindgen_test]
    fn i128() {
        test_bigint_boundaries!(i128);
        test_value_compatibility!(i128 {
            ValueKind::PosBigInt => 1,
            ValueKind::NegBigInt => -1,
        });
    }

    #[wasm_bindgen_test]
    fn u128() {
        test_bigint_boundaries!(u128);
        test_value_compatibility!(u128 {
            ValueKind::PosBigInt => 1,
        });
    }

    #[wasm_bindgen_test]
    fn isize() {
        test_bigint_boundaries!(isize as i64);
        test_value_compatibility!(isize {
            ValueKind::PosInt => 1,
            ValueKind::NegInt => -1,
            ValueKind::PosBigInt -> 1,
            ValueKind::NegBigInt -> -1,
        });
    }

    #[wasm_bindgen_test]
    fn usize() {
        test_bigint_boundaries!(usize as u64);
        test_value_compatibility!(usize {
            ValueKind::PosInt => 1,
            ValueKind::PosBigInt -> 1,
        });
    }

    #[wasm_bindgen_test]
    fn f32() {
        test_value_compatibility!(f32 {
            ValueKind::PosFloat => 0.5,
            ValueKind::NegFloat => -0.5,
            ValueKind::NaN => f32::NAN,
            ValueKind::PosInfinity => f32::INFINITY,
            ValueKind::NegInfinity => f32::NEG_INFINITY,
            ValueKind::PosInt => 1.0,
            ValueKind::NegInt => -1.0,
        });
    }

    #[wasm_bindgen_test]
    fn f64() {
        test_value_compatibility!(f64 {
            ValueKind::PosFloat => 0.5,
            ValueKind::NegFloat => -0.5,
            ValueKind::NaN => f64::NAN,
            ValueKind::PosInfinity => f64::INFINITY,
            ValueKind::NegInfinity => f64::NEG_INFINITY,
            ValueKind::PosInt => 1.0,
            ValueKind::NegInt => -1.0,
        });
    }

    #[wasm_bindgen_test]
    fn string() {
        let lone_surrogate = JsString::from_char_code(&[0xDC00]);
        // Lone surrogates currently become a replacement character.
        assert_eq!(
            from_value::<String>(lone_surrogate.into()).unwrap(),
            "\u{FFFD}"
        );
    }
}

#[wasm_bindgen_test]
fn bytes() {
    // Create a backing storage.
    let mut src = [1, 2, 3];
    // Store the original separately for the mutation test
    let orig_src = src;
    // Convert to a JS value
    let value = to_value(&serde_bytes::Bytes::new(&src)).unwrap();
    // Modify the original storage to make sure that JS value is a copy.
    src[0] = 10;

    // Make sure the JS value is a Uint8Array
    let res = value.dyn_ref::<js_sys::Uint8Array>().unwrap();
    // Copy it into another Rust storage
    let mut dst = [0; 3];
    res.copy_to(&mut dst);
    // Finally, compare that resulting storage with the original.
    assert_eq!(orig_src, dst);

    // Now, try to deserialize back.
    let deserialized: serde_bytes::ByteBuf = from_value(value).unwrap();
    assert_eq!(deserialized.as_ref(), orig_src);
}

#[wasm_bindgen_test]
fn options() {
    test_via_into(Some(0_u32), 0_u32);
    test_via_into(Some(32_u32), 32_u32);
    test_via_into(None::<u32>, JsValue::UNDEFINED);

    test_via_into(Some("".to_string()), "");
    test_via_into(Some("abc".to_string()), "abc");
    test_via_into(None::<String>, JsValue::UNDEFINED);

    // This one is an unfortunate edge case that won't roundtrip,
    // but it's pretty unlikely in real-world code.
    assert_eq!(to_value(&Some(())).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&None::<()>).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&Some(Some(()))).unwrap(), JsValue::UNDEFINED);
    assert_eq!(to_value(&Some(None::<()>)).unwrap(), JsValue::UNDEFINED);
}

fn assert_json<R>(lhs_value: JsValue, rhs: R)
where
    R: Serialize + DeserializeOwned + PartialEq + Debug,
{
    if lhs_value.is_undefined() {
        assert_eq!("null", serde_json::to_string(&rhs).unwrap())
    } else {
        assert_eq!(
            js_sys::JSON::stringify(&lhs_value).unwrap(),
            serde_json::to_string(&rhs).unwrap(),
        );
    }

    let restored_lhs: R = from_value(lhs_value.clone()).unwrap();
    assert_eq!(restored_lhs, rhs, "from_value from {:?}", lhs_value);
}

fn test_via_json_with_config<T>(value: T, serializer: &Serializer)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    assert_json(value.serialize(serializer).unwrap(), value);
}

fn test_via_json<T>(value: T)
where
    T: Serialize + DeserializeOwned + PartialEq + Debug,
{
    test_via_json_with_config(value, &SERIALIZER);
}

#[wasm_bindgen_test]
fn enums() {
    macro_rules! test_enum {
        ($(# $attr:tt)* $name:ident) => {{
            #[derive(Debug, PartialEq, Serialize, Deserialize)]
            $(# $attr)*
            enum $name<A, B> where A: Debug + Ord + Eq {
                Unit,
                Newtype(A),
                Tuple(A, B),
                Struct { a: A, b: B },
                Map(BTreeMap<A, B>),
                Seq { seq: Vec<B> } // internal tags cannot be directly embedded in arrays
            }

            test_via_json($name::Unit::<String, i32>);
            test_via_json($name::Newtype::<_, i32>("newtype content".to_string()));
            test_via_json($name::Tuple("tuple content".to_string(), 42));
            test_via_json($name::Struct {
                a: "struct content".to_string(),
                b: 42,
            });
            test_via_json_with_config($name::Map::<String, i32>(
                btreemap!{
                    "a".to_string() => 12,
                    "abc".to_string() => -1161,
                    "b".to_string() => 64,
                }
            ), &MAP_OBJECT_SERIALIZER);
            test_via_json($name::Seq::<i32, i32> { seq: vec![5, 63, 0, -62, 6] });
        }};
    }

    test_enum! {
        ExternallyTagged
    }

    #[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
    #[serde(tag = "tag")]
    enum InternallyTagged<A, B>
    where
        A: Ord,
    {
        Unit,
        Struct { a: A, b: B },
        Sequence { seq: Vec<A> },
        Map(BTreeMap<A, B>),
    }

    test_via_json(InternallyTagged::Unit::<(), ()>);
    test_via_json(InternallyTagged::Struct {
        a: "struct content".to_string(),
        b: 42,
    });
    test_via_json(InternallyTagged::Struct {
        a: "struct content".to_string(),
        b: 42.2,
    });
    test_via_json(InternallyTagged::<i32, ()>::Sequence {
        seq: vec![12, 41, -11, -65, 961],
    });

    // Internal tags with maps are not properly deserialized from Map values due to the exclusion
    // of Iterables during deserialize_any(). They can be deserialized properly from plain objects
    // so we can test that.
    test_via_json_with_config(
        InternallyTagged::Map(btreemap! {
            "a".to_string() => 12,
            "abc".to_string() => -1161,
            "b".to_string() => 64,
        }),
        &MAP_OBJECT_SERIALIZER,
    );

    test_via_round_trip_with_config(
        InternallyTagged::Struct {
            a: 10_u64,
            b: -10_i64,
        },
        &BIGINT_SERIALIZER,
    );

    test_enum! {
        #[serde(tag = "tag", content = "content")]
        AdjacentlyTagged
    }
    test_enum! {
        #[serde(untagged)]
        Untagged
    }
}

#[wasm_bindgen_test]
fn structs() {
    #[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
    struct Unit;

    test_via_into(Unit, JsValue::UNDEFINED);

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Newtype<A>(A);

    test_via_json(Newtype("newtype content".to_string()));

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Tuple<A, B>(A, B);

    test_via_json(Tuple("tuple content".to_string(), 42));

    #[derive(Debug, PartialEq, Serialize, Deserialize)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    test_via_json(Struct {
        a: "struct content".to_string(),
        b: 42,
    });
}

#[wasm_bindgen_test]
fn sequences() {
    test_via_json([1, 2]);
    test_via_json(["".to_string(), "x".to_string(), "xyz".to_string()]);
    test_via_json((100, "xyz".to_string(), true));

    // Sets are currently indistinguishable from other sequences for
    // Serde serialisers, so this will become an array on the JS side.
    test_via_json(hashset! {false, true});
}

#[wasm_bindgen_test]
fn maps() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    // Create a Rust HashMap with non-string keys to make sure
    // that we support real arbitrary maps.
    let src = hashmap! {
        Struct {
            a: 1,
            b: "smth".to_string(),
        } => Struct {
            a: 2,
            b: "SMTH".to_string(),
        },
        Struct {
            a: 42,
            b: "something".to_string(),
        } => Struct {
            a: 84,
            b: "SOMETHING".to_string(),
        },
    };

    // Convert to a JS value
    let res = to_value(&src).unwrap();

    // Make sure that the result is an ES6 Map.
    let res = res.dyn_into::<js_sys::Map>().unwrap();
    assert_eq!(res.size() as usize, src.len());

    // Compare values one by one (it's ok to use JSON for invidivual structs).
    res.entries()
        .into_iter()
        .map(|kv| kv.unwrap())
        .zip(src)
        .for_each(|(lhs_kv, rhs_kv)| {
            assert_json(lhs_kv, rhs_kv);
        });
}

#[wasm_bindgen_test]
fn maps_objects_string_key() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    let src = hashmap! {
        "a".to_string() => Struct {
            a: 2,
            b: "S".to_string(),
        },
        "b".to_string() => Struct {
            a: 3,
            b: "T".to_string(),
        },
    };

    test_via_json_with_config(src, &MAP_OBJECT_SERIALIZER);
}

#[wasm_bindgen_test]
fn serialize_json_compatible() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
    struct Struct {
        a: HashMap<String, ()>,
        b: Option<i32>,
    }
    let x = Struct {
        a: hashmap! {
            "foo".to_string() => (),
            "bar".to_string() => (),
        },
        b: None,
    };
    test_via_json_with_config(x, &Serializer::json_compatible());
}

#[wasm_bindgen_test]
fn maps_objects_object_key() {
    #[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash)]
    struct Struct<A, B> {
        a: A,
        b: B,
    }

    let src = hashmap! {
        Struct {
            a: 1,
            b: "smth".to_string(),
        } => Struct {
            a: 2,
            b: "SMTH".to_string(),
        },
        Struct {
            a: 42,
            b: "something".to_string(),
        } => Struct {
            a: 84,
            b: "SOMETHING".to_string(),
        },
    };

    let res = src.serialize(&MAP_OBJECT_SERIALIZER).unwrap_err();
    assert_eq!(
        res.to_string(),
        Error::custom("Map key is not a string and cannot be an object key").to_string()
    );
}

#[wasm_bindgen_test]
fn serde_default_fields() {
    #[derive(Deserialize)]
    #[allow(dead_code)]
    struct Struct {
        data: String,
        #[serde(default)]
        missing: bool,
        opt_field: Option<String>,
        unit_field: (),
    }

    let json = r#"{"data": "testing", "unit_field": null}"#;
    let obj = js_sys::JSON::parse(json).unwrap();

    // Check that it parses successfully despite the missing field.
    let _struct: Struct = from_value(obj).unwrap();
}
