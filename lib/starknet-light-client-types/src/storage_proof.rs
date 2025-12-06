use unionlabs::primitives::H256;

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub struct StorageProof {
    nodes: Vec<MerkleNode>,
}

#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize, serde::Deserialize),
    serde(deny_unknown_fields, rename_all = "snake_case")
)]
#[cfg_attr(feature = "bincode", derive(bincode::Encode, bincode::Decode))]
pub enum MerkleNode {
    BinaryNode { left: H256, right: H256 },
    EdgeNode { path: H256, length: u8, child: H256 },
}
