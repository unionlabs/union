use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H256,
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            account_proof::{AccountProof, TryFromAccountProofError},
            storage_proof::{StorageProof, TryFromStorageProofError},
        },
    },
    linea::proof::{MerkleProof, TryFromMerkleProofError},
};

#[model(proto(raw(protos::union::ibc::lightclients::linea::v1::Header), into, from))]
pub struct Header {
    pub l1_height: Height,
    pub l1_rollup_contract_proof: AccountProof,
    pub l2_block_number: u64,
    pub l2_block_number_proof: StorageProof,
    pub l2_state_root: H256,
    pub l2_state_root_proof: StorageProof,
    pub l2_timestamp: u64,
    pub l2_timestamp_proof: StorageProof,
    pub l2_ibc_contract_proof: MerkleProof,
}

impl From<Header> for protos::union::ibc::lightclients::linea::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_rollup_contract_proof: Some(value.l1_rollup_contract_proof.into()),
            l2_block_number: value.l2_block_number,
            l2_block_number_proof: Some(value.l2_block_number_proof.into()),
            l2_state_root: value.l2_state_root.into(),
            l2_state_root_proof: Some(value.l2_state_root_proof.into()),
            l2_timestamp: value.l2_timestamp,
            l2_timestamp_proof: Some(value.l2_timestamp_proof.into()),
            l2_ibc_contract_proof: Some(value.l2_ibc_contract_proof.into()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TryFromHeaderError {
    MissingField(MissingField),
    L1RollupContractProof(TryFromAccountProofError),
    L2BlockNumber(InvalidLength),
    L2BlockNumberProof(TryFromStorageProofError),
    L2StateRoot(InvalidLength),
    L2StateRootProof(TryFromStorageProofError),
    L2Timestamp(InvalidLength),
    L2TimestampProof(TryFromStorageProofError),
    L2IbcContractProof(TryFromMerkleProofError),
}

impl TryFrom<protos::union::ibc::lightclients::linea::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::linea::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l1_rollup_contract_proof: required!(value.l1_rollup_contract_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1RollupContractProof)?,
            l2_block_number: value.l2_block_number,
            l2_block_number_proof: required!(value.l2_block_number_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2BlockNumberProof)?,
            l2_state_root: value
                .l2_state_root
                .try_into()
                .map_err(TryFromHeaderError::L2StateRoot)?,
            l2_state_root_proof: required!(value.l2_state_root_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2StateRootProof)?,
            l2_timestamp: value.l2_timestamp,
            l2_timestamp_proof: required!(value.l2_timestamp_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2TimestampProof)?,
            l2_ibc_contract_proof: required!(value.l2_ibc_contract_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2IbcContractProof)?,
        })
    }
}
