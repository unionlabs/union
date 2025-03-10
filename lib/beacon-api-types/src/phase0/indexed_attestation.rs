use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {crate::chain_spec::MAX_VALIDATORS_PER_COMMITTEE, ssz::types::List};

use crate::phase0::AttestationData;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedAttestationSsz<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attesting_indices: List<u64, C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
}
