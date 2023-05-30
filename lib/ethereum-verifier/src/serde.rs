use hex::FromHexError;

const HEX_ENCODING_PREFIX: &str = "0x";

#[derive(Debug)]
pub enum HexError {
    Hex(FromHexError),
    MissingPrefix,
}

impl core::fmt::Display for HexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HexError::Hex(e) => write!(f, "{e}"),
            HexError::MissingPrefix => write!(
                f,
                "missing prefix `{HEX_ENCODING_PREFIX}` when deserializing hex data"
            ),
        }
    }
}

impl From<FromHexError> for HexError {
    fn from(e: FromHexError) -> Self {
        Self::Hex(e)
    }
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
