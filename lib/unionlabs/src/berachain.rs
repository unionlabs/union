// use crate::ethereum::config::preset::{Preset, MAINNET};

// const BERACHAIN_CHAIN_SPEC: Preset = Preset {
//     SECONDS_PER_SLOT: 3,
//     ..MAINNET
// };

/// <https://github.com/berachain/beacon-kit/blob/8b706f49705b4ca5e88d1551c58db91b98d7eee7/mod/storage/pkg/beacondb/keys/keys.go#L41>
pub const LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX: u8 = 17;
/// <https://github.com/berachain/beacon-kit/blob/8b706f49705b4ca5e88d1551c58db91b98d7eee7/mod/storage/pkg/beacondb/keys/keys.go#L35>
pub const LATEST_BEACON_BLOCK_HEADER_PREFIX: u8 = 0x0b;

// #[derive(Debug, Clone, PartialEq, Eq, Default)]
// pub struct BerachainChainSpec;

// crate::ethereum::config::mk_chain_spec!(BerachainChainSpec is BERACHAIN_CHAIN_SPEC);
