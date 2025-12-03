use serde::{Deserialize, Serialize};
use starknet_crypto::Felt;

fn verify_storage_proof(root: Felt, proof: Vec<MerkleNode>) {}

/// A node in the Merkle-Patricia tree, can be a leaf, binary node, or an edge node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MerkleNode {
    /// Binary/branch node.
    BinaryNode(BinaryNode),
    /// Edge/leaf node.
    EdgeNode(EdgeNode),
}

/// An internal node whose both children are non-zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BinaryNode {
    /// The hash of the left child
    #[serde(with = "felt")]
    pub left: Felt,
    /// The hash of the right child
    #[serde(with = "felt")]
    pub right: Felt,
}

/// Represents a path to the highest non-zero descendant node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EdgeNode {
    /// An unsigned integer whose binary representation represents the path from the current node to
    /// its highest non-zero descendant (bounded by 2^251)
    #[serde(with = "felt")]
    pub path: Felt,
    /// The length of the path (bounded by 251)
    pub length: u64,
    /// The hash of the unique non-zero maximal-height descendant node
    #[serde(with = "felt")]
    pub child: Felt,
}

pub mod felt {
    use serde::{Deserializer, Serialize, Serializer, de::Deserialize};
    use starknet_crypto::Felt;

    pub fn serialize<S>(data: &Felt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(&data)
        } else {
            data.to_bytes_be().serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Felt, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| Felt::from_hex(&s).map_err(serde::de::Error::custom))
        } else {
            <[u8; 32]>::deserialize(deserializer).map(|bz| Felt::from_bytes_be(&bz))
        }
    }
}
