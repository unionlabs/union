use crate::{
    ethereum::config::preset::{Preset, MAINNET},
    traits::FromStrExact,
};

const BERACHAIN_CHAIN_SPEC: Preset = Preset {
    SECONDS_PER_SLOT: 3,
    ..MAINNET
};

// TODO: Link to berachain/beacon-kit
pub const LATEST_EXECUTION_PAYLOAD_HEADER_PREFIX: u8 = 17;
pub const LATEST_BEACON_BLOCK_HEADER_PREFIX: u8 = 0x0b;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "arbitrary", derive(arbitrary::Arbitrary))]
pub struct BerachainChainSpec;

impl FromStrExact for BerachainChainSpec {
    const EXPECTING: &'static str = "berachain";
}

crate::ethereum::config::mk_chain_spec!(BerachainChainSpec is BERACHAIN_CHAIN_SPEC);
