use protos::ibc::core::client::v1::Height;
use serde::{Deserialize, Serialize};

use crate::{AccountProof, LightClientUpdate};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Header {
    /// The currently trusted height of the light client to apply this update against.
    pub trusted_height: Height,

    /// The actual update data to be applied.
    pub consensus_update: LightClientUpdate,

    /// Proof of the IBC handler contract against the execution state root provided in `consensus_update`.
    pub ibc_account_proof: AccountProof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use unionlabs::errors::MissingField;

    use crate::Header;

    impl From<Header> for protos::union::ibc::lightclients::ethereum::v1::Header {
        fn from(value: Header) -> Self {
            todo!()
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        // #[error("invalid `trusted_sync_committee`")]
        // TrustedSyncCommittee(#[from] TryFromTrustedSyncCommitteeError),
        // #[error("invalid `consensus_update`")]
        // ConsensusUpdate(#[from] TryFromLightClientUpdateError),
        // #[error("invalid `account_update`")]
        // AccountUpdate(#[from] TryFromAccountUpdateError),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::Header,
        ) -> Result<Self, Self::Error> {
            todo!()
        }
    }
}
