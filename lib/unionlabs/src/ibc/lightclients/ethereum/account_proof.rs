use macros::model;

use crate::{errors::InvalidLength, hash::H256};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountProof),
    into,
    from
))]
pub struct AccountProof {
    pub storage_root: H256,
    #[serde(with = "::serde_utils::hex_string_list")]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub proof: Vec<Vec<u8>>,
}

#[derive(Debug, PartialEq, Clone)]
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
