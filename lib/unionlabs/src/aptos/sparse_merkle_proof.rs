use macros::model;
use serde::{Deserialize, Serialize};

use super::hash_value::HashValue;
use crate::errors::InvalidLength;

/// A proof that can be used to authenticate an element in a Sparse Merkle Tree given trusted root
/// hash. For example, `TransactionInfoToAccountProof` can be constructed on top of this structure.
#[model(proto(
    raw(protos::union::ibc::lightclients::movement::v1::SparseMerkleProof),
    into,
    from
))]
pub struct SparseMerkleProof {
    /// This proof can be used to authenticate whether a given leaf exists in the tree or not.
    ///     - If this is `Some(leaf_node)`
    ///         - If `leaf_node.key` equals requested key, this is an inclusion proof and
    ///           `leaf_node.value_hash` equals the hash of the corresponding account blob.
    ///         - Otherwise this is a non-inclusion proof. `leaf_node.key` is the only key
    ///           that exists in the subtree and `leaf_node.value_hash` equals the hash of the
    ///           corresponding account blob.
    ///     - If this is `None`, this is also a non-inclusion proof which indicates the subtree is
    ///       empty.
    pub leaf: Option<SparseMerkleLeafNode>,

    /// All siblings in this proof, including the default ones. Siblings are ordered from the root
    /// level to the bottom level.
    pub siblings: Vec<HashValue>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SparseMerkleLeafNode {
    pub key: HashValue,
    pub value_hash: HashValue,
}

impl From<SparseMerkleProof> for protos::union::ibc::lightclients::movement::v1::SparseMerkleProof {
    fn from(value: SparseMerkleProof) -> Self {
        Self {
            leaf: value.leaf.map(|leaf| {
                protos::union::ibc::lightclients::movement::v1::SparseMerkleLeafNode {
                    key: leaf.key.into(),
                    value_hash: leaf.value_hash.into(),
                }
            }),
            siblings: value.siblings.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum TryFromSparseMerkleProofError {
    #[error("invalid siblings")]
    Siblings(#[source] InvalidLength),
    #[error("invalid leaf key")]
    LeafKey(#[source] InvalidLength),
    #[error("invalid leaf value hash")]
    LeafValueHash(#[source] InvalidLength),
}

impl TryFrom<protos::union::ibc::lightclients::movement::v1::SparseMerkleProof>
    for SparseMerkleProof
{
    type Error = TryFromSparseMerkleProofError;

    fn try_from(
        value: protos::union::ibc::lightclients::movement::v1::SparseMerkleProof,
    ) -> Result<Self, Self::Error> {
        Ok(Self {
            leaf: value.leaf.map(|leaf| SparseMerkleLeafNode {
                key: leaf
                    .key
                    .try_into()
                    .map_err(TryFromSparseMerkleProofError::LeafKey)
                    .unwrap(),
                value_hash: leaf
                    .value_hash
                    .try_into()
                    .map_err(TryFromSparseMerkleProofError::LeafValueHash)
                    .unwrap(),
            }),
            siblings: value
                .siblings
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<_>, _>>()
                .map_err(TryFromSparseMerkleProofError::Siblings)?,
        })
    }
}
