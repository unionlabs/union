use macros::model;

use crate::{ethereum::beacon::voluntary_exit::VoluntaryExit, hash::H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
#[model]
#[cfg_attr(feature = "ssz", derive(::ssz::Ssz))]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: H768,
}
