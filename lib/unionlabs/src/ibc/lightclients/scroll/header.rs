use macros::model;

use crate::{
    errors::{required, InvalidLength, MissingField},
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::{
            account_proof::{AccountProof, TryFromAccountProofError},
            storage_proof::{StorageProof, TryFromStorageProofError},
        },
    },
};

#[model(proto(raw(protos::union::ibc::lightclients::scroll::v1::Header), into, from))]
pub struct Header {
    pub l1_height: Height,
    pub l1_account_proof: AccountProof,
    /// This is the finalized state root proof, i.e. the l2 state on the l1
    pub l2_state_root_proof: StorageProof,
    pub last_batch_index_proof: StorageProof,
    pub batch_hash_proof: StorageProof,
    pub l2_ibc_account_proof: AccountProof,
    #[serde(with = "::serde_utils::hex_string")]
    #[debug(wrap = ::serde_utils::fmt::DebugAsHex)]
    pub batch_header: Vec<u8>,
}

impl From<Header> for protos::union::ibc::lightclients::scroll::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            l1_height: Some(value.l1_height.into()),
            l1_account_proof: Some(value.l1_account_proof.into()),
            l2_state_root_proof: Some(value.l2_state_root_proof.into()),
            last_batch_index_proof: Some(value.last_batch_index_proof.into()),
            l2_ibc_account_proof: Some(value.l2_ibc_account_proof.into()),
            batch_hash_proof: Some(value.batch_hash_proof.into()),
            batch_header: value.batch_header,
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid l1_account_proof")]
    L1AccountProof(#[source] TryFromAccountProofError),
    #[error("invalid l2_state_root")]
    L2StateRoot(#[source] InvalidLength),
    #[error("invalid l2_state_proof")]
    L2StateProof(#[source] TryFromStorageProofError),
    #[error("invalid last_batch_index_proof")]
    LastBatchIndexProof(#[source] TryFromStorageProofError),
    #[error("invalid l2_ibc_account_proof")]
    L2IbcAccountProof(#[source] TryFromAccountProofError),
    #[error("invalid batch_hash_proof")]
    BatchHashProof(#[source] TryFromStorageProofError),
    #[error("invalid l1_message_hash")]
    L1MessageHash(#[source] InvalidLength),
    #[error("invalid blob_versioned_hash")]
    BlobVersionedHash(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_height: required!(value.l1_height)?.into(),
            l1_account_proof: required!(value.l1_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L1AccountProof)?,
            l2_state_root_proof: required!(value.l2_state_root_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2StateProof)?,
            last_batch_index_proof: required!(value.last_batch_index_proof)?
                .try_into()
                .map_err(TryFromHeaderError::LastBatchIndexProof)?,
            l2_ibc_account_proof: required!(value.l2_ibc_account_proof)?
                .try_into()
                .map_err(TryFromHeaderError::L2IbcAccountProof)?,
            batch_hash_proof: required!(value.batch_hash_proof)?
                .try_into()
                .map_err(TryFromHeaderError::BatchHashProof)?,
            batch_header: value.batch_header,
        })
    }
}
