use macros::model;

/// A BLS12381 public key
#[model(
    no_serde,
    proto(
        raw(protos::union::ibc::lightclients::movement::v1::PublicKey),
        into,
        from
    )
)]
pub struct PublicKey {
    pub pubkey: Vec<u8>,
}

impl From<PublicKey> for protos::union::ibc::lightclients::movement::v1::PublicKey {
    fn from(value: PublicKey) -> Self {
        Self {
            pubkey: value.pubkey,
        }
    }
}

impl From<protos::union::ibc::lightclients::movement::v1::PublicKey> for PublicKey {
    fn from(value: protos::union::ibc::lightclients::movement::v1::PublicKey) -> Self {
        Self {
            pubkey: value.pubkey,
        }
    }
}

impl serde::Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = format!("0x{}", hex::encode(&self.pubkey));
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
            let encoded_key = encoded_key.trim_start_matches("0x");
            Ok(PublicKey {
                pubkey: hex::decode(encoded_key)
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
