use crate::{
    beacon::{Epoch, Slot},
    config::Config,
    fork::ForkParameters,
    types::U64,
};

pub trait ChainContext {
    fn genesis_time(&self) -> U64;
    fn fork_parameters(&self) -> &ForkParameters;
    fn seconds_per_slot(&self) -> U64;
    fn slots_per_epoch(&self) -> Slot;
    fn epochs_per_sync_committee_period(&self) -> Epoch;
}

pub struct DefaultChainContext {
    genesis_time: U64,
    fork_parameters: ForkParameters,
    seconds_per_slot: U64,
    slots_per_epoch: Slot,
    epochs_per_sync_committee_period: Epoch,
}

impl DefaultChainContext {
    pub fn new(
        genesis_time: U64,
        fork_parameters: ForkParameters,
        seconds_per_slot: U64,
        slots_per_epoch: Slot,
        epochs_per_sync_committee_period: Epoch,
    ) -> Self {
        Self {
            genesis_time,
            fork_parameters,
            seconds_per_slot,
            slots_per_epoch,
            epochs_per_sync_committee_period,
        }
    }

    pub fn new_with_config(genesis_time: U64, config: Config) -> Self {
        Self::new(
            genesis_time,
            config.fork_parameters,
            config.preset.SECONDS_PER_SLOT,
            config.preset.SLOTS_PER_EPOCH,
            config.preset.EPOCHS_PER_SYNC_COMMITTEE_PERIOD,
        )
    }
}

impl ChainContext for DefaultChainContext {
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
