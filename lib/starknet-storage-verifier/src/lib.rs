use std::collections::BTreeMap;

use bitvec::{order::Msb0, view::BitView};
use pathfinder_crypto::{Felt, hash::pedersen_hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Node {
    node: MerkleNode,
    #[serde(with = "felt")]
    node_hash: Felt,
}

/// A node in the Merkle-Patricia tree, can be a leaf, binary node, or an edge node.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MerkleNode {
    /// Binary/branch node.
    BinaryNode(BinaryNode),
    /// Edge/leaf node.
    EdgeNode(EdgeNode),
}

impl MerkleNode {
    pub fn hash(&self) -> Felt {
        match self {
            MerkleNode::BinaryNode(BinaryNode { left, right }) => pedersen_hash(*left, *right),
            MerkleNode::EdgeNode(EdgeNode {
                path,
                length,
                child,
            }) => pedersen_hash(*child, *path) + Felt::from_u64(*length),
        }
    }
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
    use pathfinder_crypto::Felt;
    use serde::{Deserializer, Serialize, Serializer, de::Deserialize};

    pub fn serialize<S>(data: &Felt, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if serializer.is_human_readable() {
            serializer.collect_str(&data)
        } else {
            data.to_be_bytes().serialize(serializer)
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Felt, D::Error>
    where
        D: Deserializer<'de>,
    {
        if deserializer.is_human_readable() {
            String::deserialize(deserializer)
                .and_then(|s| Felt::from_hex_str(&s).map_err(serde::de::Error::custom))
        } else {
            <[u8; 32]>::deserialize(deserializer)
                .and_then(|bz| Felt::from_be_bytes(bz).map_err(serde::de::Error::custom))
        }
    }
}

#[test]
fn test() {
    let proof: Vec<Node> = serde_json::from_str(
        r#"
  [
    {
      "node": {
        "left": "0x778ebcee8874705995f911f4c7edaac1748f5b583c146e9c37dd48e30d11cfd",
        "right": "0x219c6c95d8eeee035ffa9bd5d301175569b6151874f157c4f9546f0073710db"
      },
      "node_hash": "0x56ef8be5dc020f5437e6611ca54e4f78c245c2e49592de3db76abfe0998eb22"
    },
    {
      "node": {
        "child": "0x56ef8be5dc020f5437e6611ca54e4f78c245c2e49592de3db76abfe0998eb22",
        "length": 1,
        "path": "0x0"
      },
      "node_hash": "0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa"
    },
    {
      "node": {
        "child": "0x1611612cfc15e76d48f227e845073c85f4f55c3ef35921f169f8c475f6a819f",
        "length": 1,
        "path": "0x1"
      },
      "node_hash": "0x778ebcee8874705995f911f4c7edaac1748f5b583c146e9c37dd48e30d11cfd"
    }
  ]
"#,
    )
    .unwrap();

    dbg!(&proof);

    let mut proof = proof
        .into_iter()
        .map(|n| (n.node_hash, n.node))
        .collect::<BTreeMap<_, _>>();

    dbg!(&proof);

    let key = Felt::from_hex_str("0x0").unwrap();
    let value = Felt::from_hex_str("0x0").unwrap();

    // 0x0712ae872c44ec2baee50a19191029e437811fb22de12afb3014642cbe33f09e

    // contracts_proof.contract_leaves_data.storage_root
    let mut expected_hash =
        Felt::from_hex_str("0x2c8771df74e758b1fed285eef0cd07cb84b55abfabfb0d6a0f1b7b3aff761fa")
            .unwrap();

    // https://github.com/eqlabs/pathfinder/blob/a34566b9a9f6ea6d7eb3889130d62c8f3fe6a499/crates/crypto/src/algebra/field/felt.rs#L176
    let mut remaining_path = key.view_bits();

    while let Some(proof_node) = proof.remove(&expected_hash) {
        // Hash mismatch? Return None.
        assert!(proof_node.hash() == expected_hash);

        match proof_node {
            MerkleNode::BinaryNode(BinaryNode { left, right }) => {
                // Set the next hash to be the left or right hash,
                // depending on the direction
                // https://github.com/eqlabs/pathfinder/blob/a34566b9a9f6ea6d7eb3889130d62c8f3fe6a499/crates/merkle-tree/src/merkle_node.rs#L81
                expected_hash = match remaining_path[0] {
                    false => left,
                    true => right,
                };

                // Advance by a single bit
                remaining_path = &remaining_path[1..];
            }
            MerkleNode::EdgeNode(EdgeNode {
                path,
                length,
                child,
            }) => {
                let path_view = &path.view_bits()[(251 - length) as usize..251];
                let remaining_path_view = &remaining_path[..length as usize];

                eprintln!("length: {length}");
                eprintln!("path: {path:x}");
                eprintln!("path_view: {path_view:b}");
                eprintln!("remaining_path_view: {remaining_path_view:b}");

                if path_view != remaining_path_view {
                    // If paths don't match, we've found a proof of non membership because
                    // we:
                    // 1. Correctly moved towards the target insofar as is possible, and
                    // 2. hashing all the nodes along the path does result in the root hash,
                    //    which means
                    // 3. the target definitely does not exist in this tree
                    // return Some(Membership::NonMember);
                    dbg!("non-membership");
                    break;
                }

                // Set the next hash to the child's hash
                expected_hash = child;

                // Advance by the whole edge path
                remaining_path = &remaining_path[length as usize..];
            }
        }
    }

    assert!(proof.is_empty());
    assert_eq!(expected_hash, value);
}
