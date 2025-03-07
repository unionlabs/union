use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ssz::{types::List, Ssz},
};

use crate::phase0::AttestationData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct IndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct IndexedAttestationSsz<C: ChainSpec> {
    pub attesting_indices: List<u64, C::MAX_ATTESTATIONS_ELECTRA>,
    pub data: AttestationData,
    pub signature: H768,
}
