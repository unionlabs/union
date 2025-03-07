use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateKey<T: bip32::PrivateKey> {
    /// The key stored in plaintext.
    Raw(#[serde(with = "private_key_hex_string")] T), // TODO: Other key types (i.e. keyring)
}

mod private_key_hex_string {
    use bip32::{PrivateKey, PrivateKeyBytes};
    use serde::de::Error;

    pub fn serialize<S, T: PrivateKey>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serde_utils::hex_string::serialize(data.to_bytes(), serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: PrivateKey,
    {
        serde_utils::hex_string::deserialize(deserializer).and_then(|data: PrivateKeyBytes| {
            <T as PrivateKey>::from_bytes(&data).map_err(|x| D::Error::custom(x.to_string()))
        })
    }
}

impl<T: bip32::PrivateKey> PrivateKey<T> {
    pub fn value(self) -> T {
        match self {
            PrivateKey::Raw(raw) => raw,
        }
    }
}
