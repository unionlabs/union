use ethereum_light_client_types::{AccountProof, StorageProof};
use unionlabs::{ibc::core::client::height::Height, linea::proof::InclusionProof};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l1_rollup_contract_proof: AccountProof,
    pub l2_timestamp_proof: StorageProof,
    pub l2_block_number_proof: StorageProof,
    pub l2_state_root_proof: StorageProof,
    pub l2_ibc_contract_proof: InclusionProof,
}

#[cfg(feature = "proto")]
pub mod proto {
    use ethereum_light_client_types::{account_proof, storage_proof};
    use unionlabs::{
        errors::{InvalidLength, MissingField},
        impl_proto_via_try_from_into,
        linea::proof::TryFromMerkleProofError,
        required,
    };

    use crate::Header;

    impl_proto_via_try_from_into!(Header => protos::union::ibc::lightclients::linea::v1::Header);

    impl From<Header> for protos::union::ibc::lightclients::linea::v1::Header {
        fn from(value: Header) -> Self {
            Self {
                l1_height: Some(value.l1_height.into()),
                l1_rollup_contract_proof: Some(value.l1_rollup_contract_proof.into()),
                l2_timestamp_proof: Some(value.l2_timestamp_proof.into()),
                l2_block_number_proof: Some(value.l2_block_number_proof.into()),
                l2_state_root_proof: Some(value.l2_state_root_proof.into()),
                l2_ibc_contract_proof: Some(value.l2_ibc_contract_proof.into()),
            }
        }
    }

    #[derive(Debug, PartialEq, Clone, thiserror::Error)]
    pub enum Error {
        #[error(transparent)]
        MissingField(#[from] MissingField),
        #[error("invalid l1_rollup_contract_proof")]
        L1RollupContractProof(#[source] account_proof::proto::Error),
        #[error("invalid l2_block_number")]
        L2BlockNumber(#[source] InvalidLength),
        #[error("invalid l2_block_number_proof")]
        L2BlockNumberProof(#[source] storage_proof::proto::Error),
        #[error("invalid l2_state_root")]
        L2StateRoot(#[source] InvalidLength),
        #[error("invalid l2_state_root_proof")]
        L2StateRootProof(#[source] storage_proof::proto::Error),
        #[error("invalid l2_timestamp")]
        L2Timestamp(#[source] InvalidLength),
        #[error("invalid l2_timestamp_proof")]
        L2TimestampProof(#[source] storage_proof::proto::Error),
        #[error("invalid l2_ibc_contract_proof")]
        L2IbcContractProof(#[source] TryFromMerkleProofError),
    }

    impl TryFrom<protos::union::ibc::lightclients::linea::v1::Header> for Header {
        type Error = Error;

        fn try_from(
            value: protos::union::ibc::lightclients::linea::v1::Header,
        ) -> Result<Self, Self::Error> {
            Ok(Self {
                l1_height: required!(value.l1_height)?.into(),
                l1_rollup_contract_proof: required!(value.l1_rollup_contract_proof)?
                    .try_into()
                    .map_err(Error::L1RollupContractProof)?,
                l2_timestamp_proof: required!(value.l2_timestamp_proof)?
                    .try_into()
                    .map_err(Error::L2TimestampProof)?,
                l2_block_number_proof: required!(value.l2_block_number_proof)?
                    .try_into()
                    .map_err(Error::L2BlockNumberProof)?,
                l2_state_root_proof: required!(value.l2_state_root_proof)?
                    .try_into()
                    .map_err(Error::L2StateRootProof)?,
                l2_ibc_contract_proof: required!(value.l2_ibc_contract_proof)?
                    .try_into()
                    .map_err(Error::L2IbcContractProof)?,
            })
        }
    }
}
