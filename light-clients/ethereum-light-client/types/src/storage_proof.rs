use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, uint::U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StorageProof {
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub key: U256,
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub value: U256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromStorageProofError {
    #[error("unable to decode key")]
    Key(#[source] InvalidLength),
    #[error("unable to decode value")]
    Value(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::StorageProof> for StorageProof {
    type Error = TryFromStorageProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::StorageProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: U256::try_from_be_bytes(&value.key).map_err(TryFromStorageProofError::Key)?,
            value: U256::try_from_be_bytes(&value.value)
                .map_err(TryFromStorageProofError::Value)?,
            proof: value.proof,
        })
    }
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
