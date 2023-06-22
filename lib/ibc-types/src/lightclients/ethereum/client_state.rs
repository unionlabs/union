use crate::{
    core::client::height::Height,
    errors::MissingField,
    lightclients::{ethereum::fork_parameters::ForkParameters, tendermint::fraction::Fraction},
    IntoProto, TryFromProto, TypeUrl,
};

#[derive(Debug, Clone)]
pub struct ClientState {
    pub genesis_validators_root: Vec<u8>,
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
    pub counterparty_commitment_slot: u64,
}

impl TypeUrl for protos::union::ibc::lightclients::ethereum::v1::ClientState {
    const TYPE_URL: &'static str = "/union.ibc.lightclients.ethereum.v1.ClientState";
}

impl IntoProto for ClientState {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ClientState;
}

impl From<ClientState> for protos::union::ibc::lightclients::ethereum::v1::ClientState {
    fn from(value: ClientState) -> Self {
        Self {
            genesis_validators_root: value.genesis_validators_root,
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
            counterparty_commitment_slot: value.counterparty_commitment_slot,
        }
    }
}

impl TryFrom<protos::union::ibc::lightclients::ethereum::v1::ClientState> for ClientState {
    type Error = MissingField;

    fn try_from(
        value: protos::union::ibc::lightclients::ethereum::v1::ClientState,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            genesis_validators_root: value.genesis_validators_root,
            min_sync_committee_participants: value.min_sync_committee_participants,
            genesis_time: value.genesis_time,
            fork_parameters: value
                .fork_parameters
                .ok_or(MissingField("fork_parameters"))?
                .try_into()?,
            seconds_per_slot: value.seconds_per_slot,
            slots_per_epoch: value.slots_per_epoch,
            epochs_per_sync_committee_period: value.epochs_per_sync_committee_period,
            trust_level: value.trust_level.ok_or(MissingField("trust_level"))?.into(),
            trusting_period: value.trusting_period,
            latest_slot: value.latest_slot,
            frozen_height: value.frozen_height.map(Into::into),
            counterparty_commitment_slot: value.counterparty_commitment_slot,
        })
    }
}

impl TryFromProto for ClientState {
    type Proto = protos::union::ibc::lightclients::ethereum::v1::ClientState;
}
