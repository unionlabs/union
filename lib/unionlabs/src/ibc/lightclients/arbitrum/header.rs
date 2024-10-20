use macros::model;

use crate::{
    errors::{required, MissingField},
    ibc::{
        core::client::height::Height,
        lightclients::{
            arbitrum::l2_header::{L2Header, TryFromL2HeaderError},
            ethereum::{
                account_proof::{AccountProof, TryFromAccountProofError},
                storage_proof::{StorageProof, TryFromStorageProofError},
            },
        },
    },
};

#[model(proto(
    raw(protos::union::ibc::lightclients::arbitrum::v1::Header),
    into,
    from
))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    pub l2_ibc_account_proof: AccountProof,
    pub l1_next_node_num_slot_proof: StorageProof,
    pub l1_nodes_slot_proof: StorageProof,
    pub l2_header: L2Header,
}

impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::arbitrum::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l1_account_proof: required!(value.l1_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1AccountProof)?,
            l2_ibc_account_proof: required!(value.l2_ibc_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2IbcAccountProof)?,
            l1_next_node_num_slot_proof: required!(value.l1_next_node_num_slot_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1NextNodeNumSlotProof)?,
            l1_nodes_slot_proof: required!(value.l1_nodes_slot_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1NodesSlotProof)?,
            l2_header: required!(value.l2_header)?
                .try_into()
                .map_err(TryFromHeaderError::L2Header)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid l1_account_proof")]
    L1AccountProof(#[source] TryFromAccountProofError),
    #[error("invalid l2_ibc_account_proof")]
    L2IbcAccountProof(#[source] TryFromAccountProofError),
    #[error("invalid l1_next_node_num_slot_proof")]
    L1NextNodeNumSlotProof(#[source] TryFromStorageProofError),
    #[error("invalid l1_nodes_slot_proof")]
    L1NodesSlotProof(#[source] TryFromStorageProofError),
    #[error("invalid l2_header")]
    L2Header(#[source] TryFromL2HeaderError),
}

impl From<Header> for protos::union::ibc::lightclients::arbitrum::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_account_proof: Some(value.l1_account_proof.into()),
            l2_ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
            l1_next_node_num_slot_proof: Some(value.l1_next_node_num_slot_proof.into()),
            l1_nodes_slot_proof: Some(value.l1_nodes_slot_proof.into()),
            l2_header: Some(value.l2_header.into()),
        }
    }
}
