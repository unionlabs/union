use macros::model;

use crate::ethereum::beacon::indexed_attestation::UnboundedIndexedAttestation;
#[cfg(feature = "ssz")]
use crate::ethereum::{
    beacon::indexed_attestation::IndexedAttestation, config::MAX_VALIDATORS_PER_COMMITTEE,
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attesterslashing>
#[cfg(feature = "ssz")]
#[model]
#[derive(::ssz::Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct AttesterSlashing<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attestation_1: IndexedAttestation<C>,
    pub attestation_2: IndexedAttestation<C>,
}

#[model]
pub struct UnboundedAttesterSlashing {
    pub attestation_1: UnboundedIndexedAttestation,
    pub attestation_2: UnboundedIndexedAttestation,
}
