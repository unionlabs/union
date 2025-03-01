#[cfg(feature = "ssz")]
use ssz::types::List;
use unionlabs::primitives::H768;

use crate::AttestationData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedAttestationSsz<C: crate::MAX_VALIDATORS_PER_COMMITTEE> {
    pub attesting_indices: List<u64, C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
}
