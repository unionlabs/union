use core::{
    fmt::{Debug, Display},
    str::FromStr,
};

use crate::encoding::{Decode, Encode, Proto};

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
    T: Display + FromStr + Debug + PartialEq,
    <T as FromStr>::Err: Debug,
{
    let from_str = t.to_string().parse::<T>().unwrap();

    assert_eq!(t, &from_str, "string roundtrip failed");
}
