#![no_std]
#![feature(error_in_core)]

extern crate alloc;

use alloc::{
    format,
    string::{String, ToString},
    vec,
    vec::Vec,
};
use core::fmt::Debug;

use hex::FromHexError;

pub const HEX_ENCODING_PREFIX: &str = "0x";

#[derive(Debug, Clone, PartialEq)]
pub enum FromHexStringError {
    Hex(FromHexError),
    MissingPrefix(String),
    EmptyString,
    // NOTE: Contains the stringified error
    TryFromBytes(String),
}

impl core::error::Error for FromHexStringError {
    fn source(&self) -> Option<&(dyn core::error::Error + 'static)> {
        match self {
            FromHexStringError::Hex(hex) => Some(hex),
            FromHexStringError::EmptyString => None,
            FromHexStringError::MissingPrefix(_) => None,
            FromHexStringError::TryFromBytes(_) => None,
        }
    }
}

impl core::fmt::Display for FromHexStringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FromHexStringError::Hex(e) => write!(f, "{e}"),
            FromHexStringError::EmptyString => write!(f, "cannot parse empty string as hex"),
            FromHexStringError::MissingPrefix(data) => write!(
                f,
                "missing prefix `{HEX_ENCODING_PREFIX}` when deserializing hex data '{data}'",
            ),
            FromHexStringError::TryFromBytes(err) => {
                write!(f, "unable to convert from bytes: {err:?}")
            }
        }
    }
}

pub fn to_hex<T: AsRef<[u8]>>(data: T) -> String {
    let data = data.as_ref();

    let encoded = if data.is_empty() {
        "0".to_string()
    } else {
        hex::encode(data)
    };

    format!("{HEX_ENCODING_PREFIX}{encoded}")
}

pub fn parse_hex<T>(string: impl AsRef<[u8]>) -> Result<T, FromHexStringError>
where
    T: TryFrom<Vec<u8>, Error: Debug + 'static>,
{
    let s = &string.as_ref();

    if s.is_empty() {
        return Err(FromHexStringError::EmptyString);
    }

    match s.strip_prefix(HEX_ENCODING_PREFIX.as_bytes()) {
        Some([b'0']) => {
            T::try_from(vec![]).map_err(|err| FromHexStringError::TryFromBytes(format!("{err:?}")))
        }
        Some(maybe_hex) => hex::decode(maybe_hex)
            .map_err(FromHexStringError::Hex)?
            .try_into()
            .map_err(|err| FromHexStringError::TryFromBytes(format!("{err:?}"))),
        None => Err(FromHexStringError::MissingPrefix(
            String::from_utf8_lossy(string.as_ref()).into_owned(),
        )),
    }
}

pub mod base64 {
    use alloc::{format, string::String, vec::Vec};
    use core::fmt::Debug;

    use base64::prelude::*;
    use serde::{de, Deserialize, Deserializer};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&BASE64_STANDARD.encode(data))
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>, Error: Debug + 'static>,
    {
        BASE64_STANDARD
            .decode(String::deserialize(deserializer)?.as_bytes())
            .map_err(de::Error::custom)?
            .try_into()
            .map_err(|err| de::Error::custom(format!("{err:?}")))
    }
}

pub mod inner_base64 {
    use alloc::{string::String, vec::Vec};

    use base64::prelude::*;
    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(
        #[allow(clippy::ptr_arg)] // required by serde
        bytes: &Vec<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        serializer.collect_seq(bytes.iter().map(|b| BASE64_STANDARD.encode(b)))
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Vec<Vec<u8>>, D::Error> {
        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|item| BASE64_STANDARD.decode(item))
            .collect::<Result<Vec<_>, _>>()
            .map_err(de::Error::custom)
    }
}

pub mod base64_opt {
    use alloc::{string::String, vec::Vec};

    use base64::prelude::*;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        #[allow(clippy::ptr_arg)] // required by serde
        bytes: &Option<Vec<u8>>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        bytes
            .as_ref()
            .map(|b| BASE64_STANDARD.encode(b))
            .serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<Option<Vec<u8>>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|x| BASE64_STANDARD.decode(x).map_err(de::Error::custom))
            .transpose()
    }
}

pub mod base64_opt_default {
    use alloc::{string::String, vec::Vec};

    use base64::prelude::*;
    use serde::{de, Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        #[allow(clippy::ptr_arg)] // required by serde
        bytes: &Vec<u8>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        BASE64_STANDARD.encode(bytes).serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Vec<u8>, D::Error> {
        Option::<String>::deserialize(deserializer)?
            .map(|x| BASE64_STANDARD.decode(x).map_err(de::Error::custom))
            .transpose()
            .map(|x| x.unwrap_or_default())
    }
}

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8b514073821e558a5ce862f64361492e
// will optimize this later
pub mod fixed_size_array {
    use alloc::{format, vec::Vec};
    use core::marker::PhantomData;

    use serde::{
        de::{SeqAccess, Visitor},
        ser::SerializeTuple,
        Deserialize, Deserializer, Serialize, Serializer,
    };

    pub fn serialize<S: Serializer, T: Serialize, const N: usize>(
        data: &[T; N],
        ser: S,
    ) -> Result<S::Ok, S::Error> {
        let mut s = ser.serialize_tuple(N)?;
        for item in data {
            s.serialize_element(item)?;
        }
        s.end()
    }

    struct ArrayVisitor<T, const N: usize>(PhantomData<T>);

    impl<'de, T, const N: usize> Visitor<'de> for ArrayVisitor<T, N>
    where
        T: Deserialize<'de>,
    {
        type Value = [T; N];

        fn expecting(&self, formatter: &mut core::fmt::Formatter) -> core::fmt::Result {
            formatter.write_str(&format!("an array of length {}", N))
        }

        #[inline]
        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            // can be optimized using MaybeUninit
            let mut data = Vec::with_capacity(N);
            for _ in 0..N {
                match seq.next_element()? {
                    Some(val) => data.push(val),
                    None => return Err(serde::de::Error::invalid_length(N, &self)),
                }
            }
            match data.try_into() {
                Ok(arr) => Ok(arr),
                Err(_) => unreachable!(),
            }
        }
    }

    pub fn deserialize<'de, D, T, const N: usize>(deserializer: D) -> Result<[T; N], D::Error>
    where
        D: Deserializer<'de>,
        T: Deserialize<'de>,
    {
        deserializer.deserialize_tuple(N, ArrayVisitor::<T, N>(PhantomData))
    }
}

pub mod hex_string {
    use alloc::{string::String, vec::Vec};
    use core::fmt::Debug;

    use serde::{de, Deserialize, Deserializer, Serializer};

    use crate::{parse_hex, to_hex};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&to_hex(data))
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>, Error: Debug + 'static>,
    {
        String::deserialize(deserializer).and_then(|x| parse_hex::<T>(x).map_err(de::Error::custom))
    }
}

pub mod u64_hex {
    use alloc::{format, string::String};

    use serde::{de, Deserialize};

    use crate::HEX_ENCODING_PREFIX;

    pub fn serialize<S>(data: &u64, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("0x{data:x}"))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u64, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|x| {
            x.strip_prefix(HEX_ENCODING_PREFIX)
                .ok_or_else(|| de::Error::custom("missing 0x prefix"))
                .and_then(|s| u64::from_str_radix(s, 16).map_err(de::Error::custom))
        })
    }
}

pub mod hex_upper_unprefixed {
    use alloc::{format, string::String, vec::Vec};
    use core::fmt::Debug;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if data.as_ref().is_empty() {
            serializer.serialize_str("0")
        } else {
            serializer.serialize_str(&hex::encode_upper(data))
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>, Error: Debug + 'static>,
    {
        let s = String::deserialize(deserializer)?;
        let bz = hex::decode(s).map_err(de::Error::custom)?;
        bz.try_into()
            .map_err(|y: <T as TryFrom<Vec<u8>>>::Error| de::Error::custom(format!("{y:?}")))
    }
}

pub mod hex_allow_unprefixed {
    use alloc::{format, string::String, vec::Vec};
    use core::fmt::Debug;

    use serde::{de, Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        crate::hex_string::serialize(data, serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>, Error: Debug + 'static>,
    {
        let s = String::deserialize(deserializer)?;
        let s = s.strip_prefix("0x").unwrap_or(&s);

        let bz = hex::decode(s).map_err(de::Error::custom)?;
        bz.try_into()
            .map_err(|y: <T as TryFrom<Vec<u8>>>::Error| de::Error::custom(format!("{y:?}")))
    }
}

pub mod hex_string_list {
    use alloc::{format, string::String, vec::Vec};
    use core::fmt::Debug;

    use serde::{de, Deserialize, Deserializer, Serializer};

    use crate::parse_hex;

    pub fn serialize<S, T, C>(list: &C, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: AsRef<[u8]>,
        for<'a> &'a C: IntoIterator<Item = &'a T>,
    {
        serializer.collect_seq(list.into_iter().map(crate::to_hex))
    }

    pub fn deserialize<'de, D, T, C>(deserializer: D) -> Result<C, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<u8>, Error: Debug + 'static>,
        C: TryFrom<Vec<T>, Error: Debug>,
    {
        Vec::<String>::deserialize(deserializer)?
            .into_iter()
            .map(|x| parse_hex::<T>(x).map_err(de::Error::custom))
            .collect::<Result<Vec<_>, D::Error>>()?
            .try_into()
            .map_err(|err| de::Error::custom(format!("failed to collect list: {err:#?}")))
    }
}

pub mod string {
    use alloc::string::String;
    use core::{fmt::Display, str::FromStr};

    use serde::{de::Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        serializer.collect_str(&data)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
    {
        String::deserialize(deserializer).and_then(|s| {
            s.parse()
                // TODO fix error situation
                // FromStr::Err has no bounds
                .map_err(|_| serde::de::Error::custom("failure to parse string data"))
        })
    }
}

pub mod map_numeric_keys_as_string {
    use alloc::{
        collections::BTreeMap,
        string::{String, ToString},
    };
    use core::{fmt::Display, str::FromStr};

    use serde::{de::Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S, M, K, V>(data: M, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        K: Display,
        V: Serialize,
        M: IntoIterator<Item = (K, V)> + Copy,
    {
        serializer.collect_map(data.into_iter().map(|(k, v)| (k.to_string(), v)))
    }

    pub fn deserialize<'de, D, M, K, V>(deserializer: D) -> Result<M, D::Error>
    where
        D: Deserializer<'de>,
        K: FromStr,
        V: Deserialize<'de>,
        M: FromIterator<(K, V)>,
    {
        <BTreeMap<String, V>>::deserialize(deserializer).and_then(|s| {
            s.into_iter()
                .map(|(k, v)| {
                    Ok((
                        k.parse()
                            // TODO fix error situation
                            // FromStr::Err has no bounds
                            .map_err(|_| {
                                serde::de::Error::custom("failure to parse string data")
                            })?,
                        v,
                    ))
                })
                .collect()
        })
    }
}

pub mod string_opt {
    use alloc::string::String;
    use core::{fmt::Display, str::FromStr};

    use serde::{de::Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Display,
    {
        if let Some(data) = data {
            serializer.collect_str(&data)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: FromStr,
    {
        String::deserialize(deserializer).and_then(|s| {
            s.parse()
                .map(Some)
                // TODO fix error situation
                // FromStr::Err has no bounds
                .map_err(|_| serde::de::Error::custom("failure to parse string data"))
        })
    }
}

pub mod u256_from_dec_str {
    #![allow(clippy::disallowed_types)] // need to access the inner type to do ser/de

    use alloc::{format, string::String};

    use primitive_types::U256;
    use serde::{de::Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.collect_str(&data)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            U256::from_dec_str(&s).map_err(|err| {
                serde::de::Error::custom(format!("failure to parse string data: {err}"))
            })
        })
    }
}

pub mod bitvec_string {
    use alloc::string::String;

    use bitvec::vec::BitVec;
    use serde::{
        de::{self, Deserialize},
        Deserializer, Serializer,
    };

    pub fn serialize<S>(data: &BitVec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let output = data
            .iter()
            .by_refs()
            // REVIEW: Is string literal or char more efficient?
            .map(|bit| if *bit { '1' } else { '0' })
            .collect::<String>();

        serializer.serialize_str(&output)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<BitVec<u8>, D::Error>
    where
        D: Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            s.chars()
                .map(|c| match c {
                    '0' => Ok(false),
                    '1' => Ok(true),
                    _ => Err(de::Error::invalid_value(
                        de::Unexpected::Char(c),
                        &"string containing only 1s and 0s",
                    )),
                })
                .collect::<Result<BitVec<u8>, _>>()
        })
    }
}

// This is used for the very strange representation of nil protobuf timestamps in cometbft json responses
#[allow(non_snake_case)]
pub mod parse_from_rfc3339_string_but_0001_01_01T00_00_00Z_is_none {
    use alloc::{format, string::String};
    use core::fmt::Debug;

    use chrono::{DateTime, SecondsFormat, Utc};
    use serde::{de::Deserialize, Deserializer, Serializer};

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
        T: Clone,
        DateTime<Utc>: TryFrom<T, Error: Debug>,
    {
        match data {
            Some(data) => {
                serializer.collect_str(
                    &<DateTime<Utc>>::try_from(data.clone())
                        .map_err(|err| {
                            serde::ser::Error::custom(format!(
                                "unable to convert to datetime: {err:?}"
                            ))
                        })?
                        .to_rfc3339_opts(
                            SecondsFormat::Nanos,
                            // use_z
                            true,
                        ),
                )
            }
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<DateTime<Utc>, Error: Debug>,
    {
        String::deserialize(deserializer).and_then(|s| {
            if s == "0001-01-01T00:00:00Z" {
                Ok(None)
            } else {
                let datetime = DateTime::parse_from_rfc3339(&s).map_err(|err| {
                    serde::de::Error::custom(format!("unable to parse data: {err:?}"))
                })?;

                Ok(Some(T::try_from(datetime.into()).map_err(|err| {
                    serde::de::Error::custom(format!(
                        "unable to convert data from rfc3339 datetime: {err:?}"
                    ))
                })?))
            }
        })
    }
}

pub mod fmt {
    use core::{
        fmt::{self, Write},
        marker::PhantomData,
    };

    use base64::Engine;
    use bitvec::{order::BitOrder, store::BitStore, view::AsBits};

    use crate::to_hex;

    pub struct DebugAsHex<T>(pub T);
    impl<T: AsRef<[u8]>> fmt::Debug for DebugAsHex<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", to_hex(&self.0))
        }
    }

    pub struct DebugAsBase64<T>(pub T);
    impl<T: AsRef<[u8]>> fmt::Debug for DebugAsBase64<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", base64::prelude::BASE64_STANDARD.encode(&self.0))
        }
    }

    pub struct DebugListAsHex<I>(pub I);
    impl<I> fmt::Debug for DebugListAsHex<I>
    where
        I: IntoIterator + Copy,
        I::Item: AsRef<[u8]>,
    {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.debug_list()
                .entries(self.0.into_iter().map(DebugAsHex::<I::Item>))
                .finish()
        }
    }

    pub struct DebugBits<A: AsBits<B>, B: BitStore, O: BitOrder>(
        pub A,
        PhantomData<fn() -> (B, O)>,
    );
    impl<A: AsBits<B>, B: BitStore, O: BitOrder> DebugBits<A, B, O> {
        pub fn new(a: A) -> Self {
            Self(a, PhantomData)
        }
    }
    impl<A: AsBits<B>, B: BitStore, O: BitOrder> fmt::Debug for DebugBits<A, B, O> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            for bit in self.0.as_bits::<O>().iter().by_refs() {
                // REVIEW: Is string literal or char more efficient?
                f.write_char(if *bit { '1' } else { '0' })?
            }

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        let string = to_hex([]);
        assert_eq!(string, "0x0");
        let bz = parse_hex::<alloc::vec::Vec<u8>>(string).unwrap();
        assert_eq!(bz, []);
    }
}
