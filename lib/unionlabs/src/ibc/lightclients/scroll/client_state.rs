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
    pub l1_client_id: String,
    pub chain_id: U256,
    pub latest_batch_index: u64,
    pub latest_batch_index_slot: U256,
    pub frozen_height: Height,
    pub rollup_contract_address: H160,
    pub rollup_finalized_state_roots_slot: U256,
    pub ibc_contract_address: H160,
    pub ibc_commitment_slot: U256,
}

impl From<ClientState> for protos::union::ibc::lightclients::scroll::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            l1_client_id: value.l1_client_id,
            chain_id: value.chain_id.to_string(),
            latest_batch_index: value.latest_batch_index,
            latest_batch_index_slot: value.latest_batch_index_slot.to_be_bytes().to_vec(),
            frozen_height: Some(value.frozen_height.into()),
            rollup_contract_address: value.rollup_contract_address.into(),
            rollup_finalized_state_roots_slot: value
                .rollup_finalized_state_roots_slot
                .to_be_bytes()
                .into(),
            ibc_contract_address: value.ibc_contract_address.into(),
            ibc_commitment_slot: value.ibc_commitment_slot.to_be_bytes().into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromClientStateError {
    L1ClientId(FromDecStrErr),
    ChainId(FromDecStrErr),
    LatestBatchIndexSlot(InvalidLength),
    RollupContractAddress(InvalidLength),
    RollupFinalizedStateRootsSlot(InvalidLength),
    IbcContractAddress(InvalidLength),
    IbcCommitmentSlot(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::scroll::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::scroll::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: value.l1_client_id,
            chain_id: U256::from_str(&value.chain_id).map_err(TryFromClientStateError::ChainId)?,
            latest_batch_index: value.latest_batch_index,
            latest_batch_index_slot: U256::try_from_be_bytes(&value.latest_batch_index_slot)
                .map_err(TryFromClientStateError::LatestBatchIndexSlot)?,
            frozen_height: value.frozen_height.unwrap_or_default().into(),
            rollup_contract_address: value
                .rollup_contract_address
                .try_into()
                .map_err(TryFromClientStateError::RollupContractAddress)?,
            rollup_finalized_state_roots_slot: U256::try_from_be_bytes(
                &value.rollup_finalized_state_roots_slot,
            )
            .map_err(TryFromClientStateError::RollupFinalizedStateRootsSlot)?,
            ibc_contract_address: value
                .ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::IbcContractAddress)?,
            ibc_commitment_slot: U256::try_from_be_bytes(&value.ibc_commitment_slot)
                .map_err(TryFromClientStateError::IbcCommitmentSlot)?,
        })
    }
}
