use parlia_types::ParliaHeader;

/// Misbehaviour for parlia consensus. If there exists two blocks that attest to the different vote data at the same height, then the consensus has misbehaved and the client will be frozen.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct Misbehaviour {
    /// The valset that signed `attestation`.
    pub trusted_valset_epoch_number: u64,

    /// The block that is being attested to as finalized by `attestation_1`.
    pub source_1: ParliaHeader,
    /// The block that is being attested to as justified by `attestation_1`.
    pub target_1: ParliaHeader,
    /// The block that contains the first attestation.
    pub attestation_1: ParliaHeader,

    /// The block that is being attested to as finalized by `attestation_2`.
    pub source_2: ParliaHeader,
    /// The block that is being attested to as justified by `attestation_2`.
    pub target_2: ParliaHeader,
    /// The block that contains the second attestation.
    pub attestation_2: ParliaHeader,
}
