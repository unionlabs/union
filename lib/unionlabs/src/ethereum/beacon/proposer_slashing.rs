use macros::model;

use crate::ethereum::beacon::signed_beacon_block_header::SignedBeaconBlockHeader;

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#proposerslashing>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct ProposerSlashing {
    pub signed_header_1: SignedBeaconBlockHeader,
    pub signed_header_2: SignedBeaconBlockHeader,
}
