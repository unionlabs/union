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
    pub latest_confirmed: u64,
    pub l1_latest_confirmed_slot_proof: StorageProof,
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
            latest_confirmed: value.latest_confirmed,
            l1_latest_confirmed_slot_proof: required!(value.l1_latest_confirmed_slot_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1LatestConfirmedSlotProof)?,
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
    MissingField(MissingField),
    #[error("invalid l1 account proof")]
    L1AccountProof(TryFromAccountProofError),
    #[error("invalid l2 ibc account proof")]
    L2IbcAccountProof(TryFromAccountProofError),
    #[error("invalid l1 latest confirmed slot proof")]
    L1LatestConfirmedSlotProof(TryFromStorageProofError),
    #[error("invalid l1 nodes slot proof")]
    L1NodesSlotProof(TryFromStorageProofError),
    #[error("invalid l2 header")]
    L2Header(TryFromL2HeaderError),
}

impl From<Header> for protos::union::ibc::lightclients::arbitrum::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_account_proof: Some(value.l1_account_proof.into()),
            l2_ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
            latest_confirmed: value.latest_confirmed,
            l1_latest_confirmed_slot_proof: Some(value.l1_latest_confirmed_slot_proof.into()),
            l1_nodes_slot_proof: Some(value.l1_nodes_slot_proof.into()),
            l2_header: Some(value.l2_header.into()),
        }
    }
}
