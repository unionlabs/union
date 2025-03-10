use unionlabs::primitives::H768;
#[cfg(feature = "ssz")]
use {
    crate::chain_spec::ChainSpec,
    ssz::{
        types::{BitList, BitVector},
        Ssz,
    },
};

use crate::phase0::AttestationData;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attestation {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: Vec<u8>,
    pub data: AttestationData,
    pub signature: H768,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct AttestationSsz<C: ChainSpec> {
    // #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
    committee_bits: BitVector<C::MAX_COMMITTEES_PER_SLOT>,
}
