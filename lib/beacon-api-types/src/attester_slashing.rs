use crate::IndexedAttestation;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttesterSlashing {
    pub attestation_1: IndexedAttestation,
    pub attestation_2: IndexedAttestation,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct AttesterSlashingSsz<C: crate::MAX_VALIDATORS_PER_COMMITTEE> {
    pub attestation_1: crate::IndexedAttestationSsz<C>,
    pub attestation_2: crate::IndexedAttestationSsz<C>,
}
