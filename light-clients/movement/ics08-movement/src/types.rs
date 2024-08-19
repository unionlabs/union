use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AccountAddress(pub Vec<u8>);

#[derive(Debug, Clone, PartialEq, Eq)]
/// A BLS12381 public key
pub struct PublicKey {
    pub pubkey: Vec<u8>,
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = hex::encode(&self.pubkey);
            serializer.serialize_str(&s[..])
        } else {
            // See comment in deserialize_key.
            serializer.serialize_newtype_struct(
                "PublicKey",
                serde_bytes::Bytes::new(self.pubkey.as_slice()),
            )
        }
    }
}

impl<'de> serde::Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            Ok(PublicKey {
                pubkey: hex::decode(encoded_key.as_str())
                    .map_err(<D::Error as ::serde::de::Error>::custom)?,
            })
        } else {
            // In order to preserve the Serde data model and help analysis tools,
            // make sure to wrap our value in a container with the same name
            // as the original type.
            #[derive(::serde::Deserialize, Debug)]
            #[serde(rename = "PublicKey")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            Ok(PublicKey {
                pubkey: value.0.to_vec(),
            })
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// Either (1) a BLS signature share from an individual signer, (2) a BLS multisignature or (3) a
/// BLS aggregate signature
pub struct Signature {
    pub sig: Vec<u8>,
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = hex::encode(&self.sig);
            serializer.serialize_str(&s[..])
        } else {
            // See comment in deserialize_key.
            serializer
                .serialize_newtype_struct("Signature", serde_bytes::Bytes::new(self.sig.as_slice()))
        }
    }
}

impl<'de> serde::Deserialize<'de> for Signature {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: ::serde::Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            let encoded_key = <String>::deserialize(deserializer)?;
            Ok(Signature {
                sig: hex::decode(encoded_key.as_str())
                    .map_err(<D::Error as ::serde::de::Error>::custom)?,
            })
        } else {
            #[derive(::serde::Deserialize, Debug)]
            #[serde(rename = "Signature")]
            struct Value<'a>(&'a [u8]);

            let value = Value::deserialize(deserializer)?;
            Ok(Signature {
                sig: value.0.to_vec(),
            })
        }
    }
}
