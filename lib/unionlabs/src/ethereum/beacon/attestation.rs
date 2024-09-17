use macros::model;
use ssz::{types::BitList, Ssz};

use crate::{
    bls::BlsSignature,
    ethereum::{beacon::attestation_data::AttestationData, config::MAX_VALIDATORS_PER_COMMITTEE},
};

/// <https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#attestation>
#[model]
#[derive(Ssz)]
#[serde(bound(serialize = "", deserialize = ""))]
pub struct Attestation<C: MAX_VALIDATORS_PER_COMMITTEE> {
    pub aggregation_bits: BitList<C::MAX_VALIDATORS_PER_COMMITTEE>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}

#[model]
pub struct UnboundedAttestation {
    #[serde(with = "::serde_utils::hex_string")]
    pub aggregation_bits: Vec<u8>,
    pub data: AttestationData,
    pub signature: BlsSignature,
}
