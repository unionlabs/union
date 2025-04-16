use crate::phase0::IndexedAttestation;
#[cfg(feature = "ssz")]
use crate::{chain_spec::MAX_VALIDATORS_PER_COMMITTEE, phase0::IndexedAttestationSsz};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct AttesterSlashing {
    pub attestation_1: IndexedAttestation,
    pub attestation_2: IndexedAttestation,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct AttesterSlashingSsz<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub attestation_1: IndexedAttestationSsz<C>,
    pub attestation_2: IndexedAttestationSsz<C>,
}
