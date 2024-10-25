use unionlabs::bls::BlsPublicKey;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncCommittee {
    pub pubkeys: Vec<BlsPublicKey>,
    pub aggregate_pubkey: BlsPublicKey,
}

#[cfg(feature = "ssz")]
#[derive(Debug, Clone, PartialEq, ssz::Ssz)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SyncCommitteeSsz<C: crate::SYNC_COMMITTEE_SIZE> {
    pub pubkeys: ssz::types::Vector<BlsPublicKey, C::SYNC_COMMITTEE_SIZE>,
    pub aggregate_pubkey: BlsPublicKey,
}
