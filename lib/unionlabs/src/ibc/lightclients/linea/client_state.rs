use core::{fmt::Debug, str::FromStr};

use macros::model;
use uint::FromDecStrErr;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H160,
    ibc::core::client::height::Height,
    uint::U256,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::linea::v1::ClientState),
    into,
    from
))]
pub struct ClientState {
    pub chain_id: U256,
    // TODO: This should be ClientId
    pub l1_client_id: String,
    pub l1_latest_height: Height,
    pub l1_rollup_contract_address: H160,
    pub l1_rollup_current_l2_timestamp_slot: U256,
    pub l1_rollup_current_l2_block_number_slot: U256,
    pub l1_rollup_l2_state_root_hashes_slot: U256,
    pub l2_ibc_contract_address: H160,
    pub l2_ibc_contract_commitment_slot: U256,
    pub frozen_height: Height,
}

impl From<ClientState> for protos::union::ibc::lightclients::linea::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id.to_string(),
            l1_client_id: value.l1_client_id,
            l1_latest_height: Some(value.l1_latest_height.into()),
            l1_rollup_contract_address: value.l1_rollup_contract_address.into(),
            l1_rollup_current_l2_block_number_slot: value
                .l1_rollup_current_l2_block_number_slot
                .to_be_bytes()
                .to_vec(),
            l1_rollup_current_l2_timestamp_slot: value
                .l1_rollup_current_l2_timestamp_slot
                .to_be_bytes()
                .to_vec(),
            l1_rollup_l2_state_root_hashes_slot: value
                .l1_rollup_l2_state_root_hashes_slot
                .to_be_bytes()
                .to_vec(),
            l2_ibc_contract_address: value.l2_ibc_contract_address.into(),
            l2_ibc_contract_commitment_slot: value
                .l2_ibc_contract_commitment_slot
                .to_be_bytes()
                .into(),
            frozen_height: Some(value.frozen_height.into()),
        }
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error("unable to parse chain id")]
    ChainId(#[source] FromDecStrErr),
    #[error(transparent)]
    MissingField(MissingField),
    #[error("invalid l1 latest height")]
    L1LatestHeight,
    #[error("invalid rollup contract address")]
    L1RollupContractAddress(#[source] InvalidLength),
    #[error("invalid rollup current_l2_block_number slot")]
    L1RollupCurrentL2BlockNumberSlot(#[source] InvalidLength),
    #[error("invalid rollup current_l2_timestamp slot")]
    L1RollupCurrentL2TimestampSlot(#[source] InvalidLength),
    #[error("invalid rollup l2_state_roots mapping slot")]
    L1RollupL2StateRootHashesSlot(#[source] InvalidLength),
    #[error("invalid l2 ibc contract address")]
    L2IbcContractAddress(#[source] InvalidLength),
    #[error("invalid l2 ibc commitment slot")]
    L2IbcContractCommitmentSlot(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::linea::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::linea::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: value.l1_client_id,
            chain_id: U256::from_str(&value.chain_id).map_err(TryFromClientStateError::ChainId)?,
            l1_latest_height: required!(value.l1_latest_height)?.into(),
            l1_rollup_contract_address: value
                .l1_rollup_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L1RollupContractAddress)?,
            l1_rollup_current_l2_block_number_slot: U256::try_from_be_bytes(
                &value.l1_rollup_current_l2_block_number_slot,
            )
            .map_err(TryFromClientStateError::L1RollupCurrentL2BlockNumberSlot)?,
            l1_rollup_current_l2_timestamp_slot: U256::try_from_be_bytes(
                &value.l1_rollup_current_l2_timestamp_slot,
            )
            .map_err(TryFromClientStateError::L1RollupCurrentL2TimestampSlot)?,
            l1_rollup_l2_state_root_hashes_slot: U256::try_from_be_bytes(
                &value.l1_rollup_l2_state_root_hashes_slot,
            )
            .map_err(TryFromClientStateError::L1RollupL2StateRootHashesSlot)?,
            l2_ibc_contract_address: value
                .l2_ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L2IbcContractAddress)?,
            l2_ibc_contract_commitment_slot: U256::try_from_be_bytes(
                &value.l2_ibc_contract_commitment_slot,
            )
            .map_err(TryFromClientStateError::L2IbcContractCommitmentSlot)?,
            frozen_height: value.frozen_height.unwrap_or_default().into(),
        })
    }
}
