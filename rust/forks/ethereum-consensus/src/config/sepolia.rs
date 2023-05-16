use crate::{beacon::Version, fork::ForkParameters, preset, types::U64};

use super::Config;

pub const CONFIG: Config = Config {
    preset: preset::mainnet::PRESET,
    fork_parameters: ForkParameters {
        genesis_fork_version: Version([144, 0, 0, 105]),
        genesis_slot: U64(0),

        altair_fork_version: Version([144, 0, 0, 112]),
        altair_fork_epoch: U64(50),

        bellatrix_fork_version: Version([144, 0, 0, 113]),
        bellatrix_fork_epoch: U64(100),

        capella_fork_version: Version([144, 0, 0, 114]),
        capella_fork_epoch: U64(56832),

        // NOTE: dummy data
        eip4844_fork_version: Version([4, 0, 0, 0]),
        eip4844_fork_epoch: U64(u64::MAX),
    },
    min_genesis_time: U64(1655647200),
};
