use core::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::encoding::{Decode, DecodeAs, Encode, EncodeAs, Encoding, Proto};

pub fn assert_proto_roundtrip<T>(t: &T)
where
    T: Encode<Proto> + Decode<Proto> + Debug + Clone + PartialEq,
{
    let try_from_proto = T::decode(&t.clone().encode()).unwrap();

    assert_eq!(t, &try_from_proto, "proto roundtrip failed");
}

pub fn assert_json_roundtrip<T>(t: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + Debug + PartialEq,
{
    let from_json = serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap()).unwrap();

    assert_eq!(t, &from_json, "json roundtrip failed");
}

pub fn assert_string_roundtrip<T>(t: &T)
where
    T: Display + FromStr<Err: Debug> + Debug + PartialEq,
{
    let from_str = t.to_string().parse::<T>().unwrap();

    assert_eq!(t, &from_str, "string roundtrip failed");
}

pub fn assert_codec_iso<T, E: Encoding>(t: &T)
where
    T: Encode<E> + Decode<E> + Clone + Debug + PartialEq,
{
    let iso = T::decode_as::<E>(&t.clone().encode_as::<E>()).unwrap();

    assert_eq!(t, &iso, "roundtrip failed");
}

#[track_caller]
pub fn assert_codec_iso_bytes<T, E: Encoding>(t: &T, bz: &[u8])
where
    T: Encode<E> + Decode<E> + Clone + Debug + PartialEq,
{
    assert_eq!(T::decode_as::<E>(bz).unwrap(), t.clone());

    assert_eq!(t.clone().encode_as::<E>(), bz);
}
