use ethereum_consensus::{
    beacon::{Epoch, Root, Slot},
    config::Config,
    context::ChainContext,
    fork::ForkParameters,
    types::U64,
};
#[derive(Clone, Default, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}

impl Fraction {
    pub fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

pub trait ConsensusVerificationContext {
    fn genesis_validators_root(&self) -> Root;

    fn current_slot(&self) -> Slot;

    fn min_sync_committee_participants(&self) -> usize;

    fn signature_threshold(&self) -> Fraction;
}

pub struct LightClientContext<F> {
    fork_parameters: ForkParameters,
    seconds_per_slot: U64,
    slots_per_epoch: Slot,
    epochs_per_sync_committee_period: Epoch,
    genesis_time: U64,

    genesis_validators_root: Root,
    min_sync_committee_participants: usize,
    signature_threshold: Fraction,
    get_current_slot: F,
}

impl<F> LightClientContext<F> {
    pub fn new(
        fork_parameters: ForkParameters,
        seconds_per_slot: U64,
        slots_per_epoch: Slot,
        epochs_per_sync_committee_period: Epoch,
        genesis_time: U64,

        genesis_validators_root: Root,
        min_sync_committee_participants: usize,
        signature_threshold: Fraction,
        get_current_slot: F,
    ) -> Self {
        Self {
            fork_parameters,
            seconds_per_slot,
            slots_per_epoch,
            epochs_per_sync_committee_period,
            genesis_time,

            genesis_validators_root,
            min_sync_committee_participants,
            signature_threshold,
            get_current_slot,
        }
    }

    pub fn new_with_config(
        config: Config,
        genesis_validators_root: Root,
        genesis_time: U64,
        signature_threshold: Fraction,
        get_current_slot: F,
    ) -> Self {
        Self::new(
            config.fork_parameters,
            config.preset.SECONDS_PER_SLOT,
            config.preset.SLOTS_PER_EPOCH,
            config.preset.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
            genesis_time,
            genesis_validators_root,
            config.preset.MIN_SYNC_COMMITTEE_PARTICIPANTS,
            signature_threshold,
            get_current_slot,
        )
    }
}

impl<F> ConsensusVerificationContext for LightClientContext<F>
where
    F: Fn() -> Slot,
{
    fn genesis_validators_root(&self) -> Root {
        self.genesis_validators_root.clone()
    }

    fn min_sync_committee_participants(&self) -> usize {
        self.min_sync_committee_participants
    }

    fn signature_threshold(&self) -> Fraction {
        self.signature_threshold.clone()
    }

    fn current_slot(&self) -> Slot {
        (self.get_current_slot)()
    }
}

impl<F> ChainContext for LightClientContext<F> {
    fn genesis_time(&self) -> U64 {
        self.genesis_time
    }

    fn fork_parameters(&self) -> &ForkParameters {
        &self.fork_parameters
    }

    fn seconds_per_slot(&self) -> U64 {
        self.seconds_per_slot
    }

    fn slots_per_epoch(&self) -> Slot {
        self.slots_per_epoch
    }

    fn epochs_per_sync_committee_period(&self) -> Epoch {
        self.epochs_per_sync_committee_period
    }
}
