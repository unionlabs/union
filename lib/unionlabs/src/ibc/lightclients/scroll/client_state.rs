use alloc::sync::Arc;
use core::{fmt::Debug, str::FromStr};

use macros::model;
use uint::FromDecStrErr;

use crate::{errors::InvalidLength, hash::H160, ibc::core::client::height::Height, uint::U256};

#[model(proto(
    raw(protos::union::ibc::lightclients::scroll::v1::ClientState),
    into,
    from
))]
pub struct ClientState {
    // TODO: This should be ClientId
    pub l1_client_id: String,
    pub chain_id: U256,
    pub latest_slot: u64,
    pub frozen_height: Height,
    pub latest_batch_index_slot: U256,
    pub l2_contract_address: H160,
    pub l2_finalized_state_roots_slot: U256,
    pub l2_committed_batches_slot: U256,
    pub ibc_contract_address: H160,
    pub ibc_commitment_slot: U256,
}

impl From<ClientState> for protos::union::ibc::lightclients::scroll::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            l1_client_id: value.l1_client_id,
            chain_id: value.chain_id.to_string(),
            latest_slot: value.latest_slot,
            latest_batch_index_slot: value.latest_batch_index_slot.to_be_bytes().to_vec(),
            frozen_height: Some(value.frozen_height.into()),
            l2_contract_address: value.l2_contract_address.into(),
            l2_finalized_state_roots_slot: value.l2_finalized_state_roots_slot.to_be_bytes().into(),
            ibc_contract_address: value.ibc_contract_address.into(),
            ibc_commitment_slot: value.ibc_commitment_slot.to_be_bytes().into(),
            l2_committed_batches_slot: value.l2_committed_batches_slot.to_be_bytes().into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error("unable to parse chain id")]
    ChainId(#[source] Arc<FromDecStrErr>),
    #[error("invalid latest batch index slot")]
    LatestBatchIndexSlot(#[source] InvalidLength),
    #[error("invalid rollup contract address")]
    RollupContractAddress(#[source] InvalidLength),
    #[error("invalid rollup finalized state roots slot")]
    RollupFinalizedStateRootsSlot(#[source] InvalidLength),
    #[error("invalid ibc contract address")]
    IbcContractAddress(#[source] InvalidLength),
    #[error("invalid ibc commitment slot")]
    IbcCommitmentSlot(#[source] InvalidLength),
    #[error("invalid ibc committed batches slot")]
    RollupCommittedBatchesSlot(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: value.l1_client_id,
            chain_id: U256::from_str(&value.chain_id)
                .map_err(|err| TryFromClientStateError::ChainId(Arc::new(err)))?,
            latest_slot: value.latest_slot,
            latest_batch_index_slot: U256::try_from_be_bytes(&value.latest_batch_index_slot)
                .map_err(TryFromClientStateError::LatestBatchIndexSlot)?,
            frozen_height: value.frozen_height.unwrap_or_default().into(),
            l2_contract_address: value
                .l2_contract_address
                .try_into()
                .map_err(TryFromClientStateError::RollupContractAddress)?,
            l2_finalized_state_roots_slot: U256::try_from_be_bytes(
                &value.l2_finalized_state_roots_slot,
            )
            .map_err(TryFromClientStateError::RollupFinalizedStateRootsSlot)?,
            ibc_contract_address: value
                .ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::IbcContractAddress)?,
            ibc_commitment_slot: U256::try_from_be_bytes(&value.ibc_commitment_slot)
                .map_err(TryFromClientStateError::IbcCommitmentSlot)?,
            l2_committed_batches_slot: U256::try_from_be_bytes(&value.l2_committed_batches_slot)
                .map_err(TryFromClientStateError::RollupCommittedBatchesSlot)?,
        })
    }
}
