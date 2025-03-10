#[cfg(feature = "ssz")]
use {
    crate::{chain_spec::ChainSpec, electra::IndexedAttestationSsz},
    ssz::Ssz,
};

use crate::electra::IndexedAttestation;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttesterSlashing {
    pub attestation_1: IndexedAttestation,
    pub attestation_2: IndexedAttestation,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct AttesterSlashingSsz<C: ChainSpec> {
    pub attestation_1: IndexedAttestationSsz<C>,
    pub attestation_2: IndexedAttestationSsz<C>,
}
