use macros::model;
#[cfg(feature = "ssz")]
use {
    crate::ethereum::config::MAX_VALIDATORS_PER_COMMITTEE,
    ssz::{types::BitList, Ssz},
};

use crate::{ethereum::beacon::attestation_data::AttestationData, hash::H768};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
#[cfg(feature = "ssz")]
#[model]
#[derive(Ssz)]
#[cfg_attr(feature = "serde", serde(bound(serialize = "", deserialize = "")))]
pub struct Attestation<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub aggregation_bits: BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: H768,
}

#[model]
pub struct UnboundedAttestation {
    #[cfg_attr(feature = "serde", serde(with = "::serde_utils::hex_string"))]
    pub aggregation_bits: Vec<u8>,
    pub data: AttestationData,
    pub signature: H768,
}
