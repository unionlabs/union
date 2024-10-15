use macros::model;
#[cfg(feature = "ssz")]
use {crate::ethereum::config::MAX_VALIDATORS_PER_COMMITTEE, ssz::types::List, ssz::Ssz};

use crate::{ethereum::beacon::attestation_data::AttestationData, hash::H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#indexedattestation>
#[cfg(feature = "ssz")]
#[model]
#[derive(Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct IndexedAttestation<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attesting_indices: List<u64, C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
}

#[model]
pub struct UnboundedIndexedAttestation {
    pub attesting_indices: Vec<u64>,
    pub data: AttestationData,
    pub signature: H768,
}
