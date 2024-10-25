use alloc::sync::Arc;
use core::str::FromStr;

use macros::model;
use serde::{Deserialize, Serialize};
use uint::FromDecStrErr;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::{H160, H256},
    ibc::{
        core::client::height::Height,
        lightclients::ethereum::fork_parameters::{ForkParameters, TryFromForkParametersError},
    },
    uint::U256,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClientState {
    pub chain_id: U256,
    pub genesis_validators_root: H256,
    pub min_sync_committee_participants: u64,
    pub genesis_time: u64,
    pub fork_parameters: ForkParameters,
    pub seconds_per_slot: u64,
    pub slots_per_epoch: u64,
    pub epochs_per_sync_committee_period: u64,
    pub latest_slot: u64,
    // even though it would be better to have option, ethabicodec don't handle it as zero struct...
    pub frozen_height: Height,
    pub ibc_commitment_slot: U256,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
}

impl From<ClientState> for protos::union::ibc::lightclients::ethereum::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            chain_id: value.chain_id.to_string(),
            genesis_validators_root: value.genesis_validators_root.into(),
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: Some(value.fork_parameters.into()),
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            latest_slot: value.latest_slot,
            frozen_height: Some(value.frozen_height.into()),
            ibc_commitment_slot: value.ibc_commitment_slot.to_be_bytes().into(),
            ibc_contract_address: value.ibc_contract_address.into(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, thiserror::Error)]
pub enum TryFromClientStateError {
    #[error(transparent)]
    MissingField(#[from] MissingField),
    #[error("invalid chain id: {0:?}")]
    ChainId(Arc<FromDecStrErr>),
    #[error("invalid fork parameters")]
    ForkParameters(#[source] TryFromForkParametersError),
    #[error("invalid genesis validators root")]
    GenesisValidatorsRoot(#[source] InvalidLength),
    #[error("invalid ibc commitment slot")]
    IbcCommitmentSlot(#[source] InvalidLength),
    #[error("invalid ibc contract address")]
    IbcContractAddress(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: U256::from_str(&value.chain_id)
                .map_err(|err| TryFromClientStateError::ChainId(Arc::new(err)))?,
            genesis_validators_root: value
                .genesis_validators_root
                .try_into()
                .map_err(TryFromClientStateError::GenesisValidatorsRoot)?,
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: required!(value.fork_parameters)?
                .try_into()
                .map_err(TryFromClientStateError::ForkParameters)?,
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            latest_slot: value.latest_slot,
            frozen_height: value.frozen_height.unwrap_or_default().into(),
            ibc_commitment_slot: U256::try_from_be_bytes(&value.ibc_commitment_slot)
                .map_err(TryFromClientStateError::IbcCommitmentSlot)?,
            ibc_contract_address: value
                .ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::IbcContractAddress)?,
        })
    }
}
