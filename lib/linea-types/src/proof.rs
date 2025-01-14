use serde::{Deserialize, Serialize};
use unionlabs::primitives::Bytes;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerklePath {
    pub value: Bytes,
    pub proof_related_nodes: Vec<Bytes>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InclusionProof {
    pub key: Bytes,
    pub leaf_index: u64,
    pub proof: MerklePath,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonInclusionProof {
    pub key: Bytes,
    pub left_leaf_index: u64,
    pub left_proof: MerklePath,
    pub right_leaf_index: u64,
    pub right_proof: MerklePath,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MerkleProof {
    Inclusion(InclusionProof),
    NonInclusion(NonInclusionProof),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetProof {
    pub account_proof: MerkleProof,
    pub storage_proofs: Vec<MerkleProof>,
}
