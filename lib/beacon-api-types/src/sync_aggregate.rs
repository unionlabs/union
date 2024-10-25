use unionlabs::bls::BlsSignature;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnboundedSyncAggregate {
    #[serde(with = "::serde_utils::hex_string")]
    pub sync_committee_bits: Vec<u8>,
    pub sync_committee_signature: BlsSignature,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(bound(serialize = "", deserialize = ""))
)]
pub struct SyncAggregate<C: crate::SYNC_COMMITTEE_SIZE> {
    // TODO: Change debug print for this type in ssz::types
    // #[debug("BitVector({})", sync_committee_bits.iter().map(|b| if b { '1' } else { '0' }).collect::<String>())]
    pub sync_committee_bits: ssz::types::BitVector<C::SYNC_COMMITTEE_SIZE>,
    pub sync_committee_signature: BlsSignature,
}
