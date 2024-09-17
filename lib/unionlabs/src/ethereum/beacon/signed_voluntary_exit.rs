use macros::model;
use ssz::Ssz;

use crate::{bls::BlsSignature, ethereum::beacon::voluntary_exit::VoluntaryExit};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#signedvoluntaryexit>
#[model]
#[derive(Ssz)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: BlsSignature,
}
