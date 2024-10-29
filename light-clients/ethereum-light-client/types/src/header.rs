use unionlabs::ibc::core::client::height::Height;

use crate::{AccountProof, LightClientUpdate};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    use unionlabs::{errors::MissingField, impl_proto_via_try_from_into, required};

    use crate::{account_proof, light_client_update, Header};

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::ethereum::v1::Header);

    impl From<Header> for protos::union::ibc::lightclients::ethereum::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                trusted_height: Some(value.trusted_height.into()),
                consensus_update: Some(value.consensus_update.into()),
                ibc_account_proof: Some(value.ibc_account_proof.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid consensus_update")]
        ConsensusUpdate(#[from] light_client_update::proto::Error),
        #[error("invalid ibc_account_update")]
        IbcAccountUpdate(#[from] account_proof::proto::Error),
    }

    impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::ethereum::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                trusted_height: required!(value.trusted_height)?.into(),
                consensus_update: required!(value.consensus_update)?.try_into()?,
                ibc_account_proof: required!(value.ibc_account_proof)?.try_into()?,
            })
        }
    }
}
