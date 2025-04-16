use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {crate::chain_spec::MAX_VALIDATORS_PER_COMMITTEE, ssz::types::BitList};

use crate::phase0::AttestationData;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct Attestation {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: Vec<u8>,
    pub data: AttestationData,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct AttestationSsz<C: MAX_VALIDATORS_PER_COMMITTEE> {
    // #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
}
