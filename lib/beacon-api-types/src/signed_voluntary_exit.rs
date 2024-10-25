use macros::model;
use ssz::Ssz;

use crate::{bls::BlsSignature, ethereum::beacon::voluntary_exit::VoluntaryExit};

#[model]
#[derive(Ssz)]
pub struct SignedVoluntaryExit {
    pub message: VoluntaryExit,
    pub signature: BlsSignature,
}
