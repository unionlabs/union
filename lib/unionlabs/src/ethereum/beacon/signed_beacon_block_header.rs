use macros::model;
use ssz::Ssz;

use crate::{hash::H768, ibc::lightclients::ethereum::beacon_block_header::BeaconBlockHeader};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedbeaconblockheader>
#[model]
#[derive(Ssz)]
pub struct SignedBeaconBlockHeader {
    pub message: BeaconBlockHeader,
    pub signature: H768,
}
