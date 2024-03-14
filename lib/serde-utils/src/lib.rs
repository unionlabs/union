extern crate alloc;

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

impl std::error::Error for FromHexStringError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FromHexStringError::Hex(hex) => Some(hex),
            FromHexStringError::EmptyString => None,
            FromHexStringError::MissingPrefix(_) => None,
            FromHexStringError::TryFromBytes(_) => None,
        }
    }
}

impl core::fmt::Display for FromHexStringError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
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
    T: TryFrom<Vec<u8>>,
    <T as TryFrom<Vec<u8>>>::Error: Debug + 'static,
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
    use alloc::{string::String, vec::Vec};
    use std::fmt::Debug;

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
        T: TryFrom<Vec<u8>>,
        <T as TryFrom<Vec<u8>>>::Error: Debug + 'static,
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

// https://play.rust-lang.org/?version=stable&mode=debug&edition=2018&gist=8b514073821e558a5ce862f64361492e
// will optimize this later
pub mod fixed_size_array {
    use std::{convert::TryInto, marker::PhantomData};

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

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
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
    use std::fmt::Debug;

    use serde::{de, Deserialize};

    use crate::{parse_hex, to_hex};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&to_hex(data))
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: TryFrom<Vec<u8>>,
        <T as TryFrom<Vec<u8>>>::Error: Debug + 'static,
    {
        String::deserialize(deserializer).and_then(|x| parse_hex::<T>(x).map_err(de::Error::custom))
    }
}

pub mod hex_upper_unprefixed {
    use std::fmt::Debug;

    use serde::{de, Deserialize};

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        if data.as_ref().is_empty() {
            serializer.serialize_str("0")
        } else {
            serializer.serialize_str(&hex::encode_upper(data))
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: TryFrom<Vec<u8>>,
        <T as TryFrom<Vec<u8>>>::Error: Debug + 'static,
    {
        let s = String::deserialize(deserializer)?;
        let bz = hex::decode(s).map_err(de::Error::custom)?;
        bz.try_into()
            .map_err(|y: <T as TryFrom<Vec<u8>>>::Error| de::Error::custom(format!("{y:?}")))
    }
}

pub mod hex_string_list {
    use alloc::{string::String, vec::Vec};
    use std::fmt::Debug;

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
        T: TryFrom<Vec<u8>>,
        <T as TryFrom<Vec<u8>>>::Error: Debug + 'static,
        C: TryFrom<Vec<T>>,
        <C as TryFrom<Vec<T>>>::Error: Debug,
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
    use std::{fmt, str::FromStr};

    use serde::de::Deserialize;

    pub fn serialize<S, T>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: fmt::Display,
    {
        serializer.collect_str(&data)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
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

pub mod string_opt {
    use std::{fmt, str::FromStr};

    use serde::de::Deserialize;

    pub fn serialize<S, T>(data: &Option<T>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: fmt::Display,
    {
        if let Some(data) = data {
            serializer.collect_str(&data)
        } else {
            serializer.serialize_none()
        }
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
    where
        D: serde::Deserializer<'de>,
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

    use primitive_types::U256;
    use serde::de::Deserialize;

    pub fn serialize<S>(data: &U256, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.collect_str(&data)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<U256, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|s| {
            U256::from_dec_str(&s)
                .map_err(|_| serde::de::Error::custom("failure to parse string data"))
        })
    }
}

pub mod bitvec_string {
    use bitvec::vec::BitVec;
    use serde::de::{self, Deserialize};

    pub fn serialize<S>(data: &BitVec<u8>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
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
        D: serde::Deserializer<'de>,
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

pub mod fmt {
    use core::fmt::Display;
    use std::fmt::Write;

    use bitvec::{order::BitOrder, view::AsBits};

    use crate::to_hex;

    pub fn hex<T: AsRef<[u8]>>(data: &T, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        write!(f, "{}", to_hex(data))
    }

    #[allow(clippy::ptr_arg)] // signature required by custom_debug_derive
    pub fn hex_list<T: AsRef<[u8]>>(
        data: &Vec<T>,
        f: &mut core::fmt::Formatter,
    ) -> core::fmt::Result {
        struct DebugAsHex<T>(T);

        impl<T: AsRef<[u8]>> core::fmt::Debug for DebugAsHex<T> {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{}", to_hex(&self.0))
            }
        }

        f.debug_list().entries(data.iter().map(DebugAsHex)).finish()
    }

    pub fn bits<B: BitOrder>(bitmap: &Vec<u8>, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        for bit in bitmap.as_bits::<B>().iter().by_refs() {
            // REVIEW: Is string literal or char more efficient?
            f.write_char(if *bit { '1' } else { '0' })?
        }

        Ok(())
    }

    pub fn display<T: Display>(t: &T, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        struct DebugAsDisplay<T>(T);

        impl<T: Display> core::fmt::Debug for DebugAsDisplay<T> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        write!(f, "{t}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex() {
        parse_hex::<Vec<u8>>(to_hex([])).unwrap();
    }
}
