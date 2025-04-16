use unionlabs::primitives::{Bytes, H768};
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ssz::{
        types::{BitList, BitVector},
        Ssz,
    },
};

use crate::phase0::AttestationData;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields)
)]
pub struct Attestation {
    pub aggregation_bits: Bytes,
    pub data: AttestationData,
    pub signature: H768,
    pub committee_bits: Bytes,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Eq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""), deny_unknown_fields)
)]
pub struct AttestationSsz<C: ChainSpec> {
    // #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: BitList<C::MAX_ATTESTATIONS_ELECTRA>,
    pub data: AttestationData,
    pub signature: H768,
    pub committee_bits: BitVector<C::MAX_COMMITTEES_PER_SLOT>,
}
