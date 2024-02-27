use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{errors::InvalidLength, hash::H256};

#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct AccountProof {
    pub storage_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub proof: Vec<Vec<u8>>,
}

impl Debug for AccountProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountProof")
            .field("storage_root", &self.storage_root)
            .field(
                "proof",
                &self
                    .proof
                    .iter()
                    .map(serde_utils::to_hex)
                    .collect::<Vec<_>>(),
            )
            .finish()
    }
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
