use macros::model;
use uint::FromDecStrErr;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::H160,
    ibc::core::client::height::Height,
    id::{ClientId, ClientIdValidator},
    uint::U256,
    validated::{Validate, ValidateT},
};

#[model(proto(
    raw(protos::union::ibc::lightclients::arbitrum::v1::ClientState),
    from,
    into
))]
pub struct ClientState {
    pub l1_client_id: ClientId,
    pub chain_id: U256,
    pub l1_latest_slot: u64,
    pub l1_contract_address: H160,
    pub l1_latest_confirmed_slot: U256,
    pub l1_nodes_slot: U256,
    // TODO: Rename this in the protos
    pub l1_nodes_confirm_data_offset: U256,
    pub frozen_height: Height,
    pub l2_ibc_contract_address: H160,
    pub l2_ibc_commitment_slot: U256,
}

impl TryFrom<protos::union::ibc::lightclients::arbitrum::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::arbitrum::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            l1_client_id: value
                .l1_client_id
                .validate()
                .map_err(TryFromClientStateError::L1ClientId)?,
            chain_id: value
                .chain_id
                .parse()
                .map_err(TryFromClientStateError::ChainId)?,
            l1_latest_slot: value.l1_latest_slot,
            l1_contract_address: value
                .l1_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L1ContractAddress)?,
            l1_latest_confirmed_slot: U256::try_from_be_bytes(&value.l1_latest_confirmed_slot)
                .map_err(TryFromClientStateError::L1LatestConfirmedSlot)?,
            l1_nodes_slot: U256::try_from_be_bytes(&value.l1_nodes_slot)
                .map_err(TryFromClientStateError::L1NodesSlot)?,
            l1_nodes_confirm_data_offset: U256::try_from_be_bytes(&value.confirm_data_offset)
                .map_err(TryFromClientStateError::ConfirmDataOffset)?,
            frozen_height: required!(value.frozen_height)?.into(),
            l2_ibc_contract_address: value
                .l2_ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::L2IbcContractAddress)?,
            l2_ibc_commitment_slot: U256::try_from_be_bytes(&value.l2_ibc_commitment_slot)
                .map_err(TryFromClientStateError::L2IbcContractAddress)?,
        })
    }
}

#[derive(Debug, PartialEq, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error(transparent)]
    MissingField(MissingField),
    #[error("invalid l1 client id")]
    L1ClientId(#[source] <ClientIdValidator as Validate<String>>::Error),
    #[error("invalid l1 contract address")]
    ChainId(#[source] FromDecStrErr),
    #[error("invalid l1 latest confirmed slot")]
    L1ContractAddress(#[source] InvalidLength),
    #[error("invalid l1 nodes slot")]
    L1LatestConfirmedSlot(#[source] InvalidLength),
    #[error("invalid confirm data offset")]
    L1NodesSlot(#[source] InvalidLength),
    #[error("invalid frozen height")]
    ConfirmDataOffset(#[source] InvalidLength),
    #[error("invalid l2 ibc commitment slot")]
    L2IbcContractAddress(#[source] InvalidLength),
}

impl From<ClientState> for protos::union::ibc::lightclients::arbitrum::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            l1_client_id: value.l1_client_id.to_string(),
            chain_id: value.chain_id.to_string(),
            l1_latest_slot: value.l1_latest_slot,
            l1_contract_address: value.l1_contract_address.into(),
            l1_latest_confirmed_slot: value.l1_latest_confirmed_slot.to_be_bytes().to_vec(),
            l1_nodes_slot: value.l1_nodes_slot.to_be_bytes().to_vec(),
            confirm_data_offset: value.l1_nodes_confirm_data_offset.to_be_bytes().to_vec(),
            frozen_height: Some(value.frozen_height.into()),
            l2_ibc_contract_address: value.l2_ibc_contract_address.into(),
            l2_ibc_commitment_slot: value.l2_ibc_commitment_slot.to_be_bytes().to_vec(),
        }
    }
}
