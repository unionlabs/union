use unionlabs::ibc::core::{client::height::Height, commitment::merkle_proof::MerkleProof};

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Header {
    pub l1_height: Height,
    pub l2_height: Height,
    pub l2_consensus_state_proof: MerkleProof,
}
