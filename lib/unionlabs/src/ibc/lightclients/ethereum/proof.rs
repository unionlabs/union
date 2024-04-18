use macros::model;

use crate::{errors::InvalidLength, uint::U256};

#[model(proto(raw(protos::union::ibc::lightclients::ethereum::v1::Proof), into, from))]
pub struct Proof {
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub key: U256,
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub value: U256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub enum TryFromProofError {
    Key(InvalidLength),
    Value(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Proof> for Proof {
    type Error = TryFromProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::Proof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            key: U256::try_from_be_bytes(&value.key).map_err(TryFromProofError::Key)?,
            value: U256::try_from_be_bytes(&value.value).map_err(TryFromProofError::Value)?,
            proof: value.proof,
        })
    }
}

impl From<Proof> for protos::union::ibc::lightclients::ethereum::v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            key: value.key.to_be_bytes().into(),
            value: value.value.to_be_bytes().into(),
            proof: value.proof,
        }
    }
}
