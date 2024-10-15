use macros::model;
use ssz::Ssz;

use crate::{ethereum::beacon::voluntary_exit::VoluntaryExit, hash::H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
#[model]
#[derive(Ssz)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: H768,
}
