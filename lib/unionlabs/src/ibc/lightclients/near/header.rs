use borsh::{BorshDeserialize, BorshSerialize};
use near_primitives_core::{hash::CryptoHash, types::BlockHeight};

use super::light_client_block::{LightClientBlockView, TryFromLightClientBlockView};
use crate::{
    errors::{required, MissingField},
    near::types::{MerklePath, TryFromMerklePathItemError},
};

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct Header {
    pub new_state: LightClientBlockView,
    pub trusted_height: BlockHeight,
    pub prev_state_root_proof: MerklePath,
    pub prev_state_root: CryptoHash,
}

impl From<Header> for protos::union::ibc::lightclients::near::v1::Header {
    fn from(value: Header) -> Self {
        Self {
            new_state: Some(value.new_state.into()),
            trusted_height: value.trusted_height,
            prev_state_root_proof: value
                .prev_state_root_proof
                .into_iter()
                .map(Into::into)
                .collect(),
            prev_state_root: value.prev_state_root.into(),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromHeaderError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error(transparent)]
    NewState(#[from] TryFromLightClientBlockView),
    #[error(transparent)]
    PrevStateRootProof(#[from] TryFromMerklePathItemError),
    #[error("invalid prev state root")]
    PrevStateRoot,
}

impl TryFrom<protos::union::ibc::lightclients::near::v1::Header> for Header {
    type Error = TryFromHeaderError;

    fn try_from(
        value: protos::union::ibc::lightclients::near::v1::Header,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            new_state: required!(value.new_state)?
                .try_into()
                .map_err(TryFromHeaderError::NewState)?,
            trusted_height: value.trusted_height,
            prev_state_root_proof: value
                .prev_state_root_proof
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromHeaderError::PrevStateRootProof)?,
            prev_state_root: value
                .prev_state_root
                .as_slice()
                .try_into()
                .map_err(|_| TryFromHeaderError::PrevStateRoot)?,
        })
    }
}
