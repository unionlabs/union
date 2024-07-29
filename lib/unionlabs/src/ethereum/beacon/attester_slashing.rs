use macros::model;
use ssz::Ssz;

use crate::ethereum::{
    beacon::indexed_attestation::{IndexedAttestation, UnboundedIndexedAttestation},
    config::MAX_VALIDATORS_PER_COMMITTEE,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct AttesterSlashing<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

#[model]
pub struct UnboundedAttesterSlashing {
    pub attestation_1: UnboundedIndexedAttestation,
    pub attestation_2: UnboundedIndexedAttestation,
}
