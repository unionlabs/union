use super::bit_vec::BitVec;

#[derive(Debug, Clone, Eq, PartialEq)]
/// Either (1) a BLS signature share from an individual signer, (2) a BLS multisignature or (3) a
/// BLS aggregate signature
pub struct Signature {
    pub sig: Vec<u8>,
}

impl From<Signature> for protos::union::ibc::lightclients::movement::v1::Signature {
    fn from(value: Signature) -> Self {
        Self { sig: value.sig }
    }
}

impl From<protos::union::ibc::lightclients::movement::v1::Signature> for Signature {
    fn from(value: protos::union::ibc::lightclients::movement::v1::Signature) -> Self {
        Self { sig: value.sig }
    }
}

impl serde::Serialize for Signature {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        if serializer.is_human_readable() {
            let s = format!("0x{}", hex::encode(&self.sig));
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
            let encoded_key = encoded_key.trim_start_matches("0x");
            Ok(Signature {
                sig: hex::decode(encoded_key).map_err(<D::Error as ::serde::de::Error>::custom)?,
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

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct AggregateSignature {
    validator_bitmask: BitVec,
    sig: Option<Signature>,
}

impl From<AggregateSignature>
    for protos::union::ibc::lightclients::movement::v1::AggregateSignature
{
    fn from(value: AggregateSignature) -> Self {
        Self {
            validator_bitmask: value.validator_bitmask.inner,
            sig: value.sig.map(Into::into),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TryFromAggregateSignatureError {}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::AggregateSignature>
    for AggregateSignature
{
    type Error = TryFromAggregateSignatureError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::AggregateSignature,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            validator_bitmask: BitVec {
                inner: value.validator_bitmask,
            },
            sig: value.sig.map(Into::into),
        })
    }
}
