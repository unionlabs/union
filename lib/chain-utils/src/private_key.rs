use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrivateKey<T: bip32::PrivateKey> {
    /// The key stored in plaintext.
    Raw(#[serde(with = "private_key_hex_string")] T), // TODO: Other key types (i.e. keyring)
    /// The key mnemonic stored in plaintext.
    Mnemonic(#[serde(with = "private_key_mnemonic")] T), // TODO: Other key types (i.e. keyring)
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

mod private_key_mnemonic {
    use bip32::PrivateKey;
    use serde::{de::Error, Deserialize, Serialize};

    pub fn serialize<S, T: PrivateKey>(data: &T, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        bip39::Mnemonic::from_entropy(&data.to_bytes())
            .expect("private key bytes are valid mnemonic entropy")
            .serialize(serializer)
    }

    pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
    where
        D: serde::Deserializer<'de>,
        T: PrivateKey,
    {
        bip39::Mnemonic::deserialize(deserializer).and_then(|data| {
            <T as PrivateKey>::from_bytes(&data.to_entropy().try_into().unwrap())
                .map_err(|x| D::Error::custom(x.to_string()))
        })
    }
}

impl<T: bip32::PrivateKey> PrivateKey<T> {
    pub fn value(self) -> T {
        match self {
            PrivateKey::Raw(key) => key,
            PrivateKey::Mnemonic(key) => key,
        }
    }
}

#[cfg(test)]
mod serde_tests {
    use bip32::secp256k1::ecdsa;

    use super::*;

    #[test]
    fn raw() {
        let json =
            r#"{"raw":"0x4e9444a6efd6d42725a250b650a781da2737ea308c839eaccb0f7f3dbd2fea77"}"#;

        let key = serde_json::from_str::<PrivateKey<ecdsa::SigningKey>>(json).unwrap();

        assert_eq!(json, serde_json::to_string(&key).unwrap());
    }

    #[test]
    fn mnemonic() {
        let json = r#"{"mnemonic":"wine parrot nominee girl exchange element pudding grow area twenty next junior come render shadow evidence sentence start rough debate feed all limb real"}"#;

        let key = serde_json::from_str::<PrivateKey<ecdsa::SigningKey>>(json).unwrap();

        assert_eq!(json, serde_json::to_string(&key).unwrap());
    }
}
