use custom_debug_derive::Debug;
use macros::model;
use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountProof),
    into,
    from
))]
pub struct AccountProof {
    pub storage_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(with = "::serde_utils::fmt::hex_list")]
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug)]
pub enum TryFromAccountProofError {
    ContractAddress(InvalidLength),
    StorageRoot(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::AccountProof> for AccountProof {
    type Error = TryFromAccountProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::AccountProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            storage_root: value
                .storage_root
                .try_into()
                .map_err(TryFromAccountProofError::StorageRoot)?,
            proof: value.proof,
        })
    }
}

impl From<AccountProof> for protos::union::ibc::lightclients::ethereum::v1::AccountProof {
    fn from(value: AccountProof) -> Self {
        Self {
            storage_root: value.storage_root.into(),
            proof: value.proof,
        }
    }
}
