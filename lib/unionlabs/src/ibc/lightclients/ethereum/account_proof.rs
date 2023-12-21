use std::fmt::Debug;

use serde::{Deserialize, Serialize};

use crate::{
    errors::InvalidLength,
    hash::{H160, H256},
};

// REVIEW: H256 or actual arbitrary bytes?
#[derive(Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AccountProof {
    pub contract_address: H160,
    pub storage_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub proof: Vec<Vec<u8>>,
}

impl Debug for AccountProof {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AccountProof")
            .field("contract_address", &self.contract_address)
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
            contract_address: value
                .contract_address
                .try_into()
                .map_err(TryFromAccountProofError::ContractAddress)?,
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
            contract_address: value.contract_address.into(),
            storage_root: value.storage_root.into(),
            proof: value.proof,
        }
    }
}
