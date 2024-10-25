use unionlabs::bls::BlsSignature;

use crate::AttestationData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attestation {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: Vec<u8>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct AttestationSsz<C: crate::MAX_VALIDATORS_PER_COMMITTEE> {
    // #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: ssz::types::BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}
