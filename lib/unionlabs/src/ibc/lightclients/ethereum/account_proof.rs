use macros::model;

use crate::hash::H256;

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::AccountProof),
    into,
    from
))]
pub struct AccountProof {
    pub storage_root: H256,
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string_list"))]
    #[debug(wrap = ::serde_utils::fmt::DebugListAsHex)]
    pub proof: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use crate::{errors::InvalidLength, ibc::lightclients::ethereum::account_proof::AccountProof};

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum TryFromAccountProofError {
        #[error("invalid `contract_address`")]
        ContractAddress(#[source] InvalidLength),
        #[error("invalid `storage_root`")]
        StorageRoot(#[source] InvalidLength),
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
}
