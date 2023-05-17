use crate::{beacon::Version, fork::ForkParameters, preset, types::U64};

use super::Config;

pub const CONFIG: Config = Config {
    preset: preset::minimal::PRESET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([0, 0, 0, 1]),
        genesis_slot: U64(0),

        altair_fork_version: Version([1, 0, 0, 1]),
        altair_fork_epoch: U64(0),

        bellatrix_fork_version: Version([2, 0, 0, 1]),
        bellatrix_fork_epoch: U64(0),

        capella_fork_version: Version([3, 0, 0, 1]),
        capella_fork_epoch: U64(0),

        // NOTE: dummy data
        eip4844_fork_version: Version([4, 0, 0, 1]),
        eip4844_fork_epoch: U64(u64::MAX),
    },
    min_genesis_time: U64(1578009600),
};
