use hex::FromHexError;
use thiserror::Error;

const HEX_ENCODING_PREFIX: &str = "0x";

#[derive(Debug, Error)]
pub enum HexError {
    #[error("{0}")]
    Hex(#[from] FromHexError),
    #[error("missing prefix `{HEX_ENCODING_PREFIX}` when deserializing hex data")]
    MissingPrefix,
}

pub fn try_bytes_from_hex_str(s: &str) -> Result<Vec<u8>, HexError> {
    let target = s
        .strip_prefix(HEX_ENCODING_PREFIX)
        .ok_or(HexError::MissingPrefix)?;
    let data = hex::decode(target)?;
    Ok(data)
}

pub mod as_hex {
    use super::*;
    use serde::de::Deserialize;

    pub fn serialize<S, T: AsRef<[u8]>>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let encoding = hex::encode(data.as_ref());
        let output = format!("{HEX_ENCODING_PREFIX}{encoding}");
        serializer.collect_str(&output)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: TryFrom<Vec<u8>>,
    {
        let s = <String>::deserialize(deserializer)?;

        let data = try_bytes_from_hex_str(&s).map_err(serde::de::Error::custom)?;

        let inner = T::try_from(data)
            .map_err(|_| serde::de::Error::custom("type failed to parse bytes from hex data"))?;
        Ok(inner)
    }
}

pub mod as_string {
    use serde::de::Deserialize;
    use std::fmt;
    use std::str::FromStr;

    pub fn serialize<S, T: fmt::Display>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let output = format!("{data}");
        serializer.collect_str(&output)
    }

    pub fn deserialize<'de, D, T: FromStr>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = <String>::deserialize(deserializer)?;
        let inner: T = s
            .parse()
            // TODO fix error situation
            // FromStr::Err has no bounds
            .map_err(|_| serde::de::Error::custom("failure to parse string data"))?;
        Ok(inner)
    }
}

pub mod collection_over_string {
    use serde::de::{Deserializer, Error};
    use serde::ser::SerializeSeq;
    use std::fmt;
    use std::marker::PhantomData;
    use std::str::FromStr;

    pub fn serialize<S, T, U>(data: T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
        T: AsRef<[U]>,
        U: fmt::Display,
    {
        let mut seq = serializer.serialize_seq(None)?;
        for elem in data.as_ref().iter() {
            let rendered_elem = format!("{elem}");
            seq.serialize_element(&rendered_elem)?;
        }
        seq.end()
    }

    struct Visitor<T>(PhantomData<Vec<T>>);

    impl<'de, T: FromStr> serde::de::Visitor<'de> for Visitor<T> {
        type Value = Vec<T>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("array of string")
        }

        fn visit_seq<S>(self, mut access: S) -> Result<Self::Value, S::Error>
        where
            S: serde::de::SeqAccess<'de>,
        {
            let mut coll = Vec::with_capacity(access.size_hint().unwrap_or(0));

            while let Some(elem) = access.next_element()? {
                let recovered_elem = T::from_str(elem).map_err(|_| {
                    Error::custom("failure to parse element of sequence from string")
                })?;
                coll.push(recovered_elem);
            }
            Ok(coll)
        }
    }

    pub fn deserialize<'de, D, T, U>(deserializer: D) -> Result<T, D::Error>
    where
        D: Deserializer<'de>,
        T: TryFrom<Vec<U>>,
        U: FromStr,
    {
        let data = deserializer.deserialize_seq(Visitor(PhantomData))?;
        T::try_from(data).map_err(|_| serde::de::Error::custom("failure to parse collection"))
    }
}
