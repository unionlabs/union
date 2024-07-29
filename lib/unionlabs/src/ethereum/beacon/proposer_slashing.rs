use macros::model;
use ssz::Ssz;

use crate::ethereum::beacon::signed_beacon_block_header::SignedBeaconBlockHeader;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
#[model]
#[derive(Ssz)]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}
