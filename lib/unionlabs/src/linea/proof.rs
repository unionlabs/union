use macros::model;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MerklePath {
    #[serde(with = "::serde_utils::hex_string")]
    pub value: Vec<u8>,
    #[serde(with = "::serde_utils::hex_string_list")]
    pub proof_related_nodes: Vec<Vec<u8>>,
}

#[model(no_serde)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InclusionProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
    pub leaf_index: u64,
    pub proof: MerklePath,
}

#[model(no_serde)]
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NonInclusionProof {
    #[serde(with = "::serde_utils::hex_string")]
    pub key: Vec<u8>,
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
