use unionlabs::uint::U256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StorageProof {
    // #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub key: U256,
    // #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub value: U256,
    // #[serde(with = "::serde_utils::hex_string_list")]
    pub proof: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::{errors::InvalidLength, impl_proto_via_try_from_into, uint::U256};

    use crate::StorageProof;

    impl_proto_via_try_from_into!(StorageProof => protos::union::ibc::lightclients::ethereum::v1::StorageProof);

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::StorageProof> for StorageProof {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::StorageProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                key: U256::try_from_be_bytes(&value.key).map_err(Error::Key)?,
                value: U256::try_from_be_bytes(&value.value).map_err(Error::Value)?,
                proof: value.proof,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("unable to decode key")]
        Key(#[source] InvalidLength),
        #[error("unable to decode value")]
        Value(#[source] InvalidLength),
    }

    impl From<StorageProof> for protos::union::ibc::lightclients::ethereum::v1::StorageProof {
        fn from(value: StorageProof) -> Self {
            Self {
                key: value.key.to_be_bytes().into(),
                value: value.value.to_be_bytes().into(),
                proof: value.proof,
            }
        }
    }
}
