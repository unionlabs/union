use core::str::FromStr;

use macros::model;
use uint::FromDecStrErr;

use crate::{
    errors::{required, InvalidLength, MissingField},
    hash::{H160, H256},
    ibc::{
        core::client::height::Height,
        lightclients::{
            ethereum::fork_parameters::{ForkParameters, TryFromForkParametersError},
            tendermint::fraction::{Fraction, TryFromFractionError},
        },
    },
    uint::U256,
};

#[model(proto(
    raw(protos::union::ibc::lightclients::ethereum::v1::ClientState),
    into,
    from
))]
pub struct ClientState {
    /// id of the tracked ethereum chain
    pub chain_id: U256,
    /// see: https://ethereum.github.io/beacon-APIs/#/Beacon/getGenesis
    pub genesis_validators_root: H256,
    /// minimum allowed sync committee validator signature
    pub min_sync_committee_participants: u64,
    /// see: https://ethereum.github.io/beacon-APIs/#/Beacon/getGenesis
    pub genesis_time: u64,
    /// defined under [`eth spec`](https://ethereum.github.io/beacon-APIs/#/Config/getSpec)
    pub fork_parameters: ForkParameters,
    pub seconds_per_slot: u64,
    /// number of slots per epoch
    pub slots_per_epoch: u64,
    /// number of epochs per sync committee period (signature period)
    pub epochs_per_sync_committee_period: u64,
    /// TODO(aeryz): remove unused
    pub trust_level: Fraction,
    /// if the client is not updated for `trusting_period` seconds, it expires and needs recovery
    pub trusting_period: u64,
    /// the highest slot that the client is updated to
    pub latest_slot: u64,
    // even though it would be better to have option, ethabicodec don't handle it as zero struct...
    /// whether the client is frozen or not
    pub frozen_height: Height,
    /// the slot in the counterparty ibc contract for the commitments mapping
    pub ibc_commitment_slot: U256,
    /// the ibc contract on the counterparty chain that contains the ICS23 commitments
    pub ibc_contract_address: H160,
    /// merkle tree index of the tracked header
    pub checkpoint_root_index: u64,
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
            trust_level: Some(value.trust_level.into()),
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: Some(value.frozen_height.into()),
            ibc_commitment_slot: value.ibc_commitment_slot.to_big_endian().into(),
            ibc_contract_address: value.ibc_contract_address.into(),
            checkpoint_root_index: value.checkpoint_root_index,
        }
    }
}

#[derive(Debug)]
pub enum TryFromClientStateError {
    MissingField(MissingField),
    ChainId(FromDecStrErr),
    ForkParameters(TryFromForkParametersError),
    GenesisValidatorsRoot(InvalidLength),
    IbcCommitmentSlot(InvalidLength),
    TrustLevel(TryFromFractionError),
    IbcContractAddress(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: U256::from_str(&value.chain_id).map_err(TryFromClientStateError::ChainId)?,
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
            trust_level: required!(value.trust_level)?
                .try_into()
                .map_err(TryFromClientStateError::TrustLevel)?,
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: value.frozen_height.unwrap_or_default().into(),
            ibc_commitment_slot: U256::try_from_big_endian(&value.ibc_commitment_slot)
                .map_err(TryFromClientStateError::IbcCommitmentSlot)?,
            ibc_contract_address: value
                .ibc_contract_address
                .try_into()
                .map_err(TryFromClientStateError::IbcContractAddress)?,
            checkpoint_root_index: value.checkpoint_root_index,
        })
    }
}
