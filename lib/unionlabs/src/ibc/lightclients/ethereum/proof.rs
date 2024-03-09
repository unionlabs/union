use custom_debug_derive::Debug;
use macros::proto;
use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, uint::U256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[proto(raw = protos::union::ibc::lightclients::ethereum::v1::Proof, into, from)]
pub struct Proof {
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub key: U256,
    #[serde(with = "crate::uint::u256_big_endian_hex")]
    pub value: U256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(with = "::serde_utils::fmt::hex_list")]
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
            key: U256::try_from_big_endian(&value.key).map_err(TryFromProofError::Key)?,
            value: U256::try_from_big_endian(&value.value).map_err(TryFromProofError::Value)?,
            proof: value.proof,
        })
    }
}

impl From<Proof> for protos::union::ibc::lightclients::ethereum::v1::Proof {
    fn from(value: Proof) -> Self {
        Self {
            key: value.key.to_big_endian().into(),
            value: value.value.to_big_endian().into(),
            proof: value.proof,
        }
    }
}
