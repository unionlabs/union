use cw_storage_plus::Map;
use unionlabs::ibc::lightclients::near::validator_stake::ValidatorStakeView;

pub const EPOCH_BLOCK_PRODUCERS_MAP: Map<[u8; 32], Vec<ValidatorStakeView>> =
    Map::new("epoch_block_producers_map");
