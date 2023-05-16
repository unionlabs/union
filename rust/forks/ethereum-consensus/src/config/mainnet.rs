use crate::{beacon::Version, fork::ForkParameters, preset, types::U64};

use super::Config;

pub const CONFIG: Config = Config {
    preset: preset::mainnet::PRESET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([0, 0, 0, 0]),
        genesis_slot: U64(0),

        altair_fork_version: Version([1, 0, 0, 0]),
        altair_fork_epoch: U64(74240),

        bellatrix_fork_version: Version([2, 0, 0, 0]),
        bellatrix_fork_epoch: U64(144896),

        capella_fork_version: Version([3, 0, 0, 0]),
        capella_fork_epoch: U64(194048),

        // NOTE: dummy data
        eip4844_fork_version: Version([4, 0, 0, 0]),
        eip4844_fork_epoch: U64(u64::MAX),
    },
    min_genesis_time: U64(1606824000),
};
