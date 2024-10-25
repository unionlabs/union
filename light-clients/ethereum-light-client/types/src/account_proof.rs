use serde::{Deserialize, Serialize};
use unionlabs::hash::H256;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AccountProof {
    pub storage_root: H256,
    pub proof: Vec<Vec<u8>>,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::errors::InvalidLength;

    use crate::AccountProof;

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::AccountProof> for AccountProof {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::AccountProof,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                storage_root: value.storage_root.try_into().map_err(Error::StorageRoot)?,
                proof: value.proof,
            })
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error("invalid `contract_address`")]
        ContractAddress(#[source] InvalidLength),
        #[error("invalid `storage_root`")]
        StorageRoot(#[source] InvalidLength),
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
