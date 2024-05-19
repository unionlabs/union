mod contract;
mod nibble_slice;
mod types;
use std::collections::HashMap;

pub use contract::*;
use near_primitives_core::{hash::CryptoHash, types::AccountId};
use near_sdk::borsh::{BorshDeserialize, BorshSerialize};
use types::BlockHeaderInnerLiteView;

use crate::nibble_slice::NibbleSlice;

#[derive(BorshSerialize, BorshDeserialize)]
pub struct ClientState {
    latest_height: u64,
    ibc_account_id: AccountId,
}

#[derive(Debug, Clone, BorshSerialize, BorshDeserialize)]
pub struct ConsensusState {
    pub state_root: CryptoHash,
    pub state: BlockHeaderInnerLiteView,
}

pub struct StateProof {
    pub nodes: HashMap<CryptoHash, RawTrieNodeWithSize>,
}

impl StateProof {
    pub fn parse(data: Vec<Vec<u8>>) -> StateProof {
        let nodes = data
            .into_iter()
            .map(|bytes| {
                let hash = CryptoHash::hash_bytes(&bytes);
                let node = RawTrieNodeWithSize::try_from_slice(&bytes).unwrap();
                (hash, node)
            })
            .collect();

        StateProof { nodes }
    }

    pub fn verify(
        &self,
        state_root: &CryptoHash,
        account_id: &AccountId,
        key: &[u8],
        expected: Option<&[u8]>,
    ) -> bool {
        let mut query = Vec::with_capacity(1 + account_id.len() + 1 + key.len());
        query.push(9_u8);
        query.extend(account_id.as_bytes());
        query.extend(b",");
        query.extend(key);
        let mut key = NibbleSlice::new(&query);

        let mut expected_hash = state_root;
        while let Some(node) = self.nodes.get(expected_hash) {
            match &node.node {
                RawTrieNode::Leaf(node_key, value) => {
                    let nib = &NibbleSlice::from_encoded(&node_key).0;
                    return if &key != nib {
                        expected.is_none()
                    } else {
                        expected.is_some_and(|expected| value == expected)
                    };
                }
                RawTrieNode::Extension(node_key, child_hash) => {
                    expected_hash = child_hash;

                    // To avoid unnecessary copy
                    let nib = NibbleSlice::from_encoded(&node_key).0;
                    if !key.starts_with(&nib) {
                        return expected.is_none();
                    }
                    key = key.mid(nib.len());
                }
                RawTrieNode::BranchNoValue(children) => {
                    if key.is_empty() {
                        return expected.is_none();
                    }
                    match children[key.at(0)] {
                        Some(ref child_hash) => {
                            key = key.mid(1);
                            expected_hash = child_hash;
                        }
                        None => return expected.is_none(),
                    }
                }
                RawTrieNode::BranchWithValue(value, children) => {
                    if key.is_empty() {
                        return expected.is_some_and(|exp| value == exp);
                    }
                    match children[key.at(0)] {
                        Some(ref child_hash) => {
                            key = key.mid(1);
                            expected_hash = child_hash;
                        }
                        None => return expected.is_none(),
                    }
                }
            }
        }
        false
    }
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct RawTrieNodeWithSize {
    pub node: RawTrieNode,
    pub memory_usage: u64,
}

#[derive(BorshSerialize, BorshDeserialize, Clone, Debug, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum RawTrieNode {
    /// Leaf(key, value_length, value_hash)
    Leaf(Vec<u8>, ValueRef),
    /// Branch(children)
    BranchNoValue(Children),
    /// Branch(children, value)
    BranchWithValue(ValueRef, Children),
    /// Extension(key, child)
    Extension(Vec<u8>, CryptoHash),
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Children<T = CryptoHash>(pub [Option<T>; 16]);

impl<T> Default for Children<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T> std::ops::Index<u8> for Children<T> {
    type Output = Option<T>;
    fn index(&self, index: u8) -> &Option<T> {
        &self.0[usize::from(index)]
    }
}

impl<T> std::ops::IndexMut<u8> for Children<T> {
    fn index_mut(&mut self, index: u8) -> &mut Option<T> {
        &mut self.0[usize::from(index)]
    }
}

impl<T: BorshSerialize> BorshSerialize for Children<T> {
    fn serialize<W: std::io::Write>(&self, wr: &mut W) -> std::io::Result<()> {
        let mut bitmap: u16 = 0;
        let mut pos: u16 = 1;
        for child in self.0.iter() {
            if child.is_some() {
                bitmap |= pos
            }
            pos <<= 1;
        }
        bitmap.serialize(wr)?;
        self.0
            .iter()
            .flat_map(Option::as_ref)
            .map(|child| child.serialize(wr))
            .collect()
    }
}

impl<T: BorshDeserialize> BorshDeserialize for Children<T> {
    fn deserialize_reader<R: std::io::Read>(rd: &mut R) -> std::io::Result<Self> {
        let mut bitmap = u16::deserialize_reader(rd)?;
        let mut children = Self::default();
        while bitmap != 0 {
            let idx = bitmap.trailing_zeros() as u8;
            bitmap &= bitmap - 1;
            children[idx] = Some(T::deserialize_reader(rd)?);
        }
        Ok(children)
    }
}

#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ValueRef {
    /// Value length in bytes.
    pub length: u32,
    /// Unique value hash.
    pub hash: CryptoHash,
}

impl ValueRef {
    /// Returns length of the referenced value.
    pub fn len(&self) -> usize {
        usize::try_from(self.length).unwrap()
    }
}

impl std::cmp::PartialEq<[u8]> for ValueRef {
    fn eq(&self, rhs: &[u8]) -> bool {
        self.len() == rhs.len() && self.hash == CryptoHash::hash_bytes(rhs)
    }
}
