use core::{
    fmt::{Debug, Display},
    str::FromStr,
};

use unionlabs_primitives::Bytes;

use crate::encoding::{Decode, DecodeAs, Encode, EncodeAs, Encoding, Proto};

#[track_caller]
pub fn assert_proto_roundtrip<T>(t: &T)
where
    T: Encode<Proto> + Decode<Proto> + Debug + Clone + PartialEq,
{
    let try_from_proto = T::decode(&t.clone().encode()).unwrap();

    assert_eq!(t, &try_from_proto, "proto roundtrip failed");
}

#[track_caller]
pub fn assert_json_roundtrip<T>(t: &T)
where
    T: serde::Serialize + for<'a> serde::Deserialize<'a> + Debug + PartialEq,
{
    let from_json = serde_json::from_str::<T>(&serde_json::to_string(&t).unwrap()).unwrap();

    assert_eq!(t, &from_json, "json roundtrip failed");
}

#[track_caller]
pub fn assert_string_roundtrip<T>(s: &str, t: &T)
where
    T: Display + FromStr<Err: Debug> + Debug + PartialEq,
{
    let display = t.to_string();

    assert_eq!(s, &display, "display is not equal");

    assert_eq!(&display.parse::<T>().unwrap(), t, "parse is not equal");
}

#[track_caller]
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
    assert_eq!(<Bytes>::from(t.clone().encode_as::<E>()), <Bytes>::from(bz));

    assert_eq!(T::decode_as::<E>(bz).unwrap(), t.clone());
}
