use std::str::FromStr;

use serde::{Deserialize, Serialize};
use uint::FromDecStrErr;

use crate::{
    errors::{InvalidLength, MissingField},
    ethereum::{H256, U256},
    ibc::{
        core::client::height::Height,
        lightclients::{ethereum::fork_parameters::ForkParameters, tendermint::fraction::Fraction},
    },
    Proto, TryFromProtoErrorOf, TypeUrl,
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
    pub trust_level: Fraction,
    pub trusting_period: u64,
    pub latest_slot: u64,
    pub frozen_height: Option<Height>,
    pub counterparty_commitment_slot: U256,
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::ClientState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.ClientState";
}

impl Proto for ClientState {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ClientState;
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
            frozen_height: value.frozen_height.map(Into::into),
            counterparty_commitment_slot: value.counterparty_commitment_slot.to_big_endian().into(),
        }
    }
}

#[derive(Debug)]
pub enum TryFromClientStateError {
    MissingField(MissingField),
    ChainId(FromDecStrErr),
    ForkParameters(TryFromProtoErrorOf<ForkParameters>),
    GenesisValidatorsRoot(InvalidLength),
    CounterpartyCommitmentSlot(InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ClientState> for ClientState {
    type Error = TryFromClientStateError;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            chain_id: U256::from_str(dbg!(&value.chain_id))
                .map_err(TryFromClientStateError::ChainId)?,
            genesis_validators_root: value
                .genesis_validators_root
                .try_into()
                .map_err(TryFromClientStateError::GenesisValidatorsRoot)?,
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: value
                .fork_parameters
                .ok_or(TryFromClientStateError::MissingField(MissingField(
                    "fork_parameters",
                )))?
                .try_into()
                .map_err(TryFromClientStateError::ForkParameters)?,
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            trust_level: value
                .trust_level
                .ok_or(TryFromClientStateError::MissingField(MissingField(
                    "trust_level",
                )))?
                .into(),
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: value.frozen_height.map(Into::into),
            counterparty_commitment_slot: U256::try_from_big_endian(
                &value.counterparty_commitment_slot,
            )
            .map_err(TryFromClientStateError::CounterpartyCommitmentSlot)?,
        })
    }
}
